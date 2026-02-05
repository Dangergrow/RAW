use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub telemetry_enabled: bool,
    pub dark_mode: bool,
    pub incognito: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            telemetry_enabled: false,
            dark_mode: true,
            incognito: false,
        }
    }
}

pub struct PrivacyStore {
    conn: Connection,
}

impl PrivacyStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS cookie_jar(domain TEXT, key TEXT, value TEXT, created_at TEXT);",
        )?;
        Ok(Self { conn })
    }

    pub fn save_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings(key, value) VALUES(?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn add_cookie(&self, domain: &str, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO cookie_jar(domain, key, value, created_at) VALUES(?1, ?2, ?3, ?4)",
            params![domain, key, value, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn clear_cookies(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM cookie_jar", [])?;
        Ok(())
    }
}
