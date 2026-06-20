use rusqlite::{params, Connection};

use super::models::*;
use super::{generate_id, now_str, today_str};

fn row_to_error_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ErrorItem> {
    Ok(ErrorItem {
        id: row.get(0)?,
        remote_id: row.get(1)?,
        notebook_id: row.get(2)?,
        notebook_name: row.get(3)?,
        question_text: row.get(4)?,
        answer_text: row.get(5)?,
        analysis: row.get(6)?,
        wrong_answer_text: row.get(7)?,
        mistake_analysis: row.get(8)?,
        mistake_status: row.get(9)?,
        knowledge_points: row.get(10)?,
        user_notes: row.get(11)?,
        mastery_level: row.get(12)?,
        ef: row.get(13)?,
        interval: row.get(14)?,
        repetitions: row.get(15)?,
        next_review: row.get(16)?,
        sync_status: row.get(17)?,
        version: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
        deleted_at: row.get(21)?,
        local_image_path: row.get(22)?,
        remote_image_url: row.get(23)?,
    })
}

const ERROR_ITEM_SELECT: &str = "
    SELECT e.id, e.remote_id, e.notebook_id, n.name AS notebook_name,
           e.question_text, e.answer_text, e.analysis, e.wrong_answer_text,
           e.mistake_analysis, e.mistake_status, e.knowledge_points, e.user_notes,
           e.mastery_level, e.ef, e.interval, e.repetitions, e.next_review,
           e.sync_status, e.version, e.created_at, e.updated_at, e.deleted_at,
           img.local_path, img.remote_url
    FROM error_items e
    LEFT JOIN error_notebooks n ON n.id = e.notebook_id
    LEFT JOIN error_item_images img ON img.error_item_id = e.id
";

pub fn ensure_default_notebook(conn: &Connection) -> Result<String, rusqlite::Error> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM error_notebooks WHERE deleted_at IS NULL ORDER BY created_at ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();
    if let Some(id) = existing {
        return Ok(id);
    }

    let id = generate_id();
    conn.execute(
        "INSERT INTO error_notebooks (id, name, sync_status) VALUES (?1, '默认错题本', 'pending_sync')",
        params![id],
    )?;
    Ok(id)
}

pub fn get_error_notebooks(conn: &Connection) -> Result<Vec<ErrorNotebook>, rusqlite::Error> {
    ensure_default_notebook(conn)?;
    let today = today_str();
    let mut stmt = conn.prepare(
        "SELECT n.id, n.name, n.created_at,
                COUNT(e.id) AS item_count,
                SUM(CASE WHEN e.next_review <= ?1 AND e.deleted_at IS NULL THEN 1 ELSE 0 END) AS due_count
         FROM error_notebooks n
         LEFT JOIN error_items e ON e.notebook_id = n.id AND e.deleted_at IS NULL
         WHERE n.deleted_at IS NULL
         GROUP BY n.id
         ORDER BY n.created_at DESC",
    )?;
    let rows = stmt.query_map(params![today], |row| {
        Ok(ErrorNotebook {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            item_count: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
            due_count: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
        })
    })?;
    rows.collect()
}

pub fn create_error_draft(
    conn: &Connection,
    notebook_id: Option<&str>,
    local_path: &str,
    sha256: &str,
    mime_type: &str,
) -> Result<ErrorDraft, rusqlite::Error> {
    let notebook_id = match notebook_id {
        Some(id) => id.to_string(),
        None => ensure_default_notebook(conn)?,
    };
    let id = generate_id();
    let image_id = generate_id();
    conn.execute(
        "INSERT INTO error_items (id, notebook_id, sync_status, updated_at)
         VALUES (?1, ?2, 'pending_analyze', ?3)",
        params![id, notebook_id, now_str()],
    )?;
    conn.execute(
        "INSERT INTO error_item_images
            (id, error_item_id, local_path, sha256, mime_type, upload_status, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'local', ?6)",
        params![image_id, id, local_path, sha256, mime_type, now_str()],
    )?;

    Ok(ErrorDraft {
        id,
        local_image_path: local_path.to_string(),
        sha256: sha256.to_string(),
        mime_type: mime_type.to_string(),
        sync_status: "pending_analyze".into(),
    })
}

