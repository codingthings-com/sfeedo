use crate::database::DatabaseConnection;
use crate::models::Article;
use rusqlite::{Result as SqliteResult, params};

/// Database operations for articles
pub struct ArticleOperations<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> ArticleOperations<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Insert a new article into the database
    pub fn insert(&self, article: &Article) -> SqliteResult<()> {
        article.validate().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some(e)
            )
        })?;

        self.db.execute(
            r#"
            INSERT INTO articles (id, title, summary, content, url, source_id, published_at, fetched_at, is_read)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                article.id,
                article.title,
                article.summary,
                article.content,
                article.url,
                article.source_id,
                article.published_at,
                article.fetched_at,
                article.is_read
            ],
        )?;

        Ok(())
    }

    /// Insert multiple articles in a single transaction
    pub fn insert_batch(&self, articles: &[Article]) -> SqliteResult<()> {
        self.db.transaction(|tx| {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR IGNORE INTO articles (id, title, summary, content, url, source_id, published_at, fetched_at, is_read)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#
            )?;

            for article in articles {
                article.validate().map_err(|e| {
                    rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                        Some(e)
                    )
                })?;

                stmt.execute(params![
                    article.id,
                    article.title,
                    article.summary,
                    article.content,
                    article.url,
                    article.source_id,
                    article.published_at,
                    article.fetched_at,
                    article.is_read
                ])?;
            }

            Ok(())
        })
    }

    /// Get an article by ID
    pub fn get_by_id(&self, id: &str) -> SqliteResult<Option<Article>> {
        match self.db.query_row(
            "SELECT * FROM articles WHERE id = ?1",
            params![id],
            |row| Article::from_row(row),
        ) {
            Ok(article) => Ok(Some(article)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get articles with pagination and optional filtering
    pub fn get_articles(&self, limit: Option<u32>, offset: Option<u32>, source_id: Option<&str>, unread_only: bool) -> SqliteResult<Vec<Article>> {
        let mut sql = String::from("SELECT * FROM articles WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(source_id) = source_id {
            sql.push_str(" AND source_id = ?");
            params.push(Box::new(source_id.to_string()));
        }

        if unread_only {
            sql.push_str(" AND is_read = FALSE");
        }

        sql.push_str(" ORDER BY published_at DESC");

        if let Some(limit) = limit {
            sql.push_str(" LIMIT ?");
            params.push(Box::new(limit));
        }

        if let Some(offset) = offset {
            sql.push_str(" OFFSET ?");
            params.push(Box::new(offset));
        }

        self.db.prepare_and_execute(&sql, |stmt| {
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(&param_refs[..], |row| Article::from_row(row))?;
            
            let mut articles = Vec::new();
            for row in rows {
                articles.push(row?);
            }
            Ok(articles)
        })
    }

    /// Mark an article as read
    pub fn mark_as_read(&self, id: &str) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "UPDATE articles SET is_read = TRUE WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    /// Mark an article as unread
    pub fn mark_as_unread(&self, id: &str) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "UPDATE articles SET is_read = FALSE WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    /// Delete an article by ID
    pub fn delete(&self, id: &str) -> SqliteResult<bool> {
        let rows_affected = self.db.execute(
            "DELETE FROM articles WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    /// Delete articles older than the specified number of days
    pub fn delete_old_articles(&self, days: u32) -> SqliteResult<usize> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff_date.to_rfc3339();

        self.db.execute(
            "DELETE FROM articles WHERE published_at < ?1",
            params![cutoff_str],
        )
    }

    /// Get the count of articles
    pub fn count(&self, source_id: Option<&str>, unread_only: bool) -> SqliteResult<u32> {
        let mut sql = String::from("SELECT COUNT(*) FROM articles WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(source_id) = source_id {
            sql.push_str(" AND source_id = ?");
            params.push(Box::new(source_id.to_string()));
        }

        if unread_only {
            sql.push_str(" AND is_read = FALSE");
        }

        self.db.query_row(&sql, &params.iter().map(|p| p.as_ref()).collect::<Vec<_>>()[..], |row| {
            Ok(row.get::<_, u32>(0)?)
        })
    }

    /// Search articles by title or content
    pub fn search(&self, query: &str, limit: Option<u32>) -> SqliteResult<Vec<Article>> {
        let search_pattern = format!("%{}%", query);
        
        match limit {
            Some(limit_val) => {
                let sql = "SELECT * FROM articles WHERE title LIKE ?1 OR summary LIKE ?1 OR content LIKE ?1 ORDER BY published_at DESC LIMIT ?2";
                self.db.prepare_and_execute(sql, |stmt| {
                    let rows = stmt.query_map(params![search_pattern, limit_val], |row| Article::from_row(row))?;
                    
                    let mut articles = Vec::new();
                    for row in rows {
                        articles.push(row?);
                    }
                    Ok(articles)
                })
            },
            None => {
                let sql = "SELECT * FROM articles WHERE title LIKE ?1 OR summary LIKE ?1 OR content LIKE ?1 ORDER BY published_at DESC";
                self.db.prepare_and_execute(sql, |stmt| {
                    let rows = stmt.query_map(params![search_pattern], |row| Article::from_row(row))?;
                    
                    let mut articles = Vec::new();
                    for row in rows {
                        articles.push(row?);
                    }
                    Ok(articles)
                })
            }
        }
    }
}