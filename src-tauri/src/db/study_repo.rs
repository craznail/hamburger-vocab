use std::collections::{HashMap, HashSet};

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection};

use super::models::{DailyActivity, LearningStats};
use super::today_str;

pub fn get_learning_stats(conn: &Connection) -> Result<LearningStats, rusqlite::Error> {
    let today = Local::now().date_naive();
    let today_text = today_str();
    let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let previous_week_start = week_start - Duration::days(7);
    let next_week_start = week_start + Duration::days(7);

    let (word_total, word_mastered, word_learning, word_new, word_due) = conn.query_row(
        "SELECT
                COUNT(*),
                SUM(CASE WHEN repetitions >= 2 THEN 1 ELSE 0 END),
                SUM(CASE WHEN repetitions = 1 THEN 1 ELSE 0 END),
                SUM(CASE WHEN repetitions = 0 THEN 1 ELSE 0 END),
                SUM(CASE WHEN next_review <= ?1 THEN 1 ELSE 0 END)
             FROM cards",
        params![today_text],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            ))
        },
    )?;

    let (error_total, error_mastered, error_learning, error_new, error_due) = conn.query_row(
        "SELECT
                COUNT(*),
                SUM(CASE WHEN repetitions >= 2 OR mastery_level >= 2 THEN 1 ELSE 0 END),
                SUM(CASE WHEN repetitions = 1 OR mastery_level = 1 THEN 1 ELSE 0 END),
                SUM(CASE WHEN repetitions = 0 AND mastery_level = 0 THEN 1 ELSE 0 END),
                SUM(CASE WHEN next_review <= ?1
                           AND analysis_status = 'ready'
                         THEN 1 ELSE 0 END)
             FROM error_items
             WHERE deleted_at IS NULL",
        params![today_text],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            ))
        },
    )?;

    let total_cards = word_total + error_total;
    let mastered_cards = word_mastered + error_mastered;
    let learning_cards = word_learning + error_learning;
    let new_cards = word_new + error_new;
    let due_cards = word_due + error_due;

    let (word_reviews, word_seconds, word_mastered_reviews) = conn.query_row(
        "SELECT
            COUNT(*),
            COALESCE(SUM(duration_seconds), 0),
            COALESCE(SUM(CASE WHEN quality = 5 THEN 1 ELSE 0 END), 0)
         FROM review_logs",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    )?;
    let (error_reviews, error_seconds, error_mastered_reviews) = conn.query_row(
        "SELECT
            COUNT(*),
            COALESCE(SUM(duration_seconds), 0),
            COALESCE(SUM(CASE WHEN quality = 5 THEN 1 ELSE 0 END), 0)
         FROM error_review_logs",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        },
    )?;
    let total_reviews = word_reviews + error_reviews;
    let total_study_seconds = word_seconds + error_seconds;
    let mastered_reviews = word_mastered_reviews + error_mastered_reviews;
    let accuracy_rate = if total_reviews == 0 {
        0
    } else {
        (mastered_reviews * 100 + total_reviews / 2) / total_reviews
    };

    let (this_week_reviews, this_week_seconds) =
        period_totals(conn, &week_start.to_string(), &next_week_start.to_string())?;
    let (previous_week_reviews, previous_week_seconds) = period_totals(
        conn,
        &previous_week_start.to_string(),
        &week_start.to_string(),
    )?;

    let daily_activity = recent_activity(conn, today)?;
    let active_dates = active_dates(conn)?;
    let streak_days = current_streak(&active_dates, today);
    let longest_streak = longest_streak(&active_dates);

    Ok(LearningStats {
        total_cards,
        mastered_cards,
        learning_cards,
        new_cards,
        due_cards,
        total_reviews,
        total_study_seconds,
        accuracy_rate,
        streak_days,
        longest_streak,
        this_week_reviews,
        previous_week_reviews,
        this_week_seconds,
        previous_week_seconds,
        daily_activity,
    })
}

