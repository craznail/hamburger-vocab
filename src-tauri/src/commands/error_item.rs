use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tauri::{Manager, State};

use crate::db::models::{
    AnalyzeErrorFailureResponse, AnalyzeErrorResponse, ErrorDraft, ErrorItem, ErrorNotebook,
    ErrorReviewResult, ErrorSyncConflict, RemoteErrorImage,
};
use crate::db::DbState;
use crate::http::HttpClientState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateErrorDraftRequest {
    pub image_base64: String,
    pub mime_type: Option<String>,
    pub notebook_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveErrorItemRequest {
    pub id: String,
    pub question_text: Option<String>,
    pub answer_text: Option<String>,
    pub analysis: Option<String>,
    pub mistake_analysis: Option<String>,
    pub user_notes: Option<String>,
    pub knowledge_points: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateErrorItemRequest {
    pub id: String,
    pub quality: i64,
    pub duration_seconds: Option<i64>,
}

fn parse_data_url(value: &str, mime_type: Option<String>) -> Result<(Vec<u8>, String), String> {
    if let Some(rest) = value.strip_prefix("data:") {
        let (meta, data) = rest
            .split_once(',')
            .ok_or_else(|| "图片 Data URL 格式无效".to_string())?;
        let detected_mime = meta.split(';').next().unwrap_or("image/jpeg").to_string();
        let bytes = STANDARD
            .decode(data)
            .map_err(|e| format!("图片 base64 解码失败: {e}"))?;
        Ok((bytes, detected_mime))
    } else {
        let bytes = STANDARD
            .decode(value)
            .map_err(|e| format!("图片 base64 解码失败: {e}"))?;
        Ok((bytes, mime_type.unwrap_or_else(|| "image/jpeg".into())))
    }
}

fn image_ext(mime_type: &str) -> &'static str {
    if mime_type.contains("png") {
        "png"
    } else if mime_type.contains("webp") {
        "webp"
    } else {
        "jpg"
    }
}

fn resolve_remote_image_url(server_url: &str, remote_url: &str) -> String {
    let remote_url = remote_url.trim();
    if remote_url.is_empty()
        || remote_url.starts_with("http://")
        || remote_url.starts_with("https://")
        || remote_url.starts_with("data:")
    {
        return remote_url.to_string();
    }

    let server_url = server_url.trim().trim_end_matches('/');
    if server_url.is_empty() {
        return remote_url.to_string();
    }

    if remote_url.starts_with('/') {
        format!("{server_url}{remote_url}")
    } else {
        format!("{server_url}/{remote_url}")
    }
}

fn normalize_remote_image(image: &mut RemoteErrorImage, server_url: &str) {
    if let Some(url) = image.url.as_deref() {
        image.url = Some(resolve_remote_image_url(server_url, url));
    }
}

fn normalize_pulled_item_image_url(item: &mut serde_json::Value, server_url: &str) {
    let Some(image) = item
        .get_mut("image")
        .and_then(|value| value.as_object_mut())
    else {
        return;
    };
    let Some(url_value) = image.get_mut("url") else {
        return;
    };
    let Some(url) = url_value.as_str() else {
        return;
    };
    *url_value = serde_json::Value::String(resolve_remote_image_url(server_url, url));
}

fn read_token_and_server(conn: &rusqlite::Connection) -> Result<(String, String), String> {
    let server_url = crate::commands::mobile::load_normalized_server_url(conn)?
        .ok_or_else(|| "请先登录远程服务端".to_string())?;
    let access_token = crate::db::error_repo::get_sync_value(conn, "access_token")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "请先登录远程服务端".to_string())?;
    Ok((server_url, access_token))
}

fn access_token_needs_refresh(conn: &rusqlite::Connection) -> Result<bool, String> {
    let Some(expires_at) = crate::db::error_repo::get_sync_value(conn, "access_expires_at")
        .map_err(|e| e.to_string())?
    else {
        return Ok(false);
    };
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&expires_at) else {
        return Ok(false);
    };
    Ok(parsed.with_timezone(&chrono::Utc) <= chrono::Utc::now())
}

async fn ensure_valid_access_token(
    state: &State<'_, DbState>,
    client_state: &State<'_, HttpClientState>,
) -> Result<(String, String), String> {
    let needs_refresh = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        access_token_needs_refresh(&conn)?
    };

    if needs_refresh {
        return crate::commands::mobile::refresh_access_token_inner(state, &client_state.client)
            .await;
    }

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    read_token_and_server(&conn)
}

