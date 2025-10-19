# Finance News Aggregator Integration

This document explains how Sfeedo integrates with the `finance-news-aggregator-rs` crate to provide robust financial news feed processing.

## Overview

Sfeedo uses ONLY the `finance-news-aggregator-rs` crate (version 0.1.2) to fetch financial news from major sources like Yahoo Finance, CNBC, MarketWatch, Seeking Alpha, WSJ, NASDAQ, and CNN Finance. This integration provides:

- **Robust Feed Parsing**: Leverages the specialized parsing logic from `finance-news-aggregator-rs`
- **Multiple Financial Sources**: Built-in support for major financial news providers
- **No RSS Fallback**: Only supported financial sources work - no custom RSS parsing that fails
- **Error Handling**: Comprehensive error handling and retry logic

## Supported Financial Sources

The following sources are automatically detected and use the `finance-news-aggregator-rs` client:

- **Yahoo Finance** - Market Summary
- **CNBC** - Business News  
- **MarketWatch** - Market Pulse
- **Seeking Alpha** - Latest Articles
- **Wall Street Journal** - Market News
- **NASDAQ** - Stock News
- **CNN Finance** - Markets

## How It Works

### 1. Source Detection

When fetching from a feed source, the system automatically detects if it's a supported financial source by checking:

- Feed source name (case-insensitive matching)
- Feed URL patterns

```rust
// Example: These will be detected as financial sources
FeedSource::new("yahoo-finance", "Yahoo Finance", "https://finance.yahoo.com/rss/")
FeedSource::new("cnbc-business", "CNBC Business", "https://www.cnbc.com/rss/")
```

### 2. Finance-Only Fetching

```rust
// The aggregator ONLY works with supported financial sources:
let mut aggregator = FeedAggregator::new();

// Only financial sources supported by finance-news-aggregator-rs work
// Unsupported sources will return a configuration error
let articles = aggregator.fetch_from_source(&feed_source).await?;
```

### 3. Unified Article Model

All articles, regardless of source, are converted to Sfeedo's unified `Article` model:

```rust
pub struct Article {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: String,
    pub source_id: String,
    pub published_at: String,
    pub read: bool,
    pub created_at: String,
}
```

## Default Financial Sources

Sfeedo comes with pre-configured financial news sources that are automatically added when the application starts with no existing configuration:

```rust
use sfeedo_lib::default_sources::get_default_financial_sources;

let sources = get_default_financial_sources();
// Returns 10 pre-configured financial news sources
```

## Configuration

### Adding Financial Sources

You can add financial sources through the UI or programmatically:

```rust
// Through the configuration service
let config_service = ConfigurationService::new(&app_handle)?;
config_service.add_feed_source(
    "Yahoo Finance".to_string(),
    "https://finance.yahoo.com/rss/".to_string()
)?;
```

### Unsupported Sources

Non-financial sources will return an error:

```rust
// This will return a ConfigurationError
config_service.add_feed_source(
    "BBC News".to_string(),
    "https://feeds.bbci.co.uk/news/rss.xml".to_string()
)?; // Error: "Unsupported news source: BBC News. Only financial news sources are supported."
```

## Error Handling

The integration includes comprehensive error handling:

- **Network Errors**: Automatic retry with exponential backoff
- **Parse Errors**: Graceful fallback between parsing methods
- **Source Failures**: Individual source failures don't affect other sources
- **Rate Limiting**: Built-in delays between requests

## Example Usage

See `examples/finance_feed_example.rs` for a complete example:

```bash
cd src-tauri
cargo run --example finance_feed_example
```

## Benefits

1. **Reliability**: Uses battle-tested parsing logic from `finance-news-aggregator-rs`
2. **Performance**: Optimized for financial news sources
3. **Simplicity**: No complex RSS parsing that fails - only supported sources work
4. **Maintainability**: Zero custom parsing code to maintain
5. **Accuracy**: Better date parsing and content extraction for financial news
6. **No Failures**: Eliminates RSS parsing failures by not doing RSS parsing

## Troubleshooting

### Common Issues

1. **Network Timeouts**: The system includes automatic retry logic
2. **Unsupported Sources**: Only financial sources work - add only supported sources
3. **Rate Limiting**: Built-in delays prevent overwhelming news sources

### Debugging

Enable debug logging to see detailed fetch information:

```rust
env_logger::init();
// or in your app config
log::set_max_level(log::LevelFilter::Debug);
```

## Future Enhancements

- Support for additional financial news sources
- Configurable retry policies
- Advanced filtering and categorization
- Real-time news updates via WebSocket connections