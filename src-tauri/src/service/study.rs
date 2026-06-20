use crate::algorithm::sm2;
use crate::db::card_repo;
use crate::db::models::ReviewUpdate;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct RateResult {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
}

/// Rate a card: compute SM2 schedule, update card, and log the review.
/// Executes in a transaction to ensure atomicity.
pub fn rate_card(
    conn: &rusqlite::Connection,
    card_id: &str,
    quality: i64,
    duration_seconds: i64,
) -> AppResult<RateResult> {
    // 1. Read current card state
    let card = card_repo::get_card_by_id(conn, card_id)?
        .ok_or_else(|| AppError::NotFound(format!("卡片 {card_id} 不存在")))?;

    let ef_before = card.ef;

    // 2. Compute new schedule via SM2
    let result = sm2::compute_next_review(quality, card.ef, card.interval, card.repetitions);

    // 3. Execute update + log in a transaction
    conn.execute_batch("BEGIN IMMEDIATE")?;

    let exec_result = (|| -> Result<(), rusqlite::Error> {
        let update = ReviewUpdate {
            ef: result.ef,
            interval: result.interval,
            repetitions: result.repetitions,
            next_review: result.next_review.clone(),
        };
        card_repo::update_card_after_review(conn, card_id, &update)?;
        card_repo::add_review_log(
            conn,
            card_id,
            quality,
            ef_before,
            result.ef,
            duration_seconds,
        )?;
        Ok(())
    })();

    match exec_result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e.into());
        }
    }

    Ok(RateResult {
        ef: result.ef,
        interval: result.interval,
        repetitions: result.repetitions,
        next_review: result.next_review,
    })
}
