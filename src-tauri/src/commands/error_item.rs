use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tauri::{Manager, State};

use crate::db::models::{
    AnalyzeErrorFailureResponse, AnalyzeErrorResponse, ErrorDraft, ErrorItem, ErrorNotebook,
    ErrorReviewResult,
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

fn read_token_and_server(conn: &rusqlite::Connection) -> Result<(String, String), String> {
    let server_url = crate::db::error_repo::get_sync_value(conn, "server_url")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "请先登录远程服务端".to_string())?;
    let access_token = crate::db::error_repo::get_sync_value(conn, "access_token")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "请先登录远程服务端".to_string())?;
    Ok((server_url, access_token))
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
    .map_err(|e| e.to_string())
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
    let (item, server_url, access_token) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let item = crate::db::error_repo::get_error_item(&conn, &id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "错题草稿不存在".to_string())?;
        let (server_url, access_token) = read_token_and_server(&conn)?;
        conn.execute(
            "UPDATE error_items SET sync_status = 'analyzing', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![crate::db::now_str(), id],
        )
        .map_err(|e| e.to_string())?;
        (item, server_url, access_token)
    };

    let local_path = item
        .local_image_path
        .clone()
        .ok_or_else(|| "本地图片不存在".to_string())?;
    let bytes = fs::read(&local_path).map_err(|e| format!("读取本地图片失败: {e}"))?;
    let image_base64 = STANDARD.encode(bytes);

    let response = client_state
        .client
        .post(format!("{server_url}/api/mobile/error-items/analyze"))
        .bearer_auth(access_token)
        .json(&serde_json::json!({
            "localId": id,
            "imageBase64": image_base64,
            "mimeType": "image/jpeg",
            "language": "zh"
        }))
        .send()
        .await
        .map_err(|e| {
            let conn = state.conn.lock().ok();
            if let Some(conn) = conn {
                let _ = crate::db::error_repo::mark_analyze_failed(&conn, &id);
            }
            format!("AI 分析请求失败: {e}")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        if let Ok(failure) = serde_json::from_str::<AnalyzeErrorFailureResponse>(&text) {
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

    let parsed = response
        .json::<AnalyzeErrorResponse>()
        .await
        .map_err(|e| format!("AI 分析响应解析失败: {e}"))?;
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
pub async fn sync_error_items(
    state: State<'_, DbState>,
    client_state: State<'_, HttpClientState>,
) -> Result<serde_json::Value, String> {
    let (server_url, access_token, review_logs, since) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let (server_url, access_token) = read_token_and_server(&conn)?;
        let review_logs =
            crate::db::error_repo::pending_review_logs_json(&conn).map_err(|e| e.to_string())?;
        let since = crate::db::error_repo::get_sync_value(&conn, "last_error_sync_at")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".into());
        (server_url, access_token, review_logs, since)
    };

    let push = client_state
        .client
        .post(format!("{server_url}/api/mobile/sync/push"))
        .bearer_auth(&access_token)
        .json(&serde_json::json!({
            "errorItems": [],
            "reviewLogs": review_logs,
            "deletedIds": [],
        }))
        .send()
        .await
        .map_err(|e| format!("同步推送失败: {e}"))?;
    if !push.status().is_success() {
        return Err(format!("同步推送失败: {}", push.status()));
    }

    let pull = client_state
        .client
        .get(format!("{server_url}/api/mobile/sync/pull"))
        .query(&[("since", since)])
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| format!("同步拉取失败: {e}"))?;
    if !pull.status().is_success() {
        return Err(format!("同步拉取失败: {}", pull.status()));
    }
    let payload = pull
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    let server_time = payload
        .get("serverTime")
        .and_then(|v| v.as_str())
        .unwrap_or("1970-01-01T00:00:00.000Z")
        .to_string();

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if let Some(items) = payload.get("errorItems").and_then(|v| v.as_array()) {
        for item in items {
            crate::db::error_repo::upsert_pulled_error_item(&conn, item)
                .map_err(|e| e.to_string())?;
        }
    }
    if let Some(deleted_ids) = payload.get("deletedIds").and_then(|v| v.as_array()) {
        crate::db::error_repo::apply_pulled_deletions(&conn, deleted_ids)
            .map_err(|e| e.to_string())?;
    }
    crate::db::error_repo::mark_review_logs_synced(&conn).map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "last_error_sync_at", &server_time)
        .map_err(|e| e.to_string())?;
    Ok(payload)
}
