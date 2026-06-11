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
