//! Database module for persistent storage
//!
//! Provides SQLite-based storage for download and search history.

use chrono::Utc;
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Database manager for persistent storage
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

/// Download history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: String,
    pub file_name: Option<String>,
    pub size: Option<i64>,
    pub network: String,
    pub bot: String,
    pub channel: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
    pub completed_at: String,
}

/// Search history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRecord {
    pub id: i64,
    pub query: String,
    pub results_count: i64,
    pub results_json: Option<String>,
    pub searched_at: String,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
    pub total_pages: i64,
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // Download history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS download_history (
                id TEXT PRIMARY KEY,
                file_name TEXT,
                size INTEGER,
                network TEXT NOT NULL,
                bot TEXT NOT NULL,
                channel TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT NOT NULL
            )",
            [],
        )?;

        // Search history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                results_count INTEGER NOT NULL DEFAULT 0,
                results_json TEXT,
                searched_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: add results_json column if it doesn't exist
        let _ = conn.execute(
            "ALTER TABLE search_history ADD COLUMN results_json TEXT",
            [],
        );

        // Create indexes for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_download_completed_at ON download_history(completed_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_search_searched_at ON search_history(searched_at DESC)",
            [],
        )?;

        Ok(())
    }

    // ==================== Download History ====================

    /// Insert a download record
    pub fn insert_download(&self, record: &DownloadRecord) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO download_history 
             (id, file_name, size, network, bot, channel, status, error, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.id,
                record.file_name,
                record.size,
                record.network,
                record.bot,
                record.channel,
                record.status,
                record.error,
                record.created_at,
                record.completed_at,
            ],
        )?;
        Ok(())
    }

    /// List download history with pagination
    pub fn list_downloads(
        &self,
        page: i64,
        limit: i64,
    ) -> SqliteResult<PaginatedResponse<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();

        // Get total count
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM download_history", [], |row| {
            row.get(0)
        })?;

        let offset = (page - 1) * limit;
        let mut stmt = conn.prepare(
            "SELECT id, file_name, size, network, bot, channel, status, error, created_at, completed_at
             FROM download_history
             ORDER BY completed_at DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let items = stmt
            .query_map(params![limit, offset], |row| {
                Ok(DownloadRecord {
                    id: row.get(0)?,
                    file_name: row.get(1)?,
                    size: row.get(2)?,
                    network: row.get(3)?,
                    bot: row.get(4)?,
                    channel: row.get(5)?,
                    status: row.get(6)?,
                    error: row.get(7)?,
                    created_at: row.get(8)?,
                    completed_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let total_pages = (total + limit - 1) / limit;

        Ok(PaginatedResponse {
            items,
            total,
            page,
            limit,
            total_pages,
        })
    }

    /// Delete a download record
    pub fn delete_download(&self, id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM download_history WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Bulk delete download records
    pub fn bulk_delete_downloads(&self, ids: &[String]) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<_> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM download_history WHERE id IN ({})",
            placeholders.join(",")
        );

        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = conn.execute(&sql, params.as_slice())?;
        Ok(rows)
    }

    // ==================== Search History ====================

    /// Insert a search record with results
    pub fn insert_search(
        &self,
        query: &str,
        results_count: i64,
        results_json: Option<&str>,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO search_history (query, results_count, results_json, searched_at) VALUES (?1, ?2, ?3, ?4)",
            params![query, results_count, results_json, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// List search history with pagination
    pub fn list_searches(
        &self,
        page: i64,
        limit: i64,
    ) -> SqliteResult<PaginatedResponse<SearchRecord>> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))?;

        let offset = (page - 1) * limit;
        let mut stmt = conn.prepare(
            "SELECT id, query, results_count, results_json, searched_at
             FROM search_history
             ORDER BY searched_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let items = stmt
            .query_map(params![limit, offset], |row| {
                Ok(SearchRecord {
                    id: row.get(0)?,
                    query: row.get(1)?,
                    results_count: row.get(2)?,
                    results_json: row.get(3)?,
                    searched_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let total_pages = (total + limit - 1) / limit;

        Ok(PaginatedResponse {
            items,
            total,
            page,
            limit,
            total_pages,
        })
    }

    /// Delete a search record
    pub fn delete_search(&self, id: i64) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM search_history WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Bulk delete search records
    pub fn bulk_delete_searches(&self, ids: &[i64]) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<_> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM search_history WHERE id IN ({})",
            placeholders.join(",")
        );

        let params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = conn.execute(&sql, params.as_slice())?;
        Ok(rows)
    }

    /// Clear all search history
    pub fn clear_search_history(&self) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM search_history", [])?;
        Ok(rows)
    }
}
