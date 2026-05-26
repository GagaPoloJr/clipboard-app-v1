use rusqlite::{params, Connection, Result, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub preview: String,
    pub app_name: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text',
                preview TEXT NOT NULL DEFAULT '',
                app_name TEXT,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_history(is_pinned DESC);",
        )?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert(&self, item: &ClipboardItem) -> Result<ClipboardItem> {
        let conn = self.conn.lock().unwrap();
        
        let existing: Option<(String, i32)> = conn.query_row(
            "SELECT id, is_pinned FROM clipboard_history WHERE content = ?1",
            params![item.content],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;

        let mut final_item = item.clone();

        if let Some((id, is_pinned)) = existing {
            conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?;
            if is_pinned != 0 {
                final_item.is_pinned = true;
            }
        }

        conn.execute(
            "INSERT INTO clipboard_history (id, content, content_type, preview, app_name, is_pinned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                final_item.id,
                final_item.content,
                final_item.content_type,
                final_item.preview,
                final_item.app_name,
                final_item.is_pinned as i32,
                final_item.created_at,
                final_item.updated_at,
            ],
        )?;
        Ok(final_item)
    }

    pub fn get_all(&self, search_query: Option<&str>) -> Result<Vec<ClipboardItem>> {
        let conn = self.conn.lock().unwrap();
        if let Some(q) = search_query {
            if !q.is_empty() {
                let pattern = format!("%{}%", q);
                let mut stmt = conn.prepare(
                    "SELECT id, content, content_type, preview, app_name, is_pinned, created_at, updated_at
                     FROM clipboard_history
                     WHERE content LIKE ?1 OR preview LIKE ?1
                     ORDER BY is_pinned DESC, created_at DESC",
                )?;
                let items = stmt
                    .query_map(params![pattern], |row| {
                        Ok(ClipboardItem {
                            id: row.get(0)?,
                            content: row.get(1)?,
                            content_type: row.get(2)?,
                            preview: row.get(3)?,
                            app_name: row.get(4)?,
                            is_pinned: row.get::<_, i32>(5)? != 0,
                            created_at: row.get(6)?,
                            updated_at: row.get(7)?,
                        })
                    })?
                    .collect::<Result<Vec<_>>>()?;
                return Ok(items);
            }
        }
        let mut stmt = conn.prepare(
            "SELECT id, content, content_type, preview, app_name, is_pinned, created_at, updated_at
             FROM clipboard_history
             ORDER BY is_pinned DESC, created_at DESC",
        )?;
        let items = stmt
            .query_map([], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_type: row.get(2)?,
                    preview: row.get(3)?,
                    app_name: row.get(4)?,
                    is_pinned: row.get::<_, i32>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM clipboard_history WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE clipboard_history SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END, updated_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )?;
        let is_pinned: bool = conn
            .query_row(
                "SELECT is_pinned FROM clipboard_history WHERE id = ?1",
                params![id],
                |row| row.get::<_, i32>(0),
            )
            .map(|v| v != 0)?;
        Ok(is_pinned)
    }

    pub fn clear_unpinned(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clipboard_history WHERE is_pinned = 0", [])?;
        Ok(())
    }

    pub fn purge_oldest(&self, keep_count: usize) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN (
                SELECT id FROM clipboard_history ORDER BY is_pinned DESC, created_at DESC LIMIT ?1
            ) AND is_pinned = 0",
            params![keep_count as i64],
        )?;
        Ok(())
    }
}

pub fn create_clipboard_item(
    content: &str,
    content_type: &str,
    preview: &str,
) -> Result<ClipboardItem> {
    let now = chrono::Utc::now().to_rfc3339();
    let item = ClipboardItem {
        id: uuid::Uuid::new_v4().to_string(),
        content: content.to_string(),
        content_type: content_type.to_string(),
        preview: preview.to_string(),
        app_name: None,
        is_pinned: false,
        created_at: now.clone(),
        updated_at: now,
    };
    Ok(item)
}