pub fn get_error_items(
    conn: &Connection,
    notebook_id: Option<&str>,
) -> Result<Vec<ErrorItem>, rusqlite::Error> {
    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(id) =
        notebook_id
    {
        (
            format!("{ERROR_ITEM_SELECT} WHERE e.deleted_at IS NULL AND e.notebook_id = ?1 ORDER BY e.updated_at DESC"),
            vec![Box::new(id.to_string())],
        )
    } else {
        (
            format!("{ERROR_ITEM_SELECT} WHERE e.deleted_at IS NULL ORDER BY e.updated_at DESC"),
            vec![],
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), row_to_error_item)?;
    rows.collect()
}

pub fn get_error_item(conn: &Connection, id: &str) -> Result<Option<ErrorItem>, rusqlite::Error> {
    let sql = format!("{ERROR_ITEM_SELECT} WHERE e.id = ?1 LIMIT 1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query_map(params![id], row_to_error_item)?;
    match rows.next() {
        Some(Ok(item)) => Ok(Some(item)),
        _ => Ok(None),
    }
}

pub fn get_due_error_items(conn: &Connection) -> Result<Vec<ErrorItem>, rusqlite::Error> {
    let sql = format!(
        "{ERROR_ITEM_SELECT}
         WHERE e.deleted_at IS NULL
           AND e.sync_status NOT IN ('pending_analyze', 'analyzing')
           AND e.next_review <= ?1
         ORDER BY e.ef ASC, e.next_review ASC
         LIMIT 50"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![today_str()], row_to_error_item)?;
    rows.collect()
}

pub fn apply_analyze_response(
    conn: &Connection,
    id: &str,
    response: &AnalyzeErrorResponse,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items
         SET remote_id = ?1, question_text = ?2, answer_text = ?3, analysis = ?4,
             wrong_answer_text = ?5, mistake_analysis = ?6, mistake_status = ?7,
             knowledge_points = ?8, mastery_level = ?9, sync_status = 'synced',
             version = ?10, updated_at = ?11
         WHERE id = ?12",
        params![
            response.remote_id,
            response.question_text,
            response.answer_text,
            response.analysis,
            response.wrong_answer_text,
            response.mistake_analysis,
            response.mistake_status,
            serde_json::to_string(&response.knowledge_points).unwrap_or_else(|_| "[]".into()),
            response.mastery_level.unwrap_or(0),
            response.version,
            response.updated_at,
            id,
        ],
    )?;

    if let Some(image) = &response.image {
        conn.execute(
            "UPDATE error_item_images
             SET remote_key = COALESCE(?1, remote_key),
                 remote_url = COALESCE(?2, remote_url),
                 upload_status = 'synced',
                 updated_at = ?3
             WHERE error_item_id = ?4",
            params![image.remote_key, image.url, now_str(), id],
        )?;
    }
    Ok(())
}

pub fn mark_analyze_failed(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items SET sync_status = 'analyze_failed', updated_at = ?1 WHERE id = ?2",
        params![now_str(), id],
    )?;
    Ok(())
}

pub fn apply_analyze_failure_response(
    conn: &Connection,
    id: &str,
    response: &AnalyzeErrorFailureResponse,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items
         SET remote_id = COALESCE(?1, remote_id),
             sync_status = 'analyze_failed',
             version = COALESCE(?2, version),
             updated_at = COALESCE(?3, ?4)
         WHERE id = ?5",
        params![
            response.remote_id,
            response.version,
            response.updated_at,
            now_str(),
            id,
        ],
    )?;

    if let Some(image) = &response.image {
        conn.execute(
            "UPDATE error_item_images
             SET remote_key = COALESCE(?1, remote_key),
                 remote_url = COALESCE(?2, remote_url),
                 upload_status = CASE WHEN ?1 IS NOT NULL OR ?2 IS NOT NULL THEN 'synced' ELSE upload_status END,
                 updated_at = ?3
             WHERE error_item_id = ?4",
            params![image.remote_key, image.url, now_str(), id],
        )?;
    }
    Ok(())
}

