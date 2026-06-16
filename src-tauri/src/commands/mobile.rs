use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::http::HttpClientState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileLoginRequest {
    pub server_url: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileLoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub logged_in: bool,
    pub server_url: Option<String>,
    pub user: Option<serde_json::Value>,
}

fn normalize_server_url(value: &str) -> String {
    value.trim().trim_end_matches('/').to_string()
}

#[tauri::command]
pub async fn mobile_login(
    request: MobileLoginRequest,
    state: State<'_, DbState>,
    client_state: State<'_, HttpClientState>,
) -> Result<AuthStatus, String> {
    let server_url = normalize_server_url(&request.server_url);
    if server_url.is_empty() {
        return Err("服务端地址不能为空".into());
    }

    let response = client_state
        .client
        .post(format!("{server_url}/api/mobile/auth/login"))
        .json(&serde_json::json!({
            "email": request.email,
            "password": request.password,
        }))
        .send()
        .await
        .map_err(|e| format!("登录请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("登录失败: {}", response.status()));
    }

    let login = response
        .json::<MobileLoginResponse>()
        .await
        .map_err(|e| format!("登录响应解析失败: {e}"))?;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "server_url", &server_url).map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "access_token", &login.access_token).map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "refresh_token", &login.refresh_token).map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "access_expires_in", &login.expires_in.to_string()).map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "mobile_user", &login.user.to_string()).map_err(|e| e.to_string())?;

    Ok(AuthStatus {
        logged_in: true,
        server_url: Some(server_url),
        user: Some(login.user),
    })
}

#[tauri::command]
pub fn get_auth_status(state: State<'_, DbState>) -> Result<AuthStatus, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let access = crate::db::error_repo::get_sync_value(&conn, "access_token").map_err(|e| e.to_string())?;
    let server_url = crate::db::error_repo::get_sync_value(&conn, "server_url").map_err(|e| e.to_string())?;
    let user = crate::db::error_repo::get_sync_value(&conn, "mobile_user")
        .map_err(|e| e.to_string())?
        .and_then(|value| serde_json::from_str(&value).ok());
    Ok(AuthStatus {
        logged_in: access.is_some(),
        server_url,
        user,
    })
}
