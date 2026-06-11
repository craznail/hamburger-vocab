use crate::db::DbState;
use crate::service;
use tauri::State;

/// Integrated rate_card command: SM2 + update + review log in one invoke.
#[tauri::command]
pub fn rate_card(
    state: State<'_, DbState>,
    card_id: String,
    quality: i64,
) -> Result<service::study::RateResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    service::study::rate_card(&conn, &card_id, quality)
}

/// Parse text content (format detection + parsing + validation).
#[tauri::command]
pub fn parse_txt_content(text: String) -> Result<service::import::ParseResult, String> {
    Ok(service::import::parse_txt_content(&text))
}

/// Import from raw text: parse + create deck + import cards, all in one operation.
#[tauri::command]
pub fn import_from_text(
    state: State<'_, DbState>,
    deck_name: String,
    text: String,
) -> Result<service::import::ImportFromTextResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    service::import::import_from_text(&conn, &deck_name, &text)
}
