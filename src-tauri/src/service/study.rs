use crate::algorithm::sm2;
use crate::db::card_repo;
use crate::db::models::ReviewUpdate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RateResult {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
}

/// Rate a card: compute SM2 schedule, update card, and log the review.
/// This replaces the previous 3-invoke flow (JS SM2 + update_card + add_review_log).
pub fn rate_card(
    conn: &rusqlite::Connection,
    card_id: &str,
    quality: i64,
) -> Result<RateResult, String> {
    // 1. Read current card state
    let card = card_repo::get_card_by_id(conn, card_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Card not found".to_string())?;

    let ef_before = card.ef;

    // 2. Compute new schedule via SM2
    let result = sm2::compute_next_review(quality, card.ef, card.interval, card.repetitions);

    // 3. Update card
    let update = ReviewUpdate {
        ef: result.ef,
        interval: result.interval,
        repetitions: result.repetitions,
        next_review: result.next_review.clone(),
    };
    card_repo::update_card_after_review(conn, card_id, &update)
        .map_err(|e| e.to_string())?;

    // 4. Log the review
    card_repo::add_review_log(conn, card_id, quality, ef_before, result.ef)
        .map_err(|e| e.to_string())?;

    Ok(RateResult {
        ef: result.ef,
        interval: result.interval,
        repetitions: result.repetitions,
        next_review: result.next_review,
    })
}
