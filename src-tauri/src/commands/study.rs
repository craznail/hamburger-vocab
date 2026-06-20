use crate::db::DbState;
use tauri::State;

#[tauri::command]
pub fn get_learning_stats(
    state: State<'_, DbState>,
) -> Result<crate::db::models::LearningStats, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    crate::db::study_repo::get_learning_stats(&conn).map_err(|e| e.to_string())
}
