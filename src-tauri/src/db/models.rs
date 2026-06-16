use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub word_count: i64,
    pub mastered_count: i64,
    pub due_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct Card {
    pub id: String,
    pub deck_id: String,
    pub word: String,
    pub inflections: String,
    pub definition: String,
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub created_at: String,
    pub last_review_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct TodayCard {
    pub id: String,
    pub word: String,
    pub inflections: String,
    pub definition: String,
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub created_at: String,
    pub last_review_at: Option<String>,
    pub deck_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DeckStats {
    pub total: i64,
    pub mastered: i64,
    pub due: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardImport {
    pub word: String,
    pub inflections: Vec<String>,
    pub definition: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewUpdate {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct DailyActivity {
    pub date: String,
    pub review_count: i64,
    pub study_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct LearningStats {
    pub total_cards: i64,
    pub mastered_cards: i64,
    pub learning_cards: i64,
    pub new_cards: i64,
    pub due_cards: i64,
    pub total_reviews: i64,
    pub total_study_seconds: i64,
    pub accuracy_rate: i64,
    pub streak_days: i64,
    pub longest_streak: i64,
    pub this_week_reviews: i64,
    pub previous_week_reviews: i64,
    pub this_week_seconds: i64,
    pub previous_week_seconds: i64,
    pub daily_activity: Vec<DailyActivity>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ErrorNotebook {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub item_count: i64,
    pub due_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ErrorItem {
    pub id: String,
    pub remote_id: Option<String>,
    pub notebook_id: Option<String>,
    pub notebook_name: Option<String>,
    pub question_text: Option<String>,
    pub answer_text: Option<String>,
    pub analysis: Option<String>,
    pub wrong_answer_text: Option<String>,
    pub mistake_analysis: Option<String>,
    pub mistake_status: Option<String>,
    pub knowledge_points: String,
    pub user_notes: Option<String>,
    pub mastery_level: i64,
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub sync_status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub local_image_path: Option<String>,
    pub remote_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ErrorDraft {
    pub id: String,
    pub local_image_path: String,
    pub sha256: String,
    pub mime_type: String,
    pub sync_status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ErrorReviewResult {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub mastery_level: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeErrorResponse {
    pub remote_id: String,
    pub version: i64,
    pub question_text: Option<String>,
    pub answer_text: Option<String>,
    pub analysis: Option<String>,
    pub wrong_answer_text: Option<String>,
    pub mistake_analysis: Option<String>,
    pub mistake_status: Option<String>,
    pub knowledge_points: Vec<String>,
    pub mastery_level: Option<i64>,
    pub image: Option<RemoteErrorImage>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeErrorFailureResponse {
    pub code: Option<String>,
    pub message: Option<String>,
    pub remote_id: Option<String>,
    pub version: Option<i64>,
    pub image: Option<RemoteErrorImage>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteErrorImage {
    pub remote_key: Option<String>,
    pub url: Option<String>,
    pub sha256: Option<String>,
}