pub fn update_error_item_text(
    conn: &Connection,
    id: &str,
    question_text: Option<&str>,
    answer_text: Option<&str>,
    analysis: Option<&str>,
    mistake_analysis: Option<&str>,
    user_notes: Option<&str>,
    knowledge_points: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items
         SET question_text = COALESCE(?1, question_text),
             answer_text = COALESCE(?2, answer_text),
             analysis = COALESCE(?3, analysis),
             mistake_analysis = COALESCE(?4, mistake_analysis),
             user_notes = COALESCE(?5, user_notes),
             knowledge_points = COALESCE(?6, knowledge_points),
             sync_status = CASE WHEN remote_id IS NULL THEN sync_status ELSE 'pending_sync' END,
             updated_at = ?7
         WHERE id = ?8",
        params![
            question_text,
            answer_text,
            analysis,
            mistake_analysis,
            user_notes,
            knowledge_points,
            now_str(),
            id,
        ],
    )?;
    Ok(())
}

pub fn rate_error_item(
    conn: &Connection,
    id: &str,
    quality: i64,
    duration_seconds: i64,
) -> Result<ErrorReviewResult, rusqlite::Error> {
    let item = get_error_item(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let result = crate::algorithm::sm2::compute_next_review(
        quality,
        item.ef,
        item.interval,
        item.repetitions,
    );
    let mastery_level = if result.repetitions >= 2 {
        2
    } else if result.repetitions == 1 {
        1
    } else {
        0
    };
    conn.execute(
        "UPDATE error_items
         SET ef = ?1, interval = ?2, repetitions = ?3, next_review = ?4,
             mastery_level = ?5, sync_status = CASE WHEN remote_id IS NULL THEN sync_status ELSE 'pending_sync' END,
             updated_at = ?6
         WHERE id = ?7",
        params![result.ef, result.interval, result.repetitions, result.next_review, mastery_level, now_str(), id],
    )?;
    conn.execute(
        "INSERT INTO error_review_logs
            (id, error_item_id, quality, reviewed_at, duration_seconds, ef_before, ef_after, mastery_level, next_review, sync_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending_sync')",
        params![
            generate_id(),
            id,
            quality,
            now_str(),
            duration_seconds.clamp(0, 300),
            item.ef,
            result.ef,
            mastery_level,
            result.next_review,
        ],
    )?;
    Ok(ErrorReviewResult {
        ef: result.ef,
        interval: result.interval,
        repetitions: result.repetitions,
        next_review: result.next_review,
        mastery_level,
    })
}

pub fn get_sync_value(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    Ok(conn
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok())
}

pub fn set_sync_value(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sync_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn delete_sync_value(conn: &Connection, key: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM sync_state WHERE key = ?1", params![key])?;
    Ok(())
}

/// Drops the locally stored mobile credentials so the client returns to the
/// logged-out state. The server URL and last-sync cursor are preserved so the
/// user does not have to retype the address when reconnecting.
pub fn clear_auth(conn: &Connection) -> Result<(), rusqlite::Error> {
    for key in [
        "access_token",
        "refresh_token",
        "access_expires_at",
        "mobile_user",
    ] {
        delete_sync_value(conn, key)?;
    }
    Ok(())
}

pub fn pending_review_logs_json(
    conn: &Connection,
) -> Result<Vec<serde_json::Value>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT l.id, e.remote_id, l.quality, l.reviewed_at, l.duration_seconds, l.mastery_level, l.next_review
         FROM error_review_logs l
         JOIN error_items e ON e.id = l.error_item_id
         WHERE l.sync_status = 'pending_sync' AND e.remote_id IS NOT NULL",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "localId": row.get::<_, String>(0)?,
            "remoteErrorItemId": row.get::<_, String>(1)?,
            "quality": row.get::<_, i64>(2)?,
            "reviewedAt": row.get::<_, String>(3)?,
            "durationSeconds": row.get::<_, i64>(4)?,
            "masteryLevel": row.get::<_, i64>(5)?,
            "nextReviewAt": row.get::<_, String>(6)?,
        }))
    })?;
    rows.collect()
}

