use crate::database::DatabaseConnection;
use crate::models::AppConfig;
use rusqlite::{Result as SqliteResult, params};
use std::collections::HashMap;

/// Database operations for application configuration
pub struct ConfigOperations<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ConfigOperations<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Save the complete application configuration
    pub fn save_config(&self, config: &AppConfig) -> SqliteResult<()> {
        config.validate().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some(e)
            )
        })?;

        let key_value_map = config.to_key_value_map();
        let now = chrono::Utc::now().to_rfc3339();

        self.db.transaction(|tx| {
            for (key, value) in key_value_map {
                tx.execute(
                    r#"
                    INSERT OR REPLACE INTO app_config (key, value, updated_at)
                    VALUES (?1, ?2, ?3)
                    "#,
                    params![key, value, now],
                )?;
            }
            Ok(())
        })
    }

    /// Load the complete application configuration
    pub fn load_config(&self) -> SqliteResult<AppConfig> {
        let mut config_map = HashMap::new();

        self.db.prepare_and_execute(
            "SELECT key, value FROM app_config",
            |stmt| {
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>("key")?,
                        row.get::<_, String>("value")?
                    ))
                })?;

                for row in rows {
                    let (key, value) = row?;
                    config_map.insert(key, value);
                }
                Ok(())
            }
        )?;

        // If no configuration exists, return default
        if config_map.is_empty() {
            let default_config = AppConfig::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        // Parse configuration from key-value map
        AppConfig::from_key_value_map(&config_map).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some(e)
            )
        })
    }

    /// Set a single configuration value
    pub fn set_value(&self, key: &str, value: &str) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.db.execute(
            r#"
            INSERT OR REPLACE INTO app_config (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            "#,
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Get a single configuration value
    pub fn get_value(&self, key: &str) -> SqliteResult<Option<String>> {
        match self.db.query_row(
            "SELECT value FROM app_config WHERE key = ?1",
            params![key],
            |row| Ok(row.get::<_, String>("value")?),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Delete a configuration key
    pub fn delete_key(&self, key: &str) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "DELETE FROM app_config WHERE key = ?1",
            params![key],
        )?;
        Ok(rows_affected > 0)
    }

    /// Get all configuration keys and values
    pub fn get_all(&self) -> SqliteResult<HashMap<String, String>> {
        let mut config_map = HashMap::new();

        self.db.prepare_and_execute(
            "SELECT key, value FROM app_config ORDER BY key",
            |stmt| {
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>("key")?,
                        row.get::<_, String>("value")?
                    ))
                })?;

                for row in rows {
                    let (key, value) = row?;
                    config_map.insert(key, value);
                }
                Ok(())
            }
        )?;

        Ok(config_map)
    }

    /// Clear all configuration (reset to defaults)
    pub fn reset_to_defaults(&self) -> SqliteResult<()> {
        self.db.execute("DELETE FROM app_config", &[])?;
        let default_config = AppConfig::default();
        self.save_config(&default_config)
    }
}