async fn send_authenticated(
    state: &State<'_, DbState>,
    client_state: &State<'_, HttpClientState>,
    method: reqwest::Method,
    url: String,
    body: Option<&serde_json::Value>,
    query: Option<&[(&str, String)]>,
    timeout: Option<Duration>,
) -> Result<reqwest::Response, String> {
    let (mut server_url, mut access_token) = ensure_valid_access_token(state, client_state).await?;
    let build_request = |client: &reqwest::Client, token: &str, server_url_value: &str| {
        let target = url.replace("{server_url}", server_url_value);
        let mut request = client.request(method.clone(), target).bearer_auth(token);
        if let Some(body) = body {
            request = request.json(body);
        }
        if let Some(query) = query {
            request = request.query(query);
        }
        if let Some(timeout) = timeout {
            request = request.timeout(timeout);
        }
        request
    };

    let first = build_request(&client_state.client, &access_token, &server_url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    if first.status() != reqwest::StatusCode::UNAUTHORIZED {
        return Ok(first);
    }

    let refreshed =
        crate::commands::mobile::refresh_access_token_inner(state, &client_state.client).await?;
    server_url = refreshed.0;
    access_token = refreshed.1;
    build_request(&client_state.client, &access_token, &server_url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))
}

#[tauri::command]
pub fn create_error_draft(
    app_handle: tauri::AppHandle,
    request: CreateErrorDraftRequest,
    state: State<'_, DbState>,
) -> Result<ErrorDraft, String> {
    let (bytes, mime_type) = parse_data_url(&request.image_base64, request.mime_type)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let image_dir = app_data_dir.join("images").join("error-items");
    fs::create_dir_all(&image_dir).map_err(|e| format!("创建图片目录失败: {e}"))?;
    let local_path = image_dir.join(format!("{}.{}", sha256, image_ext(&mime_type)));
    if !Path::new(&local_path).exists() {
        fs::write(&local_path, &bytes).map_err(|e| format!("保存图片失败: {e}"))?;
    }

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::create_error_draft(
        &conn,
        request.notebook_id.as_deref(),
        &local_path.to_string_lossy(),
        &sha256,
        &mime_type,
    )
    .map_err(|_| "请先从服务端同步可用错题本，再创建本地错题".to_string())
}