pub fn mark_review_logs_synced(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_review_logs SET sync_status = 'synced' WHERE sync_status = 'pending_sync'",
        [],
    )?;
    Ok(())
}

/// After a successful push, all locally-edited error items that have already
/// been synced to the server (i.e. they carry a remote_id) can be marked as
/// synced. Items without a remote_id stay pending — they must be uploaded
/// through the analyze/upload flow first.
pub fn mark_pushed_error_items_synced(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items SET sync_status = 'synced' WHERE sync_status = 'pending_sync' AND remote_id IS NOT NULL",
        [],
    )?;
    Ok(())
}

pub fn upsert_pulled_error_item(
    conn: &Connection,
    item: &serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let remote_id = item
        .get("remoteId")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if remote_id.is_empty() {
        return Ok(());
    }

    let local_id = conn
        .query_row(
            "SELECT id FROM error_items WHERE remote_id = ?1 LIMIT 1",
            params![remote_id],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .unwrap_or_else(generate_id);
    let notebook_id = ensure_default_notebook(conn)?;
    let knowledge_points = item
        .get("knowledgePoints")
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_else(|| "[]".into());
    let version = item.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
    let updated_at = item
        .get("updatedAt")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(now_str);

    conn.execute(
        "INSERT INTO error_items
            (id, remote_id, notebook_id, question_text, answer_text, analysis, wrong_answer_text,
             mistake_analysis, mistake_status, knowledge_points, mastery_level, sync_status, version, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'synced', ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
             question_text = excluded.question_text,
             answer_text = excluded.answer_text,
             analysis = excluded.analysis,
             wrong_answer_text = excluded.wrong_answer_text,
             mistake_analysis = excluded.mistake_analysis,
             mistake_status = excluded.mistake_status,
             knowledge_points = excluded.knowledge_points,
             mastery_level = excluded.mastery_level,
             sync_status = 'synced',
             version = excluded.version,
             updated_at = excluded.updated_at",
        params![
            local_id,
            remote_id,
            notebook_id,
            item.get("questionText").and_then(|v| v.as_str()),
            item.get("answerText").and_then(|v| v.as_str()),
            item.get("analysis").and_then(|v| v.as_str()),
            item.get("wrongAnswerText").and_then(|v| v.as_str()),
            item.get("mistakeAnalysis").and_then(|v| v.as_str()),
            item.get("mistakeStatus").and_then(|v| v.as_str()),
            knowledge_points,
            item.get("masteryLevel").and_then(|v| v.as_i64()).unwrap_or(0),
            version,
            updated_at,
        ],
    )?;

    if let Some(image) = item.get("image") {
        let remote_url = image.get("url").and_then(|v| v.as_str());
        let remote_key = image.get("remoteKey").and_then(|v| v.as_str());
        let sha256 = image.get("sha256").and_then(|v| v.as_str()).unwrap_or("");
        if remote_url.is_some() || remote_key.is_some() {
            conn.execute(
                "INSERT INTO error_item_images
                    (id, error_item_id, local_path, remote_key, remote_url, sha256, mime_type, upload_status, updated_at)
                 VALUES (?1, ?2, '', ?3, ?4, ?5, 'image/jpeg', 'synced', ?6)
                 ON CONFLICT(error_item_id) DO UPDATE SET
                    remote_key = excluded.remote_key,
                    remote_url = excluded.remote_url,
                    upload_status = 'synced',
                    updated_at = excluded.updated_at",
                params![generate_id(), local_id, remote_key, remote_url, sha256, now_str()],
            )?;
        }
    }

    Ok(())
}

pub fn apply_pulled_deletions(
    conn: &Connection,
    ids: &[serde_json::Value],
) -> Result<(), rusqlite::Error> {
    for id in ids {
        if let Some(remote_id) = id.as_str() {
            conn.execute(
                "UPDATE error_items SET deleted_at = ?1, sync_status = 'deleted' WHERE remote_id = ?2",
                params![now_str(), remote_id],
            )?;
        }
    }
    Ok(())
}
