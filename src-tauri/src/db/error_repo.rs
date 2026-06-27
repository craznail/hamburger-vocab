use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

use super::models::*;
use super::{generate_id, now_str, today_str};

#[derive(Debug, Clone)]
pub struct PendingSyncOp {
    pub op_id: String,
    pub entity_type: String,
    pub action: String,
    pub local_item_id: String,
    pub remote_item_id: Option<String>,
    pub base_version: Option<i64>,
    pub payload: Value,
    pub client_timestamp: String,
}

fn parse_knowledge_points(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn sync_status_for_item(
    conn: &Connection,
    item_id: &str,
    analysis_status: &str,
) -> Result<String, rusqlite::Error> {
    if analysis_status != "ready" {
        return Ok(analysis_status.to_string());
    }

    let has_conflict: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM error_sync_conflicts WHERE local_item_id = ?1
        )",
        params![item_id],
        |row| row.get(0),
    )?;
    if has_conflict {
        return Ok("conflict".into());
    }

    let has_pending_op: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM error_sync_ops WHERE local_item_id = ?1 AND status = 'pending'
        )",
        params![item_id],
        |row| row.get(0),
    )?;
    if has_pending_op {
        return Ok("pending_sync".into());
    }

    Ok("synced".into())
}

fn is_syncable_analysis_status(analysis_status: &str) -> bool {
    analysis_status == "ready"
}

fn action_requires_syncable_item(action: &str) -> bool {
    matches!(action, "create" | "update")
}

fn has_remote_snapshot(snapshot_text: &str) -> bool {
    let trimmed = snapshot_text.trim();
    !trimmed.is_empty() && trimmed != "null"
}

fn sync_conflict_reason(error_code: Option<&str>, has_snapshot: bool) -> &'static str {
    match error_code.unwrap_or_default() {
        "VALIDATION_ERROR" => "validation_error",
        "NOT_FOUND" => "not_found",
        "VERSION_CONFLICT" => "version_conflict",
        _ if has_snapshot => "version_conflict",
        _ => "unknown",
    }
}

