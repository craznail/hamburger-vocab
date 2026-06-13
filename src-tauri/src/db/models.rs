use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub word_count: i64,
    pub mastered_count: i64,
    pub due_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    pub date: String,
    pub review_count: i64,
    pub study_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
