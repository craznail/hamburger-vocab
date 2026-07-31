use chrono::{DateTime, Utc};
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileRefreshResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub logged_in: bool,
    pub server_url: Option<String>,
    pub user: Option<serde_json::Value>,
}

fn normalize_server_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(mut parsed) = reqwest::Url::parse(trimmed) else {
        return trimmed.to_string();
    };

    if parsed
        .host_str()
        .is_some_and(|host| host.eq_ignore_ascii_case("localhost"))
    {
        let _ = parsed.set_host(Some("127.0.0.1"));
    }

    parsed.to_string().trim_end_matches('/').to_string()
}

pub(crate) fn load_normalized_server_url(
    conn: &rusqlite::Connection,
) -> Result<Option<String>, String> {
    let Some(server_url) =
        crate::db::error_repo::get_sync_value(conn, "server_url").map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };

    let normalized = normalize_server_url(&server_url);
    if normalized != server_url {
        crate::db::error_repo::set_sync_value(conn, "server_url", &normalized)
            .map_err(|e| e.to_string())?;
    }

    Ok(Some(normalized))
}

/// Convert a relative TTL (seconds from now) into an absolute expiry timestamp
/// stored as an RFC3339 string. Refreshed alongside the tokens on every login
/// and every successful token refresh.
fn absolute_expiry(expires_in: i64) -> String {
    let mut expiry = Utc::now() + chrono::Duration::seconds(expires_in);
    // Refresh a little before the real expiry to avoid edge races.
    expiry = expiry - chrono::Duration::seconds(30);
    expiry.to_rfc3339()
}

/// Returns true when the stored access token has not expired yet.
fn is_access_token_valid(conn: &rusqlite::Connection) -> Result<bool, String> {
    let access =
        crate::db::error_repo::get_sync_value(conn, "access_token").map_err(|e| e.to_string())?;
    if access.is_none() {
        return Ok(false);
    }
    let expires_at = crate::db::error_repo::get_sync_value(conn, "access_expires_at")
        .map_err(|e| e.to_string())?;
    let Some(expires_at) = expires_at else {
        // Legacy credentials without an expiry marker: treat as valid so we
        // don't surprise-upgrade users who logged in before this change.
        return Ok(true);
    };
    let Ok(parsed) = DateTime::parse_from_rfc3339(&expires_at) else {
        return Ok(true);
    };
    Ok(parsed.with_timezone(&Utc) > Utc::now())
}

fn build_auth_status(conn: &rusqlite::Connection) -> Result<AuthStatus, String> {
    let server_url = load_normalized_server_url(conn)?;
    let logged_in = is_access_token_valid(conn)?;
    let user = if logged_in {
        crate::db::error_repo::get_sync_value(conn, "mobile_user")
            .map_err(|e| e.to_string())?
            .and_then(|value| serde_json::from_str(&value).ok())
    } else {
        None
    };
    Ok(AuthStatus {
        logged_in,
        server_url,
        user,
    })
}

fn user_id(value: &serde_json::Value) -> Option<&str> {
    value.get("id").and_then(|id| id.as_str())
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
    let previous_server_url = load_normalized_server_url(&conn)?;
    let previous_user_id = crate::db::error_repo::get_sync_value(&conn, "mobile_user")
        .map_err(|e| e.to_string())?
        .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
        .and_then(|value| user_id(&value).map(ToOwned::to_owned));
    let next_user_id = user_id(&login.user).map(ToOwned::to_owned);
    let identity_changed = previous_server_url
        .as_deref()
        .is_some_and(|value| value != server_url)
        || (previous_user_id.is_some() && previous_user_id != next_user_id);

    if identity_changed {
        crate::db::error_repo::purge_remote_cache(&conn).map_err(|e| e.to_string())?;
    } else {
        crate::db::error_repo::delete_sync_value(&conn, "last_error_sync_cursor")
            .map_err(|e| e.to_string())?;
    }

    crate::db::error_repo::set_sync_value(&conn, "server_url", &server_url)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "access_token", &login.access_token)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "refresh_token", &login.refresh_token)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(
        &conn,
        "access_expires_at",
        &absolute_expiry(login.expires_in),
    )
    .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "mobile_user", &login.user.to_string())
        .map_err(|e| e.to_string())?;

    Ok(AuthStatus {
        logged_in: true,
        server_url: Some(server_url),
        user: Some(login.user),
    })
}

