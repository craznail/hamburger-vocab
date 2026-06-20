use chrono::Datelike;
use serde::{Deserialize, Serialize};

const MIN_EF: f64 = 1.3;
const MAX_EF: f64 = 2.5;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sm2Result {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
}

fn add_days(date_str: &str, days: i64) -> String {
    let parts: Vec<&str> = date_str.split('-').collect();
    let year: i32 = parts[0].parse().unwrap_or(2024);
    let month: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let day: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    let new_date = date + chrono::Duration::days(days);
    format!(
        "{}-{:02}-{:02}",
        new_date.year(),
        new_date.month(),
        new_date.day()
    )
}

/// Compute next review schedule based on SM-2 algorithm.
///
/// Quality values: 0 (forgot), 3 (hazy), 5 (mastered)
pub fn compute_next_review(quality: i64, ef: f64, interval: i64, repetitions: i64) -> Sm2Result {
    let mut ef = ef;
    let mut interval = interval;
    let mut repetitions = repetitions;

    if quality == 0 {
        // Forgot: reset
        repetitions = 0;
        interval = 1;
        ef = (MIN_EF).max(ef - 0.2);
    } else if quality == 3 {
        // Hazy: reset repetitions, keep short interval
        repetitions = 0;
        interval = 1;
        ef = (MIN_EF).max(ef - 0.15);
    } else if quality == 5 {
        // Mastered: increase interval
        repetitions += 1;
        if repetitions == 1 {
            interval = 1;
        } else if repetitions == 2 {
            interval = 6;
        } else {
            interval = ((interval as f64) * ef).ceil() as i64;
        }
        ef = MAX_EF.min(ef + 0.1);
    }

    let today = crate::db::today_str();
    let next_review = add_days(&today, interval);

    Sm2Result {
        ef,
        interval,
        repetitions,
        next_review,
    }
}
