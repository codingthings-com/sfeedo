use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// Database connection manager for the feed reader application
pub struct DatabaseConnection {
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub fn new(app_handle: &AppHandle) -> SqliteResult<Self> {
        let db_path = Self::get_database_path(app_handle)?;
        
        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("Failed to create database directory: {}", e))
                )
            })?;
        }

        let connection = Connection::open(&db_path)?;
        
        // Enable foreign key constraints
        connection.execute("PRAGMA foreign_keys = ON;", [])?;
        
        Ok(DatabaseConnection {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Create a new database connection with a specific path (for testing)
    #[cfg(test)]
    pub fn new_with_path<P: AsRef<std::path::Path>>(db_path: P) -> SqliteResult<Self> {
        let connection = Connection::open(&db_path)?;
        
        // Enable foreign key constraints
        connection.execute("PRAGMA foreign_keys = ON;", [])?;
        
        Ok(DatabaseConnection {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Get the database file path based on the app's data directory
    fn get_database_path(app_handle: &AppHandle) -> SqliteResult<PathBuf> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("Failed to get app data directory: {}", e))
                )
            })?;
        
        Ok(app_data_dir.join("sfeedo.db"))
    }

    /// Execute a query that returns no results
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<usize> {
        let conn = self.connection.lock().unwrap();
        conn.execute(sql, params)
    }

    /// Execute a query and return a single result
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> SqliteResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        conn.query_row(sql, params, f)
    }

    /// Prepare and execute a statement
    pub fn prepare_and_execute<F, R>(&self, sql: &str, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&mut rusqlite::Statement) -> SqliteResult<R>,
    {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        f(&mut stmt)
    }

    /// Begin a transaction
    pub fn transaction<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> SqliteResult<R>,
    {
        let conn = self.connection.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}