fn period_totals(conn: &Connection, start: &str, end: &str) -> Result<(i64, i64), rusqlite::Error> {
    conn.query_row(
        "SELECT COALESCE(SUM(review_count), 0), COALESCE(SUM(study_seconds), 0)
         FROM (
            SELECT COUNT(*) AS review_count, COALESCE(SUM(duration_seconds), 0) AS study_seconds
            FROM review_logs
            WHERE date(reviewed_at) >= ?1 AND date(reviewed_at) < ?2
            UNION ALL
            SELECT COUNT(*) AS review_count, COALESCE(SUM(duration_seconds), 0) AS study_seconds
            FROM error_review_logs
            WHERE date(reviewed_at) >= ?1 AND date(reviewed_at) < ?2
         )",
        params![start, end],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

fn recent_activity(
    conn: &Connection,
    today: NaiveDate,
) -> Result<Vec<DailyActivity>, rusqlite::Error> {
    let start = today - Duration::days(6);
    let mut stmt = conn.prepare(
        "SELECT date, SUM(review_count), SUM(study_seconds)
         FROM (
            SELECT date(reviewed_at) AS date, COUNT(*) AS review_count, COALESCE(SUM(duration_seconds), 0) AS study_seconds
            FROM review_logs
            WHERE date(reviewed_at) >= ?1 AND date(reviewed_at) <= ?2
            GROUP BY date(reviewed_at)
            UNION ALL
            SELECT date(reviewed_at) AS date, COUNT(*) AS review_count, COALESCE(SUM(duration_seconds), 0) AS study_seconds
            FROM error_review_logs
            WHERE date(reviewed_at) >= ?1 AND date(reviewed_at) <= ?2
            GROUP BY date(reviewed_at)
         )
         GROUP BY date",
    )?;
    let rows = stmt.query_map(params![start.to_string(), today.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut values = HashMap::new();
    for row in rows {
        let (date, count, seconds) = row?;
        values.insert(date, (count, seconds));
    }

    Ok((0..7)
        .map(|offset| {
            let date = start + Duration::days(offset);
            let date_text = date.to_string();
            let (review_count, study_seconds) = values.get(&date_text).copied().unwrap_or((0, 0));
            DailyActivity {
                date: date_text,
                review_count,
                study_seconds,
            }
        })
        .collect())
}

fn active_dates(conn: &Connection) -> Result<Vec<NaiveDate>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date
         FROM (
            SELECT date(reviewed_at) AS date FROM review_logs
            UNION ALL
            SELECT date(reviewed_at) AS date FROM error_review_logs
         )
         ORDER BY date",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut dates = Vec::new();
    for row in rows {
        if let Ok(date) = NaiveDate::parse_from_str(&row?, "%Y-%m-%d") {
            dates.push(date);
        }
    }
    Ok(dates)
}

fn current_streak(dates: &[NaiveDate], today: NaiveDate) -> i64 {
    let values: HashSet<NaiveDate> = dates.iter().copied().collect();
    let mut cursor = if values.contains(&today) {
        today
    } else if values.contains(&(today - Duration::days(1))) {
        today - Duration::days(1)
    } else {
        return 0;
    };
    let mut count = 0;
    while values.contains(&cursor) {
        count += 1;
        cursor -= Duration::days(1);
    }
    count
}

fn longest_streak(dates: &[NaiveDate]) -> i64 {
    let mut longest = 0;
    let mut current = 0;
    let mut previous = None;
    for date in dates {
        current = match previous {
            Some(prev) if *date == prev + Duration::days(1) => current + 1,
            Some(prev) if *date == prev => current,
            _ => 1,
        };
        longest = longest.max(current);
        previous = Some(*date);
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migration::run(&conn).unwrap();
        conn
    }

    #[test]
    fn empty_database_returns_zeroed_stats_and_seven_days() {
        let conn = test_db();
        let stats = get_learning_stats(&conn).unwrap();

        assert_eq!(stats.total_cards, 0);
        assert_eq!(stats.total_reviews, 0);
        assert_eq!(stats.streak_days, 0);
        assert_eq!(stats.daily_activity.len(), 7);
    }

    #[test]
    fn aggregates_word_and_error_learning_activity() {
        let conn = test_db();
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);
        let old_day = today - Duration::days(4);

        conn.execute("INSERT INTO decks (id, name) VALUES ('deck', 'Test')", [])
            .unwrap();
        for (id, repetitions, next_review) in [
            ("new", 0, today.to_string()),
            ("learning", 1, (today + Duration::days(1)).to_string()),
            ("mastered", 2, today.to_string()),
        ] {
            conn.execute(
                "INSERT INTO cards (id, deck_id, word, repetitions, next_review)
                 VALUES (?1, 'deck', ?1, ?2, ?3)",
                params![id, repetitions, next_review],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO error_notebooks (id, remote_id, name, created_at, updated_at)
             VALUES ('error-deck', 'error-deck', '错题本', ?1, ?1)",
            params![today.to_string()],
        )
        .unwrap();
        for (id, repetitions, mastery_level, next_review) in [
            ("error-new", 0, 0, today.to_string()),
            (
                "error-learning",
                1,
                1,
                (today + Duration::days(1)).to_string(),
            ),
            ("error-mastered", 2, 2, today.to_string()),
        ] {
            conn.execute(
                "INSERT INTO error_items (id, notebook_id, repetitions, mastery_level, next_review, analysis_status, remote_version)
                 VALUES (?1, 'error-deck', ?2, ?3, ?4, 'ready', 1)",
                params![id, repetitions, mastery_level, next_review],
            )
            .unwrap();
        }
        for (id, date, quality, seconds) in [
            ("r1", old_day, 0, 20),
            ("r2", yesterday, 5, 40),
            ("r3", today, 5, 60),
        ] {
            conn.execute(
                "INSERT INTO review_logs
                    (id, card_id, reviewed_at, quality, ef_before, ef_after, duration_seconds)
                 VALUES (?1, 'new', ?2, ?3, 2.5, 2.5, ?4)",
                params![id, format!("{} 12:00:00", date), quality, seconds],
            )
            .unwrap();
        }
        for (id, date, quality, seconds) in [("er1", yesterday, 5, 80), ("er2", today, 3, 100)] {
            conn.execute(
                "INSERT INTO error_review_logs
                    (id, error_item_id, reviewed_at, quality, duration_seconds, ef_before, ef_after, mastery_level, next_review)
                 VALUES (?1, 'error-new', ?2, ?3, ?4, 2.5, 2.5, 1, ?5)",
                params![id, format!("{} 12:00:00", date), quality, seconds, today.to_string()],
            )
            .unwrap();
        }

        let stats = get_learning_stats(&conn).unwrap();
        assert_eq!(stats.total_cards, 6);
        assert_eq!(stats.mastered_cards, 2);
        assert_eq!(stats.learning_cards, 2);
        assert_eq!(stats.new_cards, 2);
        assert_eq!(stats.due_cards, 4);
        assert_eq!(stats.total_reviews, 5);
        assert_eq!(stats.total_study_seconds, 300);
        assert_eq!(stats.accuracy_rate, 60);
        assert_eq!(stats.streak_days, 2);
        assert_eq!(stats.longest_streak, 2);
    }
}
