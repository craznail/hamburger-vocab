use std::sync::Mutex;
use tauri::Manager;

mod algorithm;
mod commands;
mod db;
mod error;
mod http;
mod platform;
mod service;

use db::DbState;
use http::HttpClientState;

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
            app.manage(HttpClientState::default());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Deck commands
            commands::deck::init_db,
            commands::deck::get_decks,
            commands::deck::create_deck,
            commands::deck::delete_deck,
            commands::deck::get_deck_by_id,
            commands::deck::get_deck_stats,
            commands::deck::get_today_count,
            commands::study::get_learning_stats,
            // Card commands
            commands::card::import_cards,
            commands::card::get_cards_by_deck_id,
            commands::card::get_today_cards,
            commands::card::get_practice_cards,
            commands::card::rate_practice_card,
            commands::card::export_db_path,
            commands::card::read_txt_content,
            // Import commands (new integrated commands)
            commands::import::rate_card,
            commands::import::parse_txt_content,
            commands::import::import_from_text,
            // TTS commands
            commands::tts::synthesize_speech,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
