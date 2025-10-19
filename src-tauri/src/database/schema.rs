/// SQL schema definitions for the desktop feed reader application

pub const CREATE_ARTICLES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS articles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    summary TEXT,
    content TEXT,
    url TEXT NOT NULL,
    source_id TEXT NOT NULL,
    published_at DATETIME NOT NULL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_read BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (source_id) REFERENCES feed_sources (id)
);
"#;

pub const CREATE_FEED_SOURCES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS feed_sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    last_fetched DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"#;

pub const CREATE_APP_CONFIG_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS app_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
"#;

pub const CREATE_ARTICLES_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_articles_published_at ON articles(published_at DESC);
"#;

pub const CREATE_ARTICLES_SOURCE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_articles_source_id ON articles(source_id);
"#;

pub const CREATE_ARTICLES_READ_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_articles_is_read ON articles(is_read);
"#;