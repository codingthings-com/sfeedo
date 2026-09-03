#!/bin/bash

# Release script for sfeedo
# This script increments the minor version and handles git operations

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository!"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    print_error "You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

# Get current version from package.json
CURRENT_VERSION=$(grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')
print_status "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

# Increment patch version
NEW_PATCH=$((PATCH + 1))
NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"

print_status "New version: $NEW_VERSION"

# Confirm with user
read -p "Do you want to release version $NEW_VERSION? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Release cancelled."
    exit 0
fi

print_status "Updating version numbers..."

# Update package.json
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" package.json
print_status "Updated package.json"

# Update main tauri.conf.json
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json
print_status "Updated src-tauri/tauri.conf.json"

# Update platform-specific tauri config files
for config_file in src-tauri/tauri.conf.json.deb src-tauri/tauri.conf.json.osx src-tauri/tauri.conf.json.windows; do
    if [ -f "$config_file" ]; then
        # Get current version from this specific file
        PLATFORM_CURRENT_VERSION=$(grep '"version"' "$config_file" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/')
        sed -i "s/\"version\": \"$PLATFORM_CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$config_file"
        print_status "Updated $config_file (was $PLATFORM_CURRENT_VERSION)"
    fi
done

# Update Cargo.toml
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
print_status "Updated src-tauri/Cargo.toml"

# Regenerate Cargo.lock to reflect the new version
(cd src-tauri && cargo generate-lockfile --quiet)
print_status "Updated src-tauri/Cargo.lock"

# Git operations
print_status "Adding files to git..."
git add package.json src-tauri/tauri.conf.json src-tauri/tauri.conf.json.* src-tauri/Cargo.toml src-tauri/Cargo.lock

print_status "Creating commit..."
git commit -m "Release v$NEW_VERSION

- Bump version from $CURRENT_VERSION to $NEW_VERSION
- Updated package.json, tauri.conf.json files, and Cargo.toml"

print_status "Pushing changes..."
git push

print_status "Creating and pushing tag..."
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

print_status "Release v$NEW_VERSION completed successfully!"
print_status "Tag v$NEW_VERSION has been created and pushed to origin."

# Optional: Show the tag info
echo
print_status "Tag information:"
git show --no-patch --format="Tag: %D%nDate: %ad%nAuthor: %an <%ae>%nMessage: %s" "v$NEW_VERSION"