/// Exchanges the stored refresh token for a fresh access/refresh token pair.
/// On any failure (missing token, network error, 401) the local credentials are
/// cleared so the UI falls back to the login screen.
#[tauri::command]
pub async fn refresh_access_token(
    state: State<'_, DbState>,
    client_state: State<'_, HttpClientState>,
) -> Result<AuthStatus, String> {
    refresh_access_token_inner(&state, &client_state.client).await?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    build_auth_status(&conn)
}

/// Shared refresh implementation. Persists the new tokens and returns the
/// server URL plus the refreshed access token so callers can immediately retry
/// the request that triggered the refresh.
pub async fn refresh_access_token_inner(
    state: &State<'_, DbState>,
    client: &reqwest::Client,
) -> Result<(String, String), String> {
    let (server_url, refresh_token) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let server_url = load_normalized_server_url(&conn)?;
        let refresh_token = crate::db::error_repo::get_sync_value(&conn, "refresh_token")
            .map_err(|e| e.to_string())?;
        (server_url, refresh_token)
    };

    let (Some(server_url), Some(refresh_token)) = (server_url, refresh_token) else {
        clear_local_auth(state)?;
        return Err("登录已过期，请重新登录".into());
    };

    let response = client
        .post(format!("{server_url}/api/mobile/auth/refresh"))
        .json(&serde_json::json!({ "refreshToken": refresh_token }))
        .send()
        .await;

    let response = match response {
        Ok(resp) if resp.status().is_success() => resp,
        _ => {
            clear_local_auth(state)?;
            return Err("登录已过期，请重新登录".into());
        }
    };

    let refreshed = response
        .json::<MobileRefreshResponse>()
        .await
        .map_err(|e| {
            let _ = clear_local_auth(state);
            format!("刷新登录失败: {e}")
        })?;

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "access_token", &refreshed.access_token)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(&conn, "refresh_token", &refreshed.refresh_token)
        .map_err(|e| e.to_string())?;
    crate::db::error_repo::set_sync_value(
        &conn,
        "access_expires_at",
        &absolute_expiry(refreshed.expires_in),
    )
    .map_err(|e| e.to_string())?;

    Ok((server_url, refreshed.access_token))
}

fn clear_local_auth(state: &State<'_, DbState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::clear_auth(&conn).map_err(|e| e.to_string())
}

/// Discards the locally stored credentials. The server uses stateless JWTs
/// without a revoke endpoint, so client-side deletion is all logout needs.
#[tauri::command]
pub fn mobile_logout(state: State<'_, DbState>) -> Result<AuthStatus, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::error_repo::clear_auth(&conn).map_err(|e| e.to_string())?;
    build_auth_status(&conn)
}

#[tauri::command]
pub fn get_auth_status(state: State<'_, DbState>) -> Result<AuthStatus, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    build_auth_status(&conn)
}

#[cfg(test)]
mod tests {
    use super::normalize_server_url;

    #[test]
    fn normalize_server_url_prefers_ipv4_loopback_for_localhost() {
        assert_eq!(
            normalize_server_url("http://localhost:3000"),
            "http://127.0.0.1:3000"
        );
        assert_eq!(
            normalize_server_url("http://localhost:3000/api/mobile/auth/login/"),
            "http://127.0.0.1:3000/api/mobile/auth/login"
        );
    }

    #[test]
    fn normalize_server_url_keeps_non_localhost_hosts() {
        assert_eq!(
            normalize_server_url("https://example.com/api"),
            "https://example.com/api"
        );
    }
}
