use crate::utils::types::{ ContentType, HistoryItem };
use base64::{ engine::general_purpose::STANDARD, Engine };
use rand::{ rng, Rng };
use rand::distr::Alphanumeric;
use sqlx::{ Row, SqlitePool };
use std::fs;
use tauri_plugin_aptabase::EventTracker;

const SELECT_COLUMNS: &str =
    "id, source, source_icon, content_type, content, favicon, timestamp, language, pinned, tag_color";

/// Default retention when the user has not chosen a limit yet.
pub const DEFAULT_HISTORY_LIMIT: i64 = 1000;

/// Reads the user's "keep at most N items" setting (falling back to the
/// default) and deletes the oldest items beyond it. Pinned items are kept.
pub async fn enforce_history_limit(pool: &SqlitePool) -> Result<(), String> {
    let limit = read_history_limit(pool).await?;
    if limit <= 0 {
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM history WHERE id NOT IN (\
             SELECT id FROM history ORDER BY pinned DESC, timestamp DESC LIMIT ?)"
    )
    .bind(limit)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

async fn read_history_limit(pool: &SqlitePool) -> Result<i64, String> {
    let raw: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'history_limit'")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    match raw.and_then(|v| v.trim().parse::<i64>().ok()) {
        Some(n) if n >= 10 => Ok(n),
        _ => Ok(DEFAULT_HISTORY_LIMIT),
    }
}

/// Total number of stored history entries (for the settings page).
#[tauri::command]
pub async fn history_count(pool: tauri::State<'_, SqlitePool>) -> Result<i64, String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM history")
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(count)
}

fn row_to_item(row: &sqlx::sqlite::SqliteRow) -> HistoryItem {
    HistoryItem {
        id: row.get("id"),
        source: row.get("source"),
        source_icon: row.get("source_icon"),
        content_type: ContentType::from(row.get::<String, _>("content_type")),
        content: row.get("content"),
        favicon: row.get("favicon"),
        timestamp: row.get("timestamp"),
        language: row.get("language"),
        pinned: row.get::<i64, _>("pinned") != 0,
        tag_color: row.get("tag_color"),
    }
}

pub async fn initialize_history(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    sqlx::query(
        "INSERT INTO history (id, source, content_type, content, timestamp) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(id)
    .bind("System")
    .bind("text")
    .bind("Welcome to your clipboard history!")
    .execute(pool).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_history(pool: tauri::State<'_, SqlitePool>) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx
        ::query(
            &format!("SELECT {SELECT_COLUMNS} FROM history ORDER BY pinned DESC, timestamp DESC")
        )
        .fetch_all(&*pool).await
        .map_err(|e| e.to_string())?;

    let items = rows.iter().map(row_to_item).collect();

    Ok(items)
}

#[tauri::command]
pub async fn add_history_item(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    item: HistoryItem
) -> Result<(), String> {
    let (id, source, source_icon, content_type, content, favicon, timestamp, language, pinned, tag_color) =
        item.to_row();

    let existing = sqlx
        ::query("SELECT id FROM history WHERE content = ? AND content_type = ?")
        .bind(&content)
        .bind(&content_type)
        .fetch_optional(&*pool).await
        .map_err(|e| e.to_string())?;

    match existing {
        Some(_) => {
            // Re-copy of an existing item: bump its timestamp so it jumps to
            // the top, but keep the pinned state untouched.
            sqlx
                ::query(
                    "UPDATE history SET timestamp = strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now') WHERE content = ? AND content_type = ?"
                )
                .bind(&content)
                .bind(&content_type)
                .execute(&*pool).await
                .map_err(|e| e.to_string())?;
        }
        None => {
            sqlx
                ::query(
                    "INSERT INTO history (id, source, source_icon, content_type, content, favicon, timestamp, language, pinned, tag_color) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(id)
                .bind(source)
                .bind(source_icon)
                .bind(content_type)
                .bind(content)
                .bind(favicon)
                .bind(timestamp)
                .bind(language)
                .bind(pinned)
                .bind(tag_color)
                .execute(&*pool).await
                .map_err(|e| e.to_string())?;
        }
    }

    // Keep the database within the user's retention limit (pinned items
    // always survive).
    if let Err(e) = enforce_history_limit(&*pool).await {
        eprintln!("Failed to enforce history limit: {}", e);
    }

    let _ = app_handle.track_event(
        "history_item_added",
        Some(serde_json::json!({
        "content_type": item.content_type.to_string()
    }))
    );

    Ok(())
}

#[tauri::command]
pub async fn search_history(
    pool: tauri::State<'_, SqlitePool>,
    query: String
) -> Result<Vec<HistoryItem>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let query = format!("%{}%", query);

    let rows = sqlx
        ::query(
            &format!(
                "SELECT {SELECT_COLUMNS} \
                 FROM history \
                 WHERE content LIKE ? \
                 ORDER BY pinned DESC, timestamp DESC \
                 LIMIT 100"
            )
        )
        .bind(query)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let items = rows.iter().map(row_to_item).collect();

    Ok(items)
}

#[tauri::command]
pub async fn load_history_chunk(
    pool: tauri::State<'_, SqlitePool>,
    offset: i64,
    limit: i64
) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx
        ::query(
            &format!(
                "SELECT {SELECT_COLUMNS} FROM history ORDER BY pinned DESC, timestamp DESC LIMIT ? OFFSET ?"
            )
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let items = rows.iter().map(row_to_item).collect();

    Ok(items)
}

#[tauri::command]
pub async fn delete_history_item(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    id: String
) -> Result<(), String> {
    sqlx
        ::query("DELETE FROM history WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event("history_item_deleted", None);

    Ok(())
}

#[tauri::command]
pub async fn clear_history(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>
) -> Result<(), String> {
    sqlx
        ::query("DELETE FROM history")
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event("history_cleared", None);

    Ok(())
}

/// Toggles the pinned state of an item and returns the new state.
#[tauri::command]
pub async fn toggle_pin(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    id: String
) -> Result<bool, String> {
    let current: Option<i64> = sqlx
        ::query_scalar("SELECT pinned FROM history WHERE id = ?")
        .bind(&id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let value = match current {
        Some(p) => 1 - p,
        None => return Err("Item not found".to_string()),
    };

    sqlx
        ::query("UPDATE history SET pinned = ? WHERE id = ?")
        .bind(value)
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event(
        "history_item_pinned",
        Some(serde_json::json!({ "pinned": value == 1 })),
    );

    Ok(value == 1)
}

/// Sets (or clears, with `color = None`) the color tag of an entry.
#[tauri::command]
pub async fn tag_history_item(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    id: String,
    color: Option<String>
) -> Result<(), String> {
    let result = sqlx
        ::query("UPDATE history SET tag_color = ? WHERE id = ?")
        .bind(color.as_deref())
        .bind(&id)
        .execute(&*pool).await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Item not found".to_string());
    }

    let _ = app_handle.track_event(
        "history_item_tagged",
        Some(serde_json::json!({ "tagged": color.is_some() })),
    );

    Ok(())
}

#[tauri::command]
pub async fn read_image(filename: String) -> Result<String, String> {
    let bytes = fs::read(filename).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}
