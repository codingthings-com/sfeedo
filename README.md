# Sfeedo - Desktop Feed Reader

An RSS/Atom feed reader built with Tauri, Rust, and JavaScript.

## Features

- RSS/Atom feed aggregation
- Native desktop application (Windows, macOS, Linux)
- Fast Rust backend with modern web frontend


## Installation

**Note**: Sfeedo is not a signed application. Your operating system may show a security warning on first launch. You'll need to allow the application to run in your system settings.

### Windows
Download and run the `.msi` installer from the [releases page](https://github.com/codingthings-com/sfeedo/releases).

On first launch, Windows may show "Windows protected your PC". Click "More info" and then **"Run anyway"**.

### macOS
Download the `.dmg` file from the [releases page](https://github.com/codingthings-com/sfeedo/releases), open it, and drag Sfeedo to your Applications folder.

On first launch, macOS will block the app. Go to System Settings > Privacy & Security and click **"Open Anyway"** next to the Sfeedo message.

### Linux
Download the appropriate package for your distribution:
- **Debian/Ubuntu**: `.deb` package
- **AppImage**: Universal Linux package (no installation required)

```bash
# Debian/Ubuntu
sudo dpkg -i sfeedo_*.deb

# AppImage
chmod +x sfeedo_*.AppImage
./sfeedo_*.AppImage
```

## Development

### Prerequisites
Run the setup verification script:
```bash
./setup-check.sh
```

For detailed setup instructions, see:
- **macOS**: [README-DEV-MACOS.md](README-DEV-MACOS.md)
- **Linux**: [README-DEV-LINUX.md](README-DEV-LINUX.md)

### Quick Start
```bash
# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build for production
npm run tauri build
```

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

- **Frontend**: Vite, JavaScript, HTML/CSS
- **Backend**: Rust, Tauri, Tokio
- **Feed Parsing**: [finance-news-aggregator-rs](https://crates.io/crates/finance-news-aggregator-rs)
- **Storage**: JSON-based configuration

## Configuration

Configuration file location:
- **macOS**: `~/Library/Application Support/com.codingthings.sfeedo/config.json`
- **Linux**: `~/.config/com.codingthings.sfeedo/config.json`
- **Windows**: `%APPDATA%\com.codingthings.sfeedo\config.json`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Test on your target platform(s)
4. Submit a pull request

## License

See the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), and [Vite](https://vitejs.dev/).
