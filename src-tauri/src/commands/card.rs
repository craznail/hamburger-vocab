use crate::db::DbState;
use tauri::{Manager, State};

#[tauri::command]
pub fn import_cards(
    state: State<'_, DbState>,
    deck_id: String,
    cards: Vec<crate::db::models::CardImport>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::card_repo::import_cards(&conn, &deck_id, &cards).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_cards_by_deck_id(
    state: State<'_, DbState>,
    deck_id: String,
) -> Result<Vec<crate::db::models::Card>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::card_repo::get_cards_by_deck_id(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_today_cards(
    state: State<'_, DbState>,
    deck_id: Option<String>,
) -> Result<Vec<crate::db::models::TodayCard>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::card_repo::get_today_cards(&conn, deck_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_practice_cards(
    state: State<'_, DbState>,
    deck_id: Option<String>,
) -> Result<Vec<crate::db::models::TodayCard>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::card_repo::get_practice_cards(&conn, deck_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rate_practice_card(
    state: State<'_, DbState>,
    card_id: String,
    quality: i64,
    duration_seconds: Option<i64>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::card_repo::add_practice_log(&conn, &card_id, quality, duration_seconds.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_db_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("vocab.db");
    Ok(db_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_txt_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("无法读取文件: {}", e))
}
