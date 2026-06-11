use crate::db::DbState;
use tauri::State;

#[tauri::command]
pub fn init_db(state: State<'_, DbState>) -> Result<(), String> {
    let _conn = state.conn.lock().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_decks(state: State<'_, DbState>) -> Result<Vec<crate::db::models::Deck>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::get_decks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_deck(state: State<'_, DbState>, name: String) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::create_deck(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_deck(state: State<'_, DbState>, deck_id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::delete_deck(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_deck_by_id(
    state: State<'_, DbState>,
    deck_id: String,
) -> Result<Option<crate::db::models::DeckInfo>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::get_deck_by_id(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_deck_stats(
    state: State<'_, DbState>,
    deck_id: String,
) -> Result<crate::db::models::DeckStats, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::get_deck_stats(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_today_count(state: State<'_, DbState>) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::deck_repo::get_today_count(&conn).map_err(|e| e.to_string())
}
