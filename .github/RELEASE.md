# Release Process

This project uses GitHub Actions to automatically build and release binaries for multiple platforms.

## Supported Platforms

- **Linux**: `.deb` and `.AppImage` packages
- **macOS**: `.dmg` for both Intel (x86_64) and Apple Silicon (aarch64)
- **Windows**: `.msi` and `.exe` (NSIS installer)

## Creating a Release

1. Update the version in `package.json` and `src-tauri/Cargo.toml`
2. Update the version in all `tauri.conf.json.*` files
3. Commit your changes
4. Create and push a tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
5. GitHub Actions will automatically build all platforms and create a draft release
6. Review the draft release and publish when ready

## Manual Trigger

You can also trigger the release workflow manually from the Actions tab in GitHub.

## Platform-Specific Configs

- `tauri.conf.json.deb` - Linux builds
- `tauri.conf.json.osx` - macOS builds (both architectures)
- `tauri.conf.json.windows` - Windows builds

The workflow automatically copies the appropriate config before building.
