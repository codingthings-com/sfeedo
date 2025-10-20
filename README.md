# Sfeedo - Desktop Feed Reader

A modern RSS/Atom feed aggregator built with Tauri, Rust, and JavaScript. Sfeedo provides a clean, fast, and native desktop experience for managing and reading your favorite feeds.

## Features

- 📰 RSS/Atom feed aggregation
- 🖥️ Native desktop application (Windows, macOS, Linux)
- ⚡ Fast and lightweight Rust backend
- 🎨 Modern web-based frontend
- 📄 JSON-based configuration storage
- 🔄 Automatic feed refresh management
- 📱 Cross-platform compatibility

## Quick Start

### Prerequisites Check
Run the setup verification script to ensure your environment is ready:
```bash
./setup-check.sh
```

### Development
```bash
# Install dependencies (if not already done)
npm install

# Start development server
npm run tauri dev
```

### Build for Production
```bash
npm run tauri build
```

## Development Environment Setup

Choose your operating system for detailed setup instructions:

- **macOS**: See [README-DEV-MACOS.md](README-DEV-MACOS.md)
- **Linux**: See [README-DEV-LINUX.md](README-DEV-LINUX.md)

## Project Structure

```
sfeedo/
├── src/                    # Frontend source files
│   └── main.js            # Main JavaScript entry point
├── src-tauri/             # Rust backend source
│   ├── src/               # Rust source files
│   │   ├── commands/      # Tauri command handlers
│   │   ├── config/        # Configuration management
│   │   ├── models/        # Data models
│   │   ├── services/      # Business logic services
│   │   ├── feed_aggregator.rs
│   │   ├── feed_manager.rs
│   │   ├── refresh_manager.rs
│   │   └── main.rs        # Rust entry point
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── dist/                  # Built frontend assets
├── package.json           # Node.js dependencies
├── vite.config.js         # Vite bundler configuration
└── index.html             # Main HTML template
```

## Technology Stack

### Frontend
- **Vite** - Fast build tool and development server
- **JavaScript** - Core frontend logic
- **HTML/CSS** - User interface

### Backend
- **Rust** - High-performance system programming language
- **Tauri** - Framework for building desktop apps with web technologies
- **JSON** - Configuration and feed source storage
- **Tokio** - Asynchronous runtime for Rust

### Key Dependencies
- `@tauri-apps/api` - Tauri JavaScript API
- `finance-news-aggregator-rs` - RSS/Atom feed parsing
- `reqwest` - HTTP client for Rust
- `chrono` - Date and time handling
- `serde` - Serialization framework

## Available Scripts

```bash
# Development
npm run dev          # Start Vite development server only
npm run tauri dev    # Start full Tauri development environment

# Building
npm run build        # Build frontend assets
npm run tauri build  # Build complete desktop application

# Preview
npm run preview      # Preview built frontend assets
```

## Configuration

### Tauri Configuration
Main configuration is in `src-tauri/tauri.conf.json`:
- App metadata (name, version, identifier)
- Build settings (frontend dist path, dev URL)
- Bundle configuration for different platforms

### Vite Configuration
Frontend build configuration in `vite.config.js`:
- Development server settings
- Build optimization
- Platform-specific targets

## Development Workflow

1. **Start Development**: `npm run tauri dev`
   - Launches Vite dev server on http://localhost:5173
   - Compiles and runs Rust backend
   - Opens desktop application window

2. **Make Changes**:
   - Frontend changes auto-reload via Vite HMR
   - Rust changes require restart of dev command

3. **Test Build**: `npm run tauri build`
   - Creates optimized production build
   - Generates platform-specific installers

## Troubleshooting

### Common Issues

1. **Environment Setup**: Run `./setup-check.sh` to verify all dependencies
2. **Build Failures**: Check that all system dependencies are installed
3. **Port Conflicts**: Vite uses port 5173 by default (configurable in vite.config.js)

### Getting Help

1. Check the platform-specific README files for detailed setup instructions
2. Verify your environment with the setup check script
3. Ensure all dependencies are up to date

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly on your target platform(s)
5. Submit a pull request

## License

See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) framework
- Uses [finance-news-aggregator-rs](https://crates.io/crates/finance-news-aggregator-rs) for feed parsing
- Powered by [Rust](https://www.rust-lang.org/) and [Vite](https://vitejs.dev/)


## Notes


```bash
~/Library/Application\ Support/com.codingthings.sfeedo/config.json


# or 

~/.config/.... 
```