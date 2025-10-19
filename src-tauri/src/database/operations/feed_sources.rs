use crate::database::DatabaseConnection;
use crate::models::FeedSource;
use rusqlite::{Result as SqliteResult, params};

/// Database operations for feed sources
pub struct FeedSourceOperations<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> FeedSourceOperations<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Insert a new feed source into the database
    pub fn insert(&self, feed_source: &FeedSource) -> SqliteResult<()> {
        feed_source.validate().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some(e)
            )
        })?;

        self.db.execute(
            r#"
            INSERT INTO feed_sources (id, name, url, enabled, last_fetched, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                feed_source.id,
                feed_source.name,
                feed_source.url,
                feed_source.enabled,
                feed_source.last_fetched,
                feed_source.created_at
            ],
        )?;

        Ok(())
    }

    /// Get a feed source by ID
    pub fn get_by_id(&self, id: &str) -> SqliteResult<Option<FeedSource>> {
        match self.db.query_row(
            "SELECT * FROM feed_sources WHERE id = ?1",
            params![id],
            |row| FeedSource::from_row(row),
        ) {
            Ok(feed_source) => Ok(Some(feed_source)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get all feed sources
    pub fn get_all(&self) -> SqliteResult<Vec<FeedSource>> {
        self.db.prepare_and_execute(
            "SELECT * FROM feed_sources ORDER BY created_at ASC",
            |stmt| {
                let rows = stmt.query_map([], |row| FeedSource::from_row(row))?;
                
                let mut feed_sources = Vec::new();
                for row in rows {
                    feed_sources.push(row?);
                }
                Ok(feed_sources)
            }
        )
    }

    /// Get only enabled feed sources
    pub fn get_enabled(&self) -> SqliteResult<Vec<FeedSource>> {
        self.db.prepare_and_execute(
            "SELECT * FROM feed_sources WHERE enabled = TRUE ORDER BY created_at ASC",
            |stmt| {
                let rows = stmt.query_map([], |row| FeedSource::from_row(row))?;
                
                let mut feed_sources = Vec::new();
                for row in rows {
                    feed_sources.push(row?);
                }
                Ok(feed_sources)
            }
        )
    }

    /// Update a feed source
    pub fn update(&self, feed_source: &FeedSource) -> SqliteResult<bool> {
        feed_source.validate().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some(e)
            )
        })?;

        let rows_affected = self.db.execute(
            r#"
            UPDATE feed_sources 
            SET name = ?2, url = ?3, enabled = ?4, last_fetched = ?5
            WHERE id = ?1
            "#,
            params![
                feed_source.id,
                feed_source.name,
                feed_source.url,
                feed_source.enabled,
                feed_source.last_fetched
            ],
        )?;

        Ok(rows_affected > 0)
    }

    /// Enable or disable a feed source
    pub fn set_enabled(&self, id: &str, enabled: bool) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "UPDATE feed_sources SET enabled = ?2 WHERE id = ?1",
            params![id, enabled],
        )?;
        Ok(rows_affected > 0)
    }

    /// Update the last fetched timestamp for a feed source
    pub fn update_last_fetched(&self, id: &str, timestamp: &str) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "UPDATE feed_sources SET last_fetched = ?2 WHERE id = ?1",
            params![id, timestamp],
        )?;
        Ok(rows_affected > 0)
    }

    /// Delete a feed source by ID
    pub fn delete(&self, id: &str) -> SqliteResult<bool> {
        // First delete all articles from this source
        self.db.execute(
            "DELETE FROM articles WHERE source_id = ?1",
            params![id],
        )?;

        // Then delete the feed source
        let rows_affected = self.db.execute(
            "DELETE FROM feed_sources WHERE id = ?1",
            params![id],
        )?;

        Ok(rows_affected > 0)
    }

    /// Check if a feed source exists by URL
    pub fn exists_by_url(&self, url: &str) -> SqliteResult<bool> {
        match self.db.query_row(
            "SELECT COUNT(*) FROM feed_sources WHERE url = ?1",
            params![url],
            |row| Ok(row.get::<_, u32>(0)?),
        ) {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(e),
        }
    }

    /// Get the count of feed sources
    pub fn count(&self, enabled_only: bool) -> SqliteResult<u32> {
        let sql = if enabled_only {
            "SELECT COUNT(*) FROM feed_sources WHERE enabled = TRUE"
        } else {
            "SELECT COUNT(*) FROM feed_sources"
        };

        self.db.query_row(sql, &[], |row| Ok(row.get::<_, u32>(0)?))
    }
}