#[tauri::command]
pub fn get_error_notebooks(state: State<'_, DbState>) -> Result<Vec<ErrorNotebook>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::get_error_notebooks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_error_items(
    state: State<'_, DbState>,
    notebook_id: Option<String>,
) -> Result<Vec<ErrorItem>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::get_error_items(&conn, notebook_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_due_error_items(state: State<'_, DbState>) -> Result<Vec<ErrorItem>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::get_due_error_items(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_error_item(
    request: SaveErrorItemRequest,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let knowledge_points = request
        .knowledge_points
        .as_ref()
        .map(|points| serde_json::to_string(points).unwrap_or_else(|_| "[]".into()));
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::update_error_item_text(
        &conn,
        &request.id,
        request.question_text.as_deref(),
        request.answer_text.as_deref(),
        request.analysis.as_deref(),
        request.mistake_analysis.as_deref(),
        request.user_notes.as_deref(),
        knowledge_points.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn analyze_error_draft(
    id: String,
    state: State<'_, DbState>,
    client_state: State<'_, HttpClientState>,
) -> Result<ErrorItem, String> {
    let item = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let item = crate::db::error_repo::get_error_item(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "错题草稿不存在".to_string())?;
        crate::db::error_repo::mark_item_analyzing(&conn, &id).map_err(|e| e.to_string())?;
        item
    };
    let (server_url, _) = ensure_valid_access_token(&state, &client_state).await?;

    let local_path = item
        .local_image_path
        .clone()
        .ok_or_else(|| "本地图片不存在".to_string())?;
    let bytes = fs::read(&local_path).map_err(|e| format!("读取本地图片失败: {e}"))?;
    let image_base64 = STANDARD.encode(bytes);

    let body = serde_json::json!({
        "localId": id,
        "notebookId": item.notebook_id,
        "imageBase64": image_base64,
        "mimeType": "image/jpeg",
        "language": "zh"
    });
    let response = send_authenticated(
        &state,
        &client_state,
        reqwest::Method::POST,
        "{server_url}/api/mobile/error-items/analyze".into(),
        Some(&body),
        None,
        Some(Duration::from_secs(180)),
    )
    .await
    .map_err(|e| {
        let conn = state.conn.lock().ok();
        if let Some(conn) = conn {
            let _ = crate::db::error_repo::mark_analyze_failed(&conn, &id);
        }
        e
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        if let Ok(mut failure) = serde_json::from_str::<AnalyzeErrorFailureResponse>(&text) {
            if let Some(image) = failure.image.as_mut() {
                normalize_remote_image(image, &server_url);
            }
            crate::db::error_repo::apply_analyze_failure_response(&conn, &id, &failure)
                .map_err(|e| e.to_string())?;
            return Err(format!(
                "AI 分析失败: {} ({status})",
                failure.message.unwrap_or_else(|| failure
                    .code
                    .unwrap_or_else(|| "服务端未返回详细原因".into()))
            ));
        }
        crate::db::error_repo::mark_analyze_failed(&conn, &id).map_err(|e| e.to_string())?;
        return Err(format!("AI 分析失败: {status} {text}"));
    }

    let mut parsed = response
        .json::<AnalyzeErrorResponse>()
        .await
        .map_err(|e| format!("AI 分析响应解析失败: {e}"))?;
    if let Some(image) = parsed.image.as_mut() {
        normalize_remote_image(image, &server_url);
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::apply_analyze_response(&conn, &id, &parsed)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::get_error_item(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "错题不存在".into())
}

#[tauri::command]
pub fn rate_error_item(
    request: RateErrorItemRequest,
    state: State<'_, DbState>,
) -> Result<ErrorReviewResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::rate_error_item(
        &conn,
        &request.id,
        request.quality,
        request.duration_seconds.unwrap_or(0),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_error_sync_conflicts(
    state: State<'_, DbState>,
) -> Result<Vec<ErrorSyncConflict>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::list_error_sync_conflicts(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_error_sync_conflict_keep_local(
    local_item_id: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::resolve_conflict_keep_local(&conn, &local_item_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_error_sync_conflict_accept_remote(
    local_item_id: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::resolve_conflict_accept_remote(&conn, &local_item_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_error_items(
    state: State<'_, DbState>,
    client_state: State<'_, HttpClientState>,
) -> Result<serde_json::Value, String> {
    let (ops, cursor, device_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let ops =
            crate::db::error_repo::get_syncable_pending_ops(&conn).map_err(|e| e.to_string())?;
        let cursor = crate::db::error_repo::get_sync_value(&conn, "last_error_sync_cursor")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "0".into());
        let device_id =
            crate::db::error_repo::ensure_device_id(&conn).map_err(|e| e.to_string())?;
        (ops, cursor, device_id)
    };

    let push_body = serde_json::json!({
        "deviceId": device_id,
        "ops": ops.iter().map(|op| serde_json::json!({
            "opId": op.op_id,
            "entityType": op.entity_type,
            "action": op.action,
            "localItemId": op.local_item_id,
            "remoteItemId": op.remote_item_id,
            "baseVersion": op.base_version,
            "payload": op.payload,
            "clientTimestamp": op.client_timestamp,
        })).collect::<Vec<_>>(),
    });

    let push = send_authenticated(
        &state,
        &client_state,
        reqwest::Method::POST,
        "{server_url}/api/mobile/sync/push".into(),
        Some(&push_body),
        None,
        None,
    )
    .await?;
    if !push.status().is_success() {
        return Err(format!("同步推送失败: {}", push.status()));
    }
    let push_payload = push
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("同步推送响应解析失败: {e}"))?;

    {
        let mut conn = state.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if let Some(accepted) = push_payload
            .get("acceptedOps")
            .and_then(|value| value.as_array())
        {
            for op in accepted {
                crate::db::error_repo::acknowledge_accepted_op(&tx, op)
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(conflicts) = push_payload
            .get("conflictedOps")
            .and_then(|value| value.as_array())
        {
            for conflict in conflicts {
                crate::db::error_repo::record_sync_conflict(&tx, conflict)
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    let pull_cursor = cursor;
    let pull = send_authenticated(
        &state,
        &client_state,
        reqwest::Method::GET,
        "{server_url}/api/mobile/sync/pull".into(),
        None,
        Some(&[("cursor", pull_cursor.clone())]),
        None,
    )
    .await?;
    if !pull.status().is_success() {
        return Err(format!("同步拉取失败: {}", pull.status()));
    }
    let mut payload = pull
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("同步拉取响应解析失败: {e}"))?;

    let server_url = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        crate::commands::mobile::load_normalized_server_url(&conn)?.unwrap_or_default()
    };

    let mut conn = state.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if let Some(notebooks) = payload.get("notebooks").and_then(|value| value.as_array()) {
        crate::db::error_repo::replace_notebooks(&tx, notebooks).map_err(|e| e.to_string())?;
    }
    if let Some(items) = payload
        .get_mut("errorItems")
        .and_then(|value| value.as_array_mut())
    {
        for item in items {
            normalize_pulled_item_image_url(item, &server_url);
            let remote_id = item
                .get("remoteId")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "远端错题缺少 remoteId".to_string())?;
            let local_id_hint = item.get("localId").and_then(|value| value.as_str());
            let overwrite = if let Some(local_item_id) =
                crate::db::error_repo::find_local_item_id_by_remote_id(
                    &tx,
                    remote_id,
                    local_id_hint,
                )
                .map_err(|e| e.to_string())?
            {
                !crate::db::error_repo::local_item_has_blocking_sync_state(&tx, &local_item_id)
                    .map_err(|e| e.to_string())?
            } else {
                true
            };
            crate::db::error_repo::upsert_remote_item_snapshot(&tx, item, overwrite)
                .map_err(|e| e.to_string())?;
        }
    }
    if let Some(deleted_ids) = payload.get("deletedIds").and_then(|value| value.as_array()) {
        crate::db::error_repo::apply_pulled_deletions(&tx, deleted_ids)
            .map_err(|e| e.to_string())?;
    }
    let next_cursor = payload
        .get("nextCursor")
        .and_then(|value| value.as_i64())
        .map(|value| value.to_string())
        .or_else(|| {
            payload
                .get("serverCursor")
                .and_then(|value| value.as_i64())
                .map(|value| value.to_string())
        })
        .unwrap_or(pull_cursor);
    crate::db::error_repo::set_sync_value(&tx, "last_error_sync_cursor", &next_cursor)
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(payload)
}
