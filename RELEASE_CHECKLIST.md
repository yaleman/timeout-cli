# Release Checklist for cargo-binstall Support

## Pre-Release Setup (✅ Completed)

- [x] **Package Metadata Updated**
  - Version bumped to 0.1.0
  - Repository URLs updated to yaleman/timeout-cli
  - cargo-binstall metadata configured
  - Documentation URL added

- [x] **License and Documentation**  
  - MIT LICENSE file added
  - CHANGELOG.md created with v0.1.0 entries
  - README.md updated with installation methods
  - Version info added to CLI help

- [x] **GitHub Actions Workflows**
  - CI workflow for testing across platforms
  - Release workflow for automated binary builds
  - Security audit and coverage reporting
  - Cross-platform binary targets configured

## Publication Steps

### 1. Push to GitHub Repository

```bash
# Create the repository on GitHub: yaleman/timeout-cli
git remote add origin https://github.com/yaleman/timeout-cli.git
git push -u origin main
```

### 2. Test CI Pipeline

Verify that the CI workflow passes on GitHub Actions for all platforms and Rust versions.

### 3. Publish to crates.io

```bash
# Dry run first
cargo publish --dry-run

# Actual publication
cargo publish
```

### 4. Create Release

```bash
# Tag and push release
git tag v0.1.0
git push origin v0.1.0
```

This will trigger the release workflow which will:
- Build binaries for all supported platforms
- Create GitHub Release with changelog
- Upload binary assets with proper naming for cargo-binstall
- Generate checksums for verification

### 5. Verify cargo-binstall Works

After release artifacts are available:

```bash
cargo binstall timeout-cli
timeout --version
timeout 5 echo "test"
```

## Binary Targets

The release workflow builds for:

- **Linux**: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-unknown-linux-musl
- **macOS**: x86_64-apple-darwin, aarch64-apple-darwin  
- **Windows**: x86_64-pc-windows-msvc

## cargo-binstall Compatibility

The package is configured with:
- Proper naming convention: `timeout-cli-v{version}-{target}.{archive-format}`
- GitHub releases integration via `[package.metadata.binstall]`
- Cross-platform binary distribution
- SHA256 checksums for verification

## Post-Release Verification

1. **Installation Methods**:
   - `cargo binstall timeout-cli` ✓
   - `cargo install timeout-cli` ✓
   - Direct download from GitHub releases ✓

2. **Functionality Testing**:
   - Basic timeout: `timeout 5 echo "test"`
   - Kill-after: `timeout 1 --kill-after 1 sleep 10`
   - Exit codes: `timeout 5 nonexistent_command; echo $?`
   - Help/version: `timeout --help`, `timeout --version`

3. **Cross-Platform**:
   - Linux (various distributions)
   - macOS (Intel and Apple Silicon)
   - Windows (via PowerShell/CMD)

## Maintenance

- **Dependabot**: Automatically updates dependencies
- **CI**: Ensures quality across all platforms
- **Release automation**: Standardized via GitHub Actions
- **Security**: Regular audit checks via GitHub Actions

## Success Criteria

✅ Users can install with: `cargo binstall timeout-cli`  
✅ Fast installation (downloads prebuilt binary vs compiling)  
✅ Cross-platform support (Linux, macOS, Windows)  
✅ Reliable CI/CD pipeline  
✅ Proper versioning and changelog maintenance