# Sfeedo Development Environment Setup - macOS

This guide will help you set up the development environment for Sfeedo on macOS.

## Prerequisites

### 1. Install Homebrew
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 2. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 3. Install Node.js and npm
Using Homebrew:
```bash
brew install node
```

Or using Node Version Manager (recommended):
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install --lts
nvm use --lts
```

Verify installation:
```bash
node --version
npm --version
```

### 4. Install Tauri Prerequisites
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

Note: macOS uses the system WebKit framework, so no additional webkit packages are needed via Homebrew.

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

1. **Rust compilation errors**: Ensure you have the latest Rust version
   ```bash
   rustup update
   ```

2. **Node.js version issues**: Use Node.js LTS version (18.x or 20.x)
   ```bash
   nvm use --lts
   ```

3. **Tauri build fails**: Make sure Xcode Command Line Tools are installed
   ```bash
   xcode-select --install
   ```

4. **Permission issues**: You might need to allow the app in System Preferences > Security & Privacy

### Useful Commands
```bash
# Check Tauri info
npx tauri info

# Clean build artifacts
cargo clean
rm -rf node_modules package-lock.json
npm install

# Update dependencies
cargo update
npm update

# Note on Vite updates:
# Sfeedo currently uses Vite 7.x due to target compatibility (safari13/esbuild).
# If updating Vite, use:
npm install -D vite@^7.3.6
```

## IDE Setup

### Recommended VS Code Extensions
- Rust Analyzer
- Tauri
- ES6 String HTML
- Vite

### Rust Development
For better Rust development experience, consider using:
- VS Code with Rust Analyzer extension
- RustRover (JetBrains)
- Vim/Neovim with rust.vim

