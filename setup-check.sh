#!/bin/bash

# Sfeedo Development Environment Check Script
# This script verifies that all required dependencies are installed

echo "🔍 Checking Sfeedo Development Environment..."
echo "=============================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track if all requirements are met
ALL_GOOD=true

# Function to check if command exists
check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "✅ $1: ${GREEN}$(command -v "$1")${NC}"
        if [ "$2" ]; then
            echo -e "   Version: ${GREEN}$($1 $2 2>/dev/null || echo "Unknown")${NC}"
        fi
    else
        echo -e "❌ $1: ${RED}Not found${NC}"
        ALL_GOOD=false
    fi
}

# Function to check Node.js version
check_node_version() {
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version | sed 's/v//')
        MAJOR_VERSION=$(echo $NODE_VERSION | cut -d. -f1)
        if [ "$MAJOR_VERSION" -ge 18 ]; then
            echo -e "✅ Node.js: ${GREEN}v$NODE_VERSION (Compatible)${NC}"
        else
            echo -e "⚠️  Node.js: ${YELLOW}v$NODE_VERSION (Recommend v18+ LTS)${NC}"
        fi
    else
        echo -e "❌ Node.js: ${RED}Not found${NC}"
        ALL_GOOD=false
    fi
}

# Function to check Rust version
check_rust_version() {
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "✅ Rust: ${GREEN}$RUST_VERSION${NC}"
        
        # Check if we have the minimum required version (1.77.2)
        REQUIRED_VERSION="1.77.2"
        if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then
            echo -e "   ${GREEN}Version meets minimum requirement (1.77.2)${NC}"
        else
            echo -e "   ${YELLOW}Consider updating to 1.77.2 or later${NC}"
        fi
    else
        echo -e "❌ Rust: ${RED}Not found${NC}"
        ALL_GOOD=false
    fi
}

# Check system dependencies based on OS
check_system_deps() {
    echo -e "\n📦 System Dependencies:"
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        echo "Detected Linux system"
        
        # Check for webkit2gtk-4.1 (required by Tauri)
        if pkg-config --exists webkit2gtk-4.1; then
            echo -e "✅ webkit2gtk-4.1: ${GREEN}Found${NC}"
        else
            echo -e "❌ webkit2gtk-4.1: ${RED}Not found${NC}"
            echo -e "   Install with: ${YELLOW}sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev${NC} (Ubuntu/Debian)"
            echo -e "   Or: ${YELLOW}sudo dnf install webkit2gtk4.1-devel libsoup3-devel${NC} (Fedora)"
            ALL_GOOD=false
        fi
        
        # Check for additional required libraries
        if pkg-config --exists javascriptcoregtk-4.1; then
            echo -e "✅ javascriptcoregtk-4.1: ${GREEN}Found${NC}"
        else
            echo -e "❌ javascriptcoregtk-4.1: ${RED}Not found${NC}"
            ALL_GOOD=false
        fi
        
        if pkg-config --exists libsoup-3.0; then
            echo -e "✅ libsoup-3.0: ${GREEN}Found${NC}"
        else
            echo -e "❌ libsoup-3.0: ${RED}Not found${NC}"
            ALL_GOOD=false
        fi
        
        # Check for other common dependencies
        check_command "pkg-config"
        check_command "gcc"
        
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        echo "Detected macOS system"
        
        # Check for Xcode Command Line Tools
        if xcode-select -p &> /dev/null; then
            echo -e "✅ Xcode Command Line Tools: ${GREEN}Installed${NC}"
        else
            echo -e "❌ Xcode Command Line Tools: ${RED}Not installed${NC}"
            echo -e "   Install with: ${YELLOW}xcode-select --install${NC}"
            ALL_GOOD=false
        fi
        
        check_command "brew"
    fi
}

# Main checks
echo -e "\n🔧 Core Development Tools:"
check_node_version
check_command "npm" "--version"
check_rust_version
check_command "cargo" "--version"

echo -e "\n🏗️  Build Tools:"
check_command "git" "--version"
check_command "curl" "--version"

# Check if Tauri CLI is installed
echo -e "\n📱 Tauri Tools:"
if npm list -g @tauri-apps/cli &> /dev/null || npm list @tauri-apps/cli &> /dev/null; then
    echo -e "✅ Tauri CLI: ${GREEN}Installed${NC}"
else
    echo -e "⚠️  Tauri CLI: ${YELLOW}Not installed globally${NC}"
    echo -e "   Install with: ${YELLOW}npm install -g @tauri-apps/cli${NC}"
fi

check_system_deps

# Check project dependencies
echo -e "\n📋 Project Status:"
if [ -f "package.json" ]; then
    echo -e "✅ package.json: ${GREEN}Found${NC}"
    if [ -d "node_modules" ]; then
        echo -e "✅ node_modules: ${GREEN}Found${NC}"
    else
        echo -e "⚠️  node_modules: ${YELLOW}Not found - run 'npm install'${NC}"
    fi
else
    echo -e "❌ package.json: ${RED}Not found - are you in the project directory?${NC}"
    ALL_GOOD=false
fi

if [ -f "src-tauri/Cargo.toml" ]; then
    echo -e "✅ Cargo.toml: ${GREEN}Found${NC}"
    if [ -d "src-tauri/target" ]; then
        echo -e "✅ Rust build cache: ${GREEN}Found${NC}"
    else
        echo -e "⚠️  Rust build cache: ${YELLOW}Not found - first build will take longer${NC}"
    fi
else
    echo -e "❌ Cargo.toml: ${RED}Not found - are you in the project directory?${NC}"
    ALL_GOOD=false
fi

# Final summary
echo -e "\n🎯 Summary:"
if [ "$ALL_GOOD" = true ]; then
    echo -e "${GREEN}✅ All requirements met! You're ready to develop Sfeedo.${NC}"
    echo -e "\nNext steps:"
    echo -e "1. Run: ${YELLOW}npm install${NC} (if node_modules not found)"
    echo -e "2. Run: ${YELLOW}npm run tauri dev${NC} to start development"
else
    echo -e "${RED}❌ Some requirements are missing. Please install the missing dependencies.${NC}"
    echo -e "\nRefer to README-DEV-MACOS.md or README-DEV-LINUX.md for detailed setup instructions."
fi

echo -e "\n📚 Documentation:"
echo -e "- macOS setup: README-DEV-MACOS.md"
echo -e "- Linux setup: README-DEV-LINUX.md"