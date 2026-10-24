# Release Process

This project uses GitHub Actions to automatically build and release binaries for multiple platforms.

## Supported Platforms

- **Linux**: `.deb` and `.AppImage` packages
- **macOS**: `.dmg` for both Intel (x86_64) and Apple Silicon (aarch64)
- **Windows**: `.msi` and `.exe` (NSIS installer)

## Creating a Release

1. Commit all your changes (the script refuses to run with uncommitted changes)
2. Run the release script from the repository root:
   ```bash
   ./release.sh
   ```
   It bumps the patch version (e.g. `0.1.13` → `0.1.14`) in `package.json`, `package-lock.json`,
   `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and every `tauri.conf.json*` file, commits
   `Release vX.Y.Z`, pushes the branch, then creates and pushes the `vX.Y.Z` tag.
   It stops before changing anything if the branch is behind `origin` or the tag already exists.
3. The pushed tag triggers GitHub Actions, which builds all platforms and creates a draft release
4. Review the draft release and publish when ready

To release manually instead, update the version in all of the files listed above, commit,
then create and push the tag (`git tag vX.Y.Z && git push origin vX.Y.Z`).

## Manual Trigger

You can also trigger the release workflow manually from the Actions tab in GitHub.

## Platform-Specific Configs

- `tauri.conf.json.deb` - Linux builds
- `tauri.conf.json.osx` - macOS builds (both architectures)
- `tauri.conf.json.windows` - Windows builds

The workflow automatically copies the appropriate config before building.
