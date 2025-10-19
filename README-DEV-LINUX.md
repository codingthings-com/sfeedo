# Sfeedo Development Environment Setup - Linux

This guide will help you set up the development environment for Sfeedo on Linux distributions.

## Prerequisites

### 1. Update System Packages
```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y

# Fedora
sudo dnf update -y

# Arch Linux
sudo pacman -Syu
```

### 2. Install System Dependencies

#### Ubuntu/Debian
```bash
sudo apt install -y \
    curl \
    wget \
    file \
    build-essential \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    pkg-config
```

#### Fedora
```bash
sudo dnf install -y \
    curl \
    wget \
    file \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    webkit2gtk4.1-devel \
    libsoup3-devel \
    pkg-config
```

#### Arch Linux
```bash
sudo pacman -S --needed \
    curl \
    wget \
    file \
    base-devel \
    openssl \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    webkit2gtk-4.1 \
    libsoup3 \
    pkg-config
```

### 3. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 4. Install Node.js and npm

#### Using Package Manager (Ubuntu/Debian)
```bash
# Install Node.js LTS
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs
```

#### Using Node Version Manager (Recommended)
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install --lts
nvm use --lts
```

#### Fedora
```bash
sudo dnf install -y nodejs npm
```

#### Arch Linux
```bash
sudo pacman -S nodejs npm
```

Verify installation:
```bash
node --version
npm --version
```

## Project Setup

### 1. Clone and Navigate to Project
```bash
git clone <your-repo-url>
cd sfeedo
```

### 2. Install Node.js Dependencies
```bash
npm install
```

### 3. Install Tauri CLI
```bash
npm install -g @tauri-apps/cli
# or locally in the project
npm install --save-dev @tauri-apps/cli
```

### 4. Build Rust Dependencies
```bash
cd src-tauri
cargo build
cd ..
```

## Development Workflow

### Start Development Server
```bash
# Start the frontend development server and Tauri app
npm run tauri dev
```

### Build for Production
```bash
# Build the application
npm run tauri build
```

### Frontend Only Development
```bash
# Start only the Vite development server
npm run dev
```

## Troubleshooting

### Common Issues

1. **Missing system dependencies**: Install the webkit2gtk-4.1 and other required packages
   ```bash
   # Ubuntu/Debian
   sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
   
   # Fedora
   sudo dnf install webkit2gtk4.1-devel libsoup3-devel
   ```

2. **Rust compilation errors**: Ensure you have the latest Rust version
   ```bash
   rustup update
   ```

3. **Node.js version issues**: Use Node.js LTS version (18.x or 20.x)
   ```bash
   nvm use --lts
   ```

4. **Permission issues with global npm packages**:
   ```bash
   # Configure npm to use a different directory for global packages
   mkdir ~/.npm-global
   npm config set prefix '~/.npm-global'
   echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
   source ~/.bashrc
   ```

5. **AppImage execution issues**:
   ```bash
   # Make the built AppImage executable
   chmod +x src-tauri/target/release/bundle/appimage/sfeedo_*.AppImage
   ```

### Distribution-Specific Issues

#### Ubuntu/Debian
- If you encounter webkit2gtk issues, ensure you have the 4.1 version:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev
  ```
- For older Ubuntu versions (< 22.04), you might need to add a PPA or use webkit2gtk-4.0

#### Fedora
- For older Fedora versions, you might need:
  ```bash
  sudo dnf install webkit2gtk4.1-devel libsoup3-devel
  ```

#### Arch Linux
- If webkit2gtk is not found:
  ```bash
  sudo pacman -S webkit2gtk-4.1 libsoup3
  ```

### Useful Commands
```bash
# Check Tauri info and system compatibility
npx tauri info

# Clean build artifacts
cargo clean
rm -rf node_modules
npm install

# Update dependencies
cargo update
npm update

# Check system dependencies
ldd --version
pkg-config --version
```

## IDE Setup

### Recommended VS Code Extensions
- Rust Analyzer
- Tauri
- ES6 String HTML
- Vite

### Alternative IDEs
- **Rust Development**: RustRover (JetBrains), Vim/Neovim with rust.vim
- **Frontend Development**: WebStorm, Sublime Text, Atom

## Building for Distribution

### AppImage (Recommended for Linux)
The default build creates an AppImage that works across most Linux distributions:
```bash
npm run tauri build
```

The built AppImage will be located at:
```
src-tauri/target/release/bundle/appimage/sfeedo_*.AppImage
```

### Debian Package
To build a .deb package:
```bash
# Add to src-tauri/tauri.conf.json under "bundle"
"targets": ["deb"]
```

### RPM Package
To build an .rpm package:
```bash
# Add to src-tauri/tauri.conf.json under "bundle"
"targets": ["rpm"]
```



## Performance Tips

1. **Use release builds for testing**: `cargo build --release`
2. **Enable LTO for smaller binaries**: Add to Cargo.toml:
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   ```
3. **Use system libraries when possible**: Consider using system-provided libraries instead of bundled ones for smaller builds