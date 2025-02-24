use directories::ProjectDirs;
use rusqlite::{Connection, Result as SqliteResult, Error as SqliteError};
use std::fs;
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> SqliteResult<Self> {
        let db_path = get_database_path()?;
        // Handle IO error separately since it can't be automatically converted
        if let Err(e) = fs::create_dir_all(db_path.parent().unwrap()) {
            return Err(SqliteError::SqliteFailure(
                rusqlite::ffi::Error::new(1), // SQLITE_ERROR
                Some(format!("IO error: {}", e))
            ));
        }
        
        let conn = Connection::open(&db_path)?;
        
        // Create the table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database { conn })
    }

    pub fn set_api_key(&self, api_key: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('anthropic_api_key', ?1)",
            [api_key],
        )?;
        Ok(())
    }

    pub fn get_api_key(&self) -> SqliteResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM settings WHERE key = 'anthropic_api_key'"
        )?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_api_key(&self) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM settings WHERE key = 'anthropic_api_key'",
            [],
        )?;
        Ok(())
    }
}

fn get_database_path() -> SqliteResult<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "llm-writing-feedback", "app")
        .ok_or_else(|| SqliteError::InvalidPath("Could not determine project directory".into()))?;
    
    let data_dir = proj_dirs.data_dir();
    Ok(data_dir.join("settings.db"))
} 