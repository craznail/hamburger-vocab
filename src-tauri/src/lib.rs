use std::sync::Mutex;
use tauri::Manager;

mod db;
use db::DbState;

// ---- Tauri Commands ----

#[tauri::command]
fn init_db(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let _conn = state.conn.lock().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_decks(state: tauri::State<'_, DbState>) -> Result<Vec<db::Deck>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_decks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_deck(state: tauri::State<'_, DbState>, name: String) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::create_deck(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_deck(state: tauri::State<'_, DbState>, deck_id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_deck(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_deck_by_id(state: tauri::State<'_, DbState>, deck_id: String) -> Result<Option<db::DeckInfo>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_deck_by_id(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_deck_stats(state: tauri::State<'_, DbState>, deck_id: String) -> Result<db::DeckStats, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_deck_stats(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_today_count(state: tauri::State<'_, DbState>) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_today_count(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_cards(
    state: tauri::State<'_, DbState>,
    deck_id: String,
    cards: Vec<db::CardImport>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::import_cards(&conn, &deck_id, &cards).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_cards_by_deck_id(
    state: tauri::State<'_, DbState>,
    deck_id: String,
) -> Result<Vec<db::Card>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_cards_by_deck_id(&conn, &deck_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_today_cards(
    state: tauri::State<'_, DbState>,
    deck_id: Option<String>,
) -> Result<Vec<db::TodayCard>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_today_cards(&conn, deck_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_card_after_review(
    state: tauri::State<'_, DbState>,
    card_id: String,
    ef: f64,
    interval: i64,
    repetitions: i64,
    next_review: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let update = db::ReviewUpdate {
        ef,
        interval,
        repetitions,
        next_review,
    };
    db::update_card_after_review(&conn, &card_id, &update).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_review_log(
    state: tauri::State<'_, DbState>,
    card_id: String,
    quality: i64,
    ef_before: f64,
    ef_after: f64,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::add_review_log(&conn, &card_id, quality, ef_before, ef_after)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_db_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    let db_path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("vocab.db");
    Ok(db_path.to_string_lossy().to_string())
}

#[tauri::command]
fn read_txt_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("无法读取文件: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize database
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("vocab.db");
            log::info!("Database path: {:?}", db_path);

            let conn = db::init_db(&db_path).expect("failed to initialize database");
            app.manage(DbState {
                conn: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_db,
            get_decks,
            create_deck,
            delete_deck,
            get_deck_by_id,
            get_deck_stats,
            get_today_count,
            import_cards,
            get_cards_by_deck_id,
            get_today_cards,
            update_card_after_review,
            add_review_log,
            export_db_path,
            read_txt_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
