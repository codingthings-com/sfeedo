use rusqlite::Result as SqliteResult;
use crate::database::{DatabaseConnection, schema};

/// Database migration system for the feed reader application
pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// List of all database migrations in order
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Create initial tables",
        sql: schema::CREATE_FEED_SOURCES_TABLE,
    },
    Migration {
        version: 2,
        description: "Create articles table",
        sql: schema::CREATE_ARTICLES_TABLE,
    },
    Migration {
        version: 3,
        description: "Create app config table",
        sql: schema::CREATE_APP_CONFIG_TABLE,
    },
    Migration {
        version: 4,
        description: "Create articles published_at index",
        sql: schema::CREATE_ARTICLES_INDEX,
    },
    Migration {
        version: 5,
        description: "Create articles source_id index",
        sql: schema::CREATE_ARTICLES_SOURCE_INDEX,
    },
    Migration {
        version: 6,
        description: "Create articles is_read index",
        sql: schema::CREATE_ARTICLES_READ_INDEX,
    },
];

/// Run all pending database migrations
pub fn run_migrations(db: &DatabaseConnection) -> SqliteResult<()> {
    // Create migrations table if it doesn't exist
    db.execute(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        &[],
    )?;

    // Get the current migration version
    let current_version = get_current_migration_version(db)?;

    // Apply pending migrations
    for migration in MIGRATIONS {
        if migration.version > current_version {
            apply_migration(db, migration)?;
        }
    }

    Ok(())
}

/// Get the current migration version from the database
fn get_current_migration_version(db: &DatabaseConnection) -> SqliteResult<i32> {
    match db.query_row(
        "SELECT MAX(version) FROM migrations",
        &[],
        |row| row.get::<_, Option<i32>>(0),
    ) {
        Ok(Some(version)) => Ok(version),
        Ok(None) => Ok(0), // No migrations applied yet
        Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("no such table") => Ok(0),
        Err(e) => Err(e),
    }
}

/// Apply a single migration
fn apply_migration(db: &DatabaseConnection, migration: &Migration) -> SqliteResult<()> {
    db.transaction(|tx| {
        // Execute the migration SQL
        tx.execute(migration.sql, [])?;
        
        // Record the migration as applied
        tx.execute(
            "INSERT INTO migrations (version, description) VALUES (?1, ?2)",
            &[&migration.version as &dyn rusqlite::ToSql, &migration.description],
        )?;
        
        Ok(())
    })?;

    log::info!("Applied migration {}: {}", migration.version, migration.description);
    Ok(())
}