fn row_to_error_item(conn: &Connection, row: &rusqlite::Row<'_>) -> rusqlite::Result<ErrorItem> {
    let id: String = row.get(0)?;
    let analysis_status: String = row.get(17)?;
    let sync_status = sync_status_for_item(conn, &id, &analysis_status)?;
    Ok(ErrorItem {
        id,
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
        sync_status,
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
           e.analysis_status, e.remote_version, e.created_at, e.updated_at, e.deleted_at,
           img.local_path, img.remote_url
    FROM error_items e
    LEFT JOIN error_notebooks n ON n.id = e.notebook_id
    LEFT JOIN error_item_images img ON img.error_item_id = e.id
";

fn get_item_image_metadata(
    conn: &Connection,
    local_item_id: &str,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    rusqlite::Error,
> {
    conn.query_row(
        "SELECT remote_key, remote_url, sha256, mime_type
         FROM error_item_images
         WHERE error_item_id = ?1",
        params![local_item_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .optional()
    .map(|value| value.unwrap_or((None, None, None, None)))
}

fn clear_non_syncable_item_sync_state(
    conn: &Connection,
    local_item_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM error_sync_conflicts WHERE local_item_id = ?1",
        params![local_item_id],
    )?;
    conn.execute(
        "DELETE FROM error_sync_ops
         WHERE local_item_id = ?1
           AND action IN ('create', 'update')",
        params![local_item_id],
    )?;
    Ok(())
}

fn backfill_remote_image_from_snapshot(
    conn: &Connection,
    local_item_id: &str,
) -> Result<bool, rusqlite::Error> {
    let snapshot_text = conn
        .query_row(
            "SELECT server_snapshot_json
             FROM error_items
             WHERE id = ?1",
            params![local_item_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();
    let Some(snapshot_text) = snapshot_text else {
        return Ok(false);
    };
    let snapshot = serde_json::from_str::<Value>(&snapshot_text).unwrap_or_else(|_| json!({}));
    let has_image_payload = snapshot
        .get("image")
        .and_then(|image| image.as_object())
        .is_some_and(|image| {
            image.get("url").and_then(|value| value.as_str()).is_some()
                || image
                    .get("remoteKey")
                    .and_then(|value| value.as_str())
                    .is_some()
                || image
                    .get("sha256")
                    .and_then(|value| value.as_str())
                    .is_some()
        });
    if !has_image_payload {
        return Ok(false);
    }
    apply_remote_image(conn, local_item_id, snapshot.get("image"))?;
    Ok(true)
}

fn ensure_item_image_metadata(
    conn: &Connection,
    local_item_id: &str,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    rusqlite::Error,
> {
    let metadata = get_item_image_metadata(conn, local_item_id)?;
    let needs_backfill = metadata.0.is_none()
        && metadata.1.is_none()
        && metadata.2.is_none()
        && metadata.3.is_none();
    if !needs_backfill {
        return Ok(metadata);
    }
    if backfill_remote_image_from_snapshot(conn, local_item_id)? {
        return get_item_image_metadata(conn, local_item_id);
    }
    Ok(metadata)
}

fn hydrate_remote_images_from_snapshots(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT e.id
         FROM error_items e
         LEFT JOIN error_item_images img ON img.error_item_id = e.id
         WHERE e.deleted_at IS NULL
           AND e.server_snapshot_json IS NOT NULL
           AND (
             img.id IS NULL
             OR (img.remote_url IS NULL AND img.remote_key IS NULL)
           )",
    )?;
    let item_ids = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for item_id in item_ids {
        let _ = backfill_remote_image_from_snapshot(conn, &item_id)?;
    }
    Ok(())
}

fn build_item_payload(conn: &Connection, local_item_id: &str) -> Result<Value, rusqlite::Error> {
    let item = get_error_item(conn, local_item_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let (remote_key, remote_url, sha256, content_type) =
        ensure_item_image_metadata(conn, local_item_id)?;

    Ok(json!({
        "notebookId": item.notebook_id,
        "questionText": item.question_text,
        "answerText": item.answer_text,
        "analysis": item.analysis,
        "wrongAnswerText": item.wrong_answer_text,
        "mistakeAnalysis": item.mistake_analysis,
        "mistakeStatus": item.mistake_status,
        "knowledgePoints": parse_knowledge_points(&item.knowledge_points),
        "userNotes": item.user_notes,
        "masteryLevel": item.mastery_level,
        "ef": item.ef,
        "interval": item.interval,
        "repetitions": item.repetitions,
        "nextReview": item.next_review,
        "image": {
            "remoteKey": remote_key,
            "url": remote_url,
            "sha256": sha256,
            "contentType": content_type,
        }
    }))
}

fn upsert_pending_op(
    conn: &Connection,
    action: &str,
    local_item_id: &str,
    remote_item_id: Option<&str>,
    base_version: Option<i64>,
    payload: &Value,
) -> Result<String, rusqlite::Error> {
    let now = now_str();
    if action == "update" {
        if let Some(op_id) = conn
            .query_row(
                "SELECT op_id FROM error_sync_ops
                 WHERE local_item_id = ?1 AND status = 'pending' AND action = 'create'
                 ORDER BY created_at ASC LIMIT 1",
                params![local_item_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            conn.execute(
                "UPDATE error_sync_ops
                 SET payload_json = ?1, updated_at = ?2
                 WHERE op_id = ?3",
                params![payload.to_string(), now, op_id],
            )?;
            return Ok(op_id);
        }
    }

    if let Some(op_id) = conn
        .query_row(
            "SELECT op_id FROM error_sync_ops
             WHERE local_item_id = ?1 AND status = 'pending' AND action = ?2
             ORDER BY created_at ASC LIMIT 1",
            params![local_item_id, action],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        conn.execute(
            "UPDATE error_sync_ops
             SET remote_item_id = COALESCE(?1, remote_item_id),
                 payload_json = ?2,
                 updated_at = ?3
             WHERE op_id = ?4",
            params![remote_item_id, payload.to_string(), now, op_id],
        )?;
        return Ok(op_id);
    }

    let op_id = generate_id();
    conn.execute(
        "INSERT INTO error_sync_ops
            (op_id, entity_type, action, local_item_id, remote_item_id, base_version, payload_json, client_timestamp, status, created_at, updated_at)
         VALUES (?1, 'error_item', ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?7, ?7)",
        params![
            op_id,
            action,
            local_item_id,
            remote_item_id,
            base_version,
            payload.to_string(),
            now
        ],
    )?;
    Ok(op_id)
}

fn find_local_item_id_for_remote_snapshot(
    conn: &Connection,
    remote_id: &str,
    local_id_hint: Option<&str>,
) -> Result<Option<String>, rusqlite::Error> {
    if let Some(local_id) = conn
        .query_row(
            "SELECT id FROM error_items WHERE remote_id = ?1 LIMIT 1",
            params![remote_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        return Ok(Some(local_id));
    }

    if let Some(local_id_hint) = local_id_hint {
        return conn
            .query_row(
                "SELECT id FROM error_items WHERE id = ?1 LIMIT 1",
                params![local_id_hint],
                |row| row.get::<_, String>(0),
            )
            .optional();
    }

    Ok(None)
}

pub fn find_local_item_id_by_remote_id(
    conn: &Connection,
    remote_id: &str,
    local_id_hint: Option<&str>,
) -> Result<Option<String>, rusqlite::Error> {
    find_local_item_id_for_remote_snapshot(conn, remote_id, local_id_hint)
}

pub fn local_item_has_blocking_sync_state(
    conn: &Connection,
    local_item_id: &str,
) -> Result<bool, rusqlite::Error> {
    conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM error_sync_ops WHERE local_item_id = ?1 AND status IN ('pending', 'conflicted')
            UNION ALL
            SELECT 1 FROM error_sync_conflicts WHERE local_item_id = ?1
        )",
        params![local_item_id],
        |row| row.get(0),
    )
}

fn apply_remote_image(
    conn: &Connection,
    local_item_id: &str,
    image: Option<&Value>,
) -> Result<(), rusqlite::Error> {
    let Some(image) = image else {
        return Ok(());
    };
    let existing_image_id = conn
        .query_row(
            "SELECT id FROM error_item_images WHERE error_item_id = ?1 LIMIT 1",
            params![local_item_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let now = now_str();
    if let Some(image_id) = existing_image_id {
        conn.execute(
            "UPDATE error_item_images
             SET remote_key = COALESCE(?1, remote_key),
                 remote_url = COALESCE(?2, remote_url),
                 sha256 = COALESCE(?3, sha256),
                 mime_type = COALESCE(?4, mime_type),
                 updated_at = ?5
             WHERE id = ?6",
            params![
                image.get("remoteKey").and_then(|v| v.as_str()),
                image.get("url").and_then(|v| v.as_str()),
                image.get("sha256").and_then(|v| v.as_str()),
                image.get("contentType").and_then(|v| v.as_str()),
                now,
                image_id
            ],
        )?;
    } else {
        conn.execute(
            "INSERT INTO error_item_images
                (id, error_item_id, local_path, remote_key, remote_url, sha256, mime_type, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, COALESCE(?6, 'image/jpeg'), ?7, ?7)",
            params![
                generate_id(),
                local_item_id,
                image.get("remoteKey").and_then(|v| v.as_str()),
                image.get("url").and_then(|v| v.as_str()),
                image.get("sha256").and_then(|v| v.as_str()).unwrap_or(""),
                image.get("contentType").and_then(|v| v.as_str()),
                now
            ],
        )?;
    }
    Ok(())
}

fn upsert_remote_item_snapshot_inner(
    conn: &Connection,
    item: &Value,
    overwrite_working_copy: bool,
) -> Result<String, rusqlite::Error> {
    let remote_id = item
        .get("remoteId")
        .and_then(|v| v.as_str())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let local_id_hint = item.get("localId").and_then(|v| v.as_str());
    let local_id = find_local_item_id_for_remote_snapshot(conn, remote_id, local_id_hint)?
        .unwrap_or_else(|| {
            local_id_hint
                .map(ToOwned::to_owned)
                .unwrap_or_else(generate_id)
        });
    let knowledge_points = item
        .get("knowledgePoints")
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_else(|| "[]".into());
    let fallback_updated_at = now_str();
    let updated_at = item
        .get("updatedAt")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_updated_at.as_str())
        .to_string();
    let notebook_id = item
        .get("notebookId")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let remote_version = item.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
    let default_next_review = today_str();

    if overwrite_working_copy {
        conn.execute(
            "INSERT INTO error_items
                (id, remote_id, notebook_id, question_text, answer_text, analysis, wrong_answer_text,
                 mistake_analysis, mistake_status, knowledge_points, user_notes, mastery_level, ef,
                 interval, repetitions, next_review, analysis_status, remote_version, server_snapshot_json,
                 created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 'ready', ?17, ?18, ?19, ?20, NULL)
             ON CONFLICT(id) DO UPDATE SET
                remote_id = excluded.remote_id,
                notebook_id = excluded.notebook_id,
                question_text = excluded.question_text,
                answer_text = excluded.answer_text,
                analysis = excluded.analysis,
                wrong_answer_text = excluded.wrong_answer_text,
                mistake_analysis = excluded.mistake_analysis,
                mistake_status = excluded.mistake_status,
                knowledge_points = excluded.knowledge_points,
                user_notes = excluded.user_notes,
                mastery_level = excluded.mastery_level,
                ef = excluded.ef,
                interval = excluded.interval,
                repetitions = excluded.repetitions,
                next_review = excluded.next_review,
                analysis_status = 'ready',
                remote_version = excluded.remote_version,
                server_snapshot_json = excluded.server_snapshot_json,
                updated_at = excluded.updated_at,
                deleted_at = NULL",
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
                item.get("userNotes").and_then(|v| v.as_str()),
                item.get("masteryLevel").and_then(|v| v.as_i64()).unwrap_or(0),
                item.get("ef").and_then(|v| v.as_f64()).unwrap_or(2.5),
                item.get("interval").and_then(|v| v.as_i64()).unwrap_or(1),
                item.get("repetitions").and_then(|v| v.as_i64()).unwrap_or(0),
                item.get("nextReview").and_then(|v| v.as_str()).unwrap_or(default_next_review.as_str()),
                remote_version,
                item.to_string(),
                item.get("createdAt").and_then(|v| v.as_str()).unwrap_or_else(|| updated_at.as_str()),
                updated_at
            ],
        )?;
    } else {
        conn.execute(
            "INSERT INTO error_items
                (id, remote_id, notebook_id, analysis_status, remote_version, server_snapshot_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'ready', ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                remote_id = excluded.remote_id,
                notebook_id = COALESCE(error_items.notebook_id, excluded.notebook_id),
                analysis_status = 'ready',
                remote_version = excluded.remote_version,
                server_snapshot_json = excluded.server_snapshot_json,
                updated_at = excluded.updated_at",
            params![
                local_id,
                remote_id,
                notebook_id,
                remote_version,
                item.to_string(),
                item.get("createdAt").and_then(|v| v.as_str()).unwrap_or_else(|| updated_at.as_str()),
                updated_at
            ],
        )?;
    }

    apply_remote_image(conn, &local_id, item.get("image"))?;
    Ok(local_id)
}

pub fn ensure_device_id(conn: &Connection) -> Result<String, rusqlite::Error> {
    if let Some(id) = get_sync_value(conn, "device_id")? {
        return Ok(id);
    }
    let id = generate_id();
    set_sync_value(conn, "device_id", &id)?;
    Ok(id)
}

pub fn get_error_notebooks(conn: &Connection) -> Result<Vec<ErrorNotebook>, rusqlite::Error> {
    let today = today_str();
    let mut stmt = conn.prepare(
        "SELECT n.id, n.name, n.created_at,
                COUNT(e.id) AS item_count,
                SUM(CASE
                        WHEN e.deleted_at IS NULL
                         AND e.analysis_status = 'ready'
                         AND e.next_review <= ?1
                         AND e.remote_id IS NOT NULL
                        THEN 1 ELSE 0 END
                ) AS due_count
         FROM error_notebooks n
         LEFT JOIN error_items e ON e.notebook_id = n.id AND e.deleted_at IS NULL
         GROUP BY n.id
         ORDER BY n.created_at ASC",
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

pub fn upsert_pulled_notebook(conn: &Connection, notebook: &Value) -> Result<(), rusqlite::Error> {
    let remote_id = notebook
        .get("remoteId")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if remote_id.is_empty() {
        return Ok(());
    }

    let created_at = notebook
        .get("createdAt")
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| {
            notebook
                .get("updatedAt")
                .and_then(|value| value.as_str())
                .unwrap_or("")
        });
    let created_at = if created_at.is_empty() {
        now_str()
    } else {
        created_at.to_string()
    };
    let updated_at = notebook
        .get("updatedAt")
        .and_then(|value| value.as_str())
        .unwrap_or(created_at.as_str())
        .to_string();

    conn.execute(
        "INSERT INTO error_notebooks (id, remote_id, name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(remote_id) DO UPDATE SET
            name = excluded.name,
            updated_at = excluded.updated_at",
        params![
            remote_id,
            remote_id,
            notebook
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("未命名错题本"),
            created_at,
            updated_at
        ],
    )?;
    Ok(())
}

pub fn replace_notebooks(conn: &Connection, notebooks: &[Value]) -> Result<(), rusqlite::Error> {
    let remote_ids: Vec<String> = notebooks
        .iter()
        .filter_map(|notebook| {
            notebook
                .get("remoteId")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
        })
        .collect();
    for notebook in notebooks {
        upsert_pulled_notebook(conn, notebook)?;
    }
    if remote_ids.is_empty() {
        conn.execute("DELETE FROM error_notebooks", [])?;
    } else {
        let placeholders = remote_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM error_notebooks WHERE remote_id NOT IN ({placeholders})");
        let params = rusqlite::params_from_iter(remote_ids.iter());
        conn.execute(&sql, params)?;
    }
    Ok(())
}

pub fn create_error_draft(
    conn: &Connection,
    notebook_id: Option<&str>,
    local_path: &str,
    sha256: &str,
    mime_type: &str,
) -> Result<ErrorDraft, rusqlite::Error> {
    let notebook_id = notebook_id.ok_or(rusqlite::Error::InvalidQuery)?;
    let notebook_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM error_notebooks WHERE id = ?1)",
        params![notebook_id],
        |row| row.get(0),
    )?;
    if !notebook_exists {
        return Err(rusqlite::Error::InvalidQuery);
    }

    let id = generate_id();
    let image_id = generate_id();
    let now = now_str();
    conn.execute(
        "INSERT INTO error_items (id, notebook_id, analysis_status, created_at, updated_at)
         VALUES (?1, ?2, 'pending_analysis', ?3, ?3)",
        params![id, notebook_id, now],
    )?;
    conn.execute(
        "INSERT INTO error_item_images
            (id, error_item_id, local_path, sha256, mime_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![image_id, id, local_path, sha256, mime_type, now],
    )?;

    Ok(ErrorDraft {
        id,
        local_image_path: local_path.to_string(),
        sha256: sha256.to_string(),
        mime_type: mime_type.to_string(),
        sync_status: "pending_analysis".into(),
    })
}

pub fn get_error_items(
    conn: &Connection,
    notebook_id: Option<&str>,
) -> Result<Vec<ErrorItem>, rusqlite::Error> {
    hydrate_remote_images_from_snapshots(conn)?;
    let sql = if notebook_id.is_some() {
        format!("{ERROR_ITEM_SELECT} WHERE e.deleted_at IS NULL AND e.notebook_id = ?1 ORDER BY e.updated_at DESC")
    } else {
        format!("{ERROR_ITEM_SELECT} WHERE e.deleted_at IS NULL ORDER BY e.updated_at DESC")
    };
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = if let Some(notebook_id) = notebook_id {
        stmt.query(params![notebook_id])?
    } else {
        stmt.query([])?
    };
    let mut items = Vec::new();
    while let Some(row) = rows.next()? {
        items.push(row_to_error_item(conn, row)?);
    }
    Ok(items)
}

pub fn get_error_item(conn: &Connection, id: &str) -> Result<Option<ErrorItem>, rusqlite::Error> {
    let _ = backfill_remote_image_from_snapshot(conn, id)?;
    let sql = format!("{ERROR_ITEM_SELECT} WHERE e.id = ?1 LIMIT 1");
    let mut stmt = conn.prepare(&sql)?;
    stmt.query_row(params![id], |row| row_to_error_item(conn, row))
        .optional()
}

pub fn get_due_error_items(conn: &Connection) -> Result<Vec<ErrorItem>, rusqlite::Error> {
    hydrate_remote_images_from_snapshots(conn)?;
    let sql = format!(
        "{ERROR_ITEM_SELECT}
         WHERE e.deleted_at IS NULL
           AND e.analysis_status = 'ready'
           AND e.remote_id IS NOT NULL
           AND e.next_review <= ?1
           AND NOT EXISTS (
             SELECT 1 FROM error_sync_ops op
             WHERE op.local_item_id = e.id AND op.status IN ('pending', 'conflicted')
           )
           AND NOT EXISTS (
             SELECT 1 FROM error_sync_conflicts conflict
             WHERE conflict.local_item_id = e.id
           )
         ORDER BY e.next_review ASC, e.updated_at DESC
         LIMIT 50"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![today_str()])?;
    let mut items = Vec::new();
    while let Some(row) = rows.next()? {
        items.push(row_to_error_item(conn, row)?);
    }
    Ok(items)
}

pub fn mark_item_analyzing(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items
         SET analysis_status = 'analyzing', updated_at = ?1
         WHERE id = ?2",
        params![now_str(), id],
    )?;
    Ok(())
}

pub fn apply_analyze_response(
    conn: &Connection,
    id: &str,
    response: &AnalyzeErrorResponse,
) -> Result<(), rusqlite::Error> {
    let now = now_str();
    conn.execute(
        "UPDATE error_items
         SET question_text = ?1,
             answer_text = ?2,
             analysis = ?3,
             wrong_answer_text = ?4,
             mistake_analysis = ?5,
             mistake_status = ?6,
             knowledge_points = ?7,
             mastery_level = ?8,
             analysis_status = 'ready',
             updated_at = ?9
         WHERE id = ?10",
        params![
            response.question_text,
            response.answer_text,
            response.analysis,
            response.wrong_answer_text,
            response.mistake_analysis,
            response.mistake_status,
            serde_json::to_string(&response.knowledge_points).unwrap_or_else(|_| "[]".into()),
            response.mastery_level.unwrap_or(0),
            now,
            id
        ],
    )?;

    if let Some(image) = &response.image {
        conn.execute(
            "UPDATE error_item_images
             SET remote_key = COALESCE(?1, remote_key),
                 remote_url = COALESCE(?2, remote_url),
                 sha256 = COALESCE(?3, sha256),
                 mime_type = COALESCE(?4, mime_type),
                 updated_at = ?5
             WHERE error_item_id = ?6",
            params![
                image.remote_key,
                image.url,
                image.sha256,
                image.content_type,
                now,
                id
            ],
        )?;
    }

    let payload = build_item_payload(conn, id)?;
    upsert_pending_op(conn, "create", id, None, None, &payload)?;
    Ok(())
}

pub fn mark_analyze_failed(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE error_items
         SET analysis_status = 'analyze_failed',
             updated_at = ?1
         WHERE id = ?2",
        params![now_str(), id],
    )?;
    clear_non_syncable_item_sync_state(conn, id)?;
    Ok(())
}

pub fn apply_analyze_failure_response(
    conn: &Connection,
    id: &str,
    response: &AnalyzeErrorFailureResponse,
) -> Result<(), rusqlite::Error> {
    mark_analyze_failed(conn, id)?;
    if let Some(image) = &response.image {
        conn.execute(
            "UPDATE error_item_images
             SET remote_key = COALESCE(?1, remote_key),
                 remote_url = COALESCE(?2, remote_url),
                 sha256 = COALESCE(?3, sha256),
                 mime_type = COALESCE(?4, mime_type),
                 updated_at = ?5
             WHERE error_item_id = ?6",
            params![
                image.remote_key,
                image.url,
                image.sha256,
                image.content_type,
                now_str(),
                id
            ],
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
            id
        ],
    )?;

    let remote = conn.query_row(
        "SELECT remote_id, remote_version, analysis_status FROM error_items WHERE id = ?1",
        params![id],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    )?;
    if !is_syncable_analysis_status(&remote.2) {
        clear_non_syncable_item_sync_state(conn, id)?;
        return Ok(());
    }
    let payload = build_item_payload(conn, id)?;
    if remote.0.is_some() {
        upsert_pending_op(
            conn,
            "update",
            id,
            remote.0.as_deref(),
            Some(remote.1),
            &payload,
        )?;
    } else {
        upsert_pending_op(conn, "create", id, None, None, &payload)?;
    }
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
    let now = now_str();
    conn.execute(
        "UPDATE error_items
         SET ef = ?1,
             interval = ?2,
             repetitions = ?3,
             next_review = ?4,
             mastery_level = ?5,
             updated_at = ?6
         WHERE id = ?7",
        params![
            result.ef,
            result.interval,
            result.repetitions,
            result.next_review,
            mastery_level,
            now,
            id
        ],
    )?;
    conn.execute(
        "INSERT INTO error_review_logs
            (id, error_item_id, quality, reviewed_at, duration_seconds, ef_before, ef_after, mastery_level, next_review, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?4)",
        params![
            generate_id(),
            id,
            quality,
            now,
            duration_seconds.clamp(0, 300),
            item.ef,
            result.ef,
            mastery_level,
            result.next_review
        ],
    )?;

    let remote = conn.query_row(
        "SELECT remote_id, remote_version FROM error_items WHERE id = ?1",
        params![id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?)),
    )?;
    if let Some(remote_id) = remote.0 {
        let payload = json!({
            "quality": quality,
            "reviewedAt": now,
            "durationSeconds": duration_seconds.clamp(0, 300),
            "ef": result.ef,
            "interval": result.interval,
            "repetitions": result.repetitions,
            "masteryLevel": mastery_level,
            "nextReview": result.next_review,
        });
        upsert_pending_op(
            conn,
            "review",
            id,
            Some(remote_id.as_str()),
            Some(remote.1),
            &payload,
        )?;
    }

    Ok(ErrorReviewResult {
        ef: result.ef,
        interval: result.interval,
        repetitions: result.repetitions,
        next_review: result.next_review,
        mastery_level,
    })
}

pub fn get_pending_sync_ops(conn: &Connection) -> Result<Vec<PendingSyncOp>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT op_id, entity_type, action, local_item_id, remote_item_id, base_version, payload_json, client_timestamp
         FROM error_sync_ops
         WHERE status = 'pending'
         ORDER BY client_timestamp ASC, created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let payload_text: String = row.get(6)?;
        Ok(PendingSyncOp {
            op_id: row.get(0)?,
            entity_type: row.get(1)?,
            action: row.get(2)?,
            local_item_id: row.get(3)?,
            remote_item_id: row.get(4)?,
            base_version: row.get(5)?,
            payload: serde_json::from_str(&payload_text).unwrap_or_else(|_| json!({})),
            client_timestamp: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_syncable_pending_ops(conn: &Connection) -> Result<Vec<PendingSyncOp>, rusqlite::Error> {
    let ops = get_pending_sync_ops(conn)?;
    let mut syncable = Vec::with_capacity(ops.len());
    for op in ops {
        if action_requires_syncable_item(&op.action) {
            let analysis_status = conn
                .query_row(
                    "SELECT analysis_status FROM error_items WHERE id = ?1",
                    params![op.local_item_id.as_str()],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            if analysis_status.as_deref() != Some("ready") {
                continue;
            }
        }
        syncable.push(op);
    }
    Ok(syncable)
}

pub fn acknowledge_accepted_op(conn: &Connection, accepted: &Value) -> Result<(), rusqlite::Error> {
    let op_id = accepted
        .get("opId")
        .and_then(|value| value.as_str())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let local_item_id = accepted
        .get("localItemId")
        .and_then(|value| value.as_str())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let action = accepted
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    if action == "delete" {
        conn.execute(
            "UPDATE error_items
             SET deleted_at = COALESCE(deleted_at, ?1),
                 updated_at = ?1
             WHERE id = ?2",
            params![now_str(), local_item_id],
        )?;
    } else if let Some(server_snapshot) = accepted.get("serverSnapshot") {
        upsert_remote_item_snapshot_inner(conn, server_snapshot, true)?;
    }

    conn.execute(
        "DELETE FROM error_sync_conflicts WHERE local_item_id = ?1",
        params![local_item_id],
    )?;
    conn.execute(
        "DELETE FROM error_sync_ops WHERE op_id = ?1",
        params![op_id],
    )?;
    Ok(())
}

pub fn record_sync_conflict(conn: &Connection, conflict: &Value) -> Result<(), rusqlite::Error> {
    let op_id = conflict
        .get("opId")
        .and_then(|value| value.as_str())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let local_item_id = conflict
        .get("localItemId")
        .and_then(|value| value.as_str())
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let server_snapshot = conflict
        .get("serverSnapshot")
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let server_version = conflict
        .get("serverVersion")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let error_code = conflict.get("code").and_then(|value| value.as_str());

    conn.execute(
        "UPDATE error_sync_ops
         SET status = 'conflicted', updated_at = ?1
         WHERE local_item_id = ?2 AND status = 'pending'",
        params![now_str(), local_item_id],
    )?;
    conn.execute(
        "INSERT INTO error_sync_conflicts
            (id, local_item_id, op_id, server_version, server_snapshot_json, error_code, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(local_item_id) DO UPDATE SET
            op_id = excluded.op_id,
            server_version = excluded.server_version,
            server_snapshot_json = excluded.server_snapshot_json,
            error_code = excluded.error_code,
            updated_at = excluded.updated_at",
        params![
            generate_id(),
            local_item_id,
            op_id,
            server_version,
            server_snapshot.to_string(),
            error_code,
            now_str()
        ],
    )?;
    Ok(())
}

pub fn upsert_remote_item_snapshot(
    conn: &Connection,
    item: &Value,
    overwrite_working_copy: bool,
) -> Result<String, rusqlite::Error> {
    upsert_remote_item_snapshot_inner(conn, item, overwrite_working_copy)
}

pub fn apply_pulled_deletions(
    conn: &Connection,
    deleted_ids: &[Value],
) -> Result<(), rusqlite::Error> {
    for value in deleted_ids {
        let Some(remote_id) = value.as_str() else {
            continue;
        };
        let local_item = conn
            .query_row(
                "SELECT id FROM error_items WHERE remote_id = ?1 LIMIT 1",
                params![remote_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        let Some(local_item_id) = local_item else {
            continue;
        };
        let has_blocking_local_state: bool = conn.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM error_sync_ops WHERE local_item_id = ?1 AND status IN ('pending', 'conflicted')
                UNION ALL
                SELECT 1 FROM error_sync_conflicts WHERE local_item_id = ?1
            )",
            params![local_item_id],
            |row| row.get(0),
        )?;
        if has_blocking_local_state {
            continue;
        }
        conn.execute(
            "UPDATE error_items
             SET deleted_at = ?1,
                 updated_at = ?1
             WHERE id = ?2",
            params![now_str(), local_item_id],
        )?;
    }
    Ok(())
}

pub fn list_error_sync_conflicts(
    conn: &Connection,
) -> Result<Vec<ErrorSyncConflict>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.local_item_id, e.remote_id, c.server_version, c.error_code, c.server_snapshot_json, c.created_at
         FROM error_sync_conflicts c
         JOIN error_items e ON e.id = c.local_item_id
         ORDER BY c.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let snapshot_text = row
            .get::<_, Option<String>>(5)?
            .unwrap_or_else(|| "null".into());
        let has_snapshot = has_remote_snapshot(&snapshot_text);
        let error_code: Option<String> = row.get(4)?;
        Ok(ErrorSyncConflict {
            id: row.get(0)?,
            local_item_id: row.get(1)?,
            remote_id: row.get(2)?,
            server_version: row.get(3)?,
            reason: sync_conflict_reason(error_code.as_deref(), has_snapshot).into(),
            has_remote_snapshot: has_snapshot,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn resolve_conflict_keep_local(
    conn: &Connection,
    local_item_id: &str,
) -> Result<(), rusqlite::Error> {
    let conflict = conn
        .query_row(
            "SELECT server_version, server_snapshot_json
             FROM error_sync_conflicts
             WHERE local_item_id = ?1",
            params![local_item_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let remote_id = conn
        .query_row(
            "SELECT remote_id FROM error_items WHERE id = ?1",
            params![local_item_id],
            |row| row.get::<_, Option<String>>(0),
        )?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

    conn.execute(
        "UPDATE error_items
         SET remote_version = ?1,
             server_snapshot_json = ?2,
             updated_at = ?3
         WHERE id = ?4",
        params![conflict.0, conflict.1, now_str(), local_item_id],
    )?;
    conn.execute(
        "DELETE FROM error_sync_conflicts WHERE local_item_id = ?1",
        params![local_item_id],
    )?;
    conn.execute(
        "DELETE FROM error_sync_ops WHERE local_item_id = ?1 AND status = 'conflicted'",
        params![local_item_id],
    )?;

    let payload = build_item_payload(conn, local_item_id)?;
    upsert_pending_op(
        conn,
        "update",
        local_item_id,
        Some(remote_id.as_str()),
        Some(conflict.0),
        &payload,
    )?;
    Ok(())
}

pub fn resolve_conflict_accept_remote(
    conn: &Connection,
    local_item_id: &str,
) -> Result<(), rusqlite::Error> {
    let snapshot_text = conn
        .query_row(
            "SELECT server_snapshot_json
             FROM error_sync_conflicts
             WHERE local_item_id = ?1",
            params![local_item_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let snapshot: Value = serde_json::from_str(&snapshot_text).unwrap_or_else(|_| json!({}));

    upsert_remote_item_snapshot_inner(conn, &snapshot, true)?;
    conn.execute(
        "DELETE FROM error_sync_conflicts WHERE local_item_id = ?1",
        params![local_item_id],
    )?;
    conn.execute(
        "DELETE FROM error_sync_ops WHERE local_item_id = ?1 AND status = 'conflicted'",
        params![local_item_id],
    )?;
    Ok(())
}

pub fn get_sync_value(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM sync_state WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
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

pub fn purge_remote_cache(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM error_sync_conflicts", [])?;
    conn.execute("DELETE FROM error_sync_ops", [])?;
    conn.execute("DELETE FROM error_review_logs", [])?;
    conn.execute("DELETE FROM error_item_images", [])?;
    conn.execute("DELETE FROM error_items", [])?;
    conn.execute("DELETE FROM error_notebooks", [])?;
    delete_sync_value(conn, "last_error_sync_cursor")?;
    delete_sync_value(conn, "last_error_sync_at")?;
    Ok(())
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
    fn local_edit_updates_pending_create_payload_instead_of_creating_update_op() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();
        let draft =
            create_error_draft(&conn, Some("math"), "/tmp/a.jpg", "sha", "image/jpeg").unwrap();
        apply_analyze_response(
            &conn,
            &draft.id,
            &AnalyzeErrorResponse {
                question_text: Some("old".into()),
                answer_text: Some("a".into()),
                analysis: Some("b".into()),
                wrong_answer_text: None,
                mistake_analysis: None,
                mistake_status: None,
                knowledge_points: vec!["x".into()],
                mastery_level: Some(0),
                image: Some(RemoteErrorImage {
                    remote_key: Some("k".into()),
                    url: Some("/img".into()),
                    sha256: Some("sha".into()),
                    content_type: Some("image/jpeg".into()),
                    size: Some(1),
                }),
            },
        )
        .unwrap();

        update_error_item_text(
            &conn,
            &draft.id,
            Some("new"),
            None,
            None,
            None,
            None,
            Some("[\"x\"]"),
        )
        .unwrap();

        let ops = get_pending_sync_ops(&conn).unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].action, "create");
        assert_eq!(ops[0].payload["questionText"], "new");
    }

    #[test]
    fn pulled_snapshot_only_updates_server_snapshot_when_local_item_has_pending_op() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();
        let draft =
            create_error_draft(&conn, Some("math"), "/tmp/a.jpg", "sha", "image/jpeg").unwrap();
        apply_analyze_response(
            &conn,
            &draft.id,
            &AnalyzeErrorResponse {
                question_text: Some("local".into()),
                answer_text: Some("a".into()),
                analysis: Some("b".into()),
                wrong_answer_text: None,
                mistake_analysis: None,
                mistake_status: None,
                knowledge_points: vec![],
                mastery_level: Some(0),
                image: Some(RemoteErrorImage {
                    remote_key: Some("k".into()),
                    url: Some("/img".into()),
                    sha256: Some("sha".into()),
                    content_type: Some("image/jpeg".into()),
                    size: Some(1),
                }),
            },
        )
        .unwrap();
        conn.execute(
            "UPDATE error_items SET remote_id = 'remote-1', remote_version = 1, analysis_status = 'ready' WHERE id = ?1",
            params![draft.id],
        )
        .unwrap();
        update_error_item_text(
            &conn,
            &draft.id,
            Some("local edited"),
            None,
            None,
            None,
            None,
            Some("[]"),
        )
        .unwrap();

        upsert_remote_item_snapshot(
            &conn,
            &json!({
                "remoteId": "remote-1",
                "localId": draft.id,
                "notebookId": "math",
                "version": 2,
                "questionText": "server value",
                "knowledgePoints": [],
                "masteryLevel": 0,
                "ef": 2.5,
                "interval": 1,
                "repetitions": 0,
                "nextReview": "2026-06-27",
                "image": { "url": "/img" },
                "updatedAt": "2026-06-27 12:00:00"
            }),
            false,
        )
        .unwrap();

        let item = get_error_item(&conn, &draft.id).unwrap().unwrap();
        assert_eq!(item.question_text.as_deref(), Some("local edited"));
        assert_eq!(item.version, 2);
    }

    #[test]
    fn pulled_remote_only_item_persists_remote_image_url() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();

        let local_id = upsert_remote_item_snapshot(
            &conn,
            &json!({
                "remoteId": "remote-1",
                "localId": "remote-1",
                "notebookId": "math",
                "version": 1,
                "questionText": "server item",
                "knowledgePoints": [],
                "masteryLevel": 0,
                "ef": 2.5,
                "interval": 1,
                "repetitions": 0,
                "nextReview": "2026-06-27",
                "image": {
                    "url": "https://example.com/a.jpg",
                    "remoteKey": "key-1",
                    "sha256": "sha-1",
                    "contentType": "image/jpeg"
                },
                "updatedAt": "2026-06-27 12:00:00"
            }),
            true,
        )
        .unwrap();

        let item = get_error_item(&conn, &local_id).unwrap().unwrap();
        assert_eq!(
            item.remote_image_url.as_deref(),
            Some("https://example.com/a.jpg")
        );
        assert_eq!(item.local_image_path, None);
    }

    #[test]
    fn getter_backfills_missing_image_row_from_server_snapshot() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO error_items
                (id, remote_id, notebook_id, question_text, analysis_status, remote_version, server_snapshot_json, created_at, updated_at)
             VALUES ('item-1', 'remote-1', 'math', '题目', 'ready', 1, ?1, '2026-06-27 10:00:00', '2026-06-27 10:00:00')",
            params![json!({
                "remoteId": "remote-1",
                "localId": "item-1",
                "notebookId": "math",
                "version": 1,
                "questionText": "题目",
                "knowledgePoints": [],
                "masteryLevel": 0,
                "ef": 2.5,
                "interval": 1,
                "repetitions": 0,
                "nextReview": "2026-06-27",
                "image": {
                    "url": "https://example.com/fix.jpg",
                    "remoteKey": "key-fix",
                    "sha256": "sha-fix",
                    "contentType": "image/jpeg"
                }
            }).to_string()],
        )
        .unwrap();

        let item = get_error_item(&conn, "item-1").unwrap().unwrap();
        assert_eq!(
            item.remote_image_url.as_deref(),
            Some("https://example.com/fix.jpg")
        );

        let image_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM error_item_images WHERE error_item_id = 'item-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(image_count, 1);
    }

    #[test]
    fn pending_or_failed_local_items_do_not_keep_sync_ops_after_save() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();

        let draft =
            create_error_draft(&conn, Some("math"), "/tmp/a.jpg", "sha", "image/jpeg").unwrap();
        update_error_item_text(
            &conn,
            &draft.id,
            Some("pending edit"),
            None,
            None,
            None,
            None,
            Some("[]"),
        )
        .unwrap();
        assert!(get_pending_sync_ops(&conn).unwrap().is_empty());

        conn.execute(
            "INSERT INTO error_sync_ops
                (op_id, entity_type, action, local_item_id, payload_json, client_timestamp, status, created_at, updated_at)
             VALUES ('stale-op', 'error_item', 'create', ?1, '{}', '2026-06-27 12:00:00', 'conflicted', '2026-06-27 12:00:00', '2026-06-27 12:00:00')",
            params![draft.id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO error_sync_conflicts
                (id, local_item_id, op_id, server_version, server_snapshot_json, error_code, created_at, updated_at)
             VALUES ('stale-conflict', ?1, 'stale-op', 0, 'null', 'VALIDATION_ERROR', '2026-06-27 12:00:00', '2026-06-27 12:00:00')",
            params![draft.id],
        )
        .unwrap();

        apply_analyze_failure_response(
            &conn,
            &draft.id,
            &AnalyzeErrorFailureResponse {
                code: Some("AI_ANALYSIS_FAILED".into()),
                message: Some("boom".into()),
                image: Some(RemoteErrorImage {
                    remote_key: Some("img-key".into()),
                    url: Some("https://example.com/image.jpg".into()),
                    sha256: Some("sha".into()),
                    content_type: Some("image/jpeg".into()),
                    size: Some(1),
                }),
            },
        )
        .unwrap();
        update_error_item_text(
            &conn,
            &draft.id,
            Some("failed edit"),
            None,
            None,
            None,
            None,
            Some("[]"),
        )
        .unwrap();

        assert!(get_pending_sync_ops(&conn).unwrap().is_empty());
        assert!(list_error_sync_conflicts(&conn).unwrap().is_empty());
    }

    #[test]
    fn get_syncable_pending_ops_skips_non_ready_create_and_update_ops() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();

        let ready_draft =
            create_error_draft(&conn, Some("math"), "/tmp/a.jpg", "sha", "image/jpeg").unwrap();
        apply_analyze_response(
            &conn,
            &ready_draft.id,
            &AnalyzeErrorResponse {
                question_text: Some("ready".into()),
                answer_text: Some("a".into()),
                analysis: Some("b".into()),
                wrong_answer_text: None,
                mistake_analysis: None,
                mistake_status: None,
                knowledge_points: vec![],
                mastery_level: Some(0),
                image: Some(RemoteErrorImage {
                    remote_key: Some("k".into()),
                    url: Some("/img".into()),
                    sha256: Some("sha".into()),
                    content_type: Some("image/jpeg".into()),
                    size: Some(1),
                }),
            },
        )
        .unwrap();

        let failed_draft =
            create_error_draft(&conn, Some("math"), "/tmp/b.jpg", "sha-2", "image/jpeg").unwrap();
        conn.execute(
            "UPDATE error_items
             SET analysis_status = 'analyze_failed'
             WHERE id = ?1",
            params![failed_draft.id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO error_sync_ops
                (op_id, entity_type, action, local_item_id, payload_json, client_timestamp, status, created_at, updated_at)
             VALUES ('failed-op', 'error_item', 'create', ?1, '{}', '2026-06-27 12:00:00', 'pending', '2026-06-27 12:00:00', '2026-06-27 12:00:00')",
            params![failed_draft.id],
        )
        .unwrap();

        let all_ops = get_pending_sync_ops(&conn).unwrap();
        let syncable_ops = get_syncable_pending_ops(&conn).unwrap();
        assert_eq!(all_ops.len(), 2);
        assert_eq!(syncable_ops.len(), 1);
        assert_eq!(syncable_ops[0].local_item_id, ready_draft.id);
    }

    #[test]
    fn list_error_sync_conflicts_exposes_reason_and_snapshot_presence() {
        let conn = test_db();
        upsert_pulled_notebook(
            &conn,
            &json!({
                "remoteId": "math",
                "name": "数学",
                "createdAt": "2026-06-27 10:00:00",
                "updatedAt": "2026-06-27 10:00:00"
            }),
        )
        .unwrap();
        let draft =
            create_error_draft(&conn, Some("math"), "/tmp/a.jpg", "sha", "image/jpeg").unwrap();
        apply_analyze_response(
            &conn,
            &draft.id,
            &AnalyzeErrorResponse {
                question_text: Some("ready".into()),
                answer_text: Some("a".into()),
                analysis: Some("b".into()),
                wrong_answer_text: None,
                mistake_analysis: None,
                mistake_status: None,
                knowledge_points: vec![],
                mastery_level: Some(0),
                image: Some(RemoteErrorImage {
                    remote_key: Some("k".into()),
                    url: Some("/img".into()),
                    sha256: Some("sha".into()),
                    content_type: Some("image/jpeg".into()),
                    size: Some(1),
                }),
            },
        )
        .unwrap();

        record_sync_conflict(
            &conn,
            &json!({
                "opId": get_pending_sync_ops(&conn).unwrap()[0].op_id,
                "localItemId": draft.id,
                "code": "VALIDATION_ERROR",
                "serverVersion": 0,
                "serverSnapshot": null
            }),
        )
        .unwrap();

        let conflicts = list_error_sync_conflicts(&conn).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].reason, "validation_error");
        assert!(!conflicts[0].has_remote_snapshot);
    }
}
