# Justfile for timeout-cli project

# Default recipe - show available commands
default:
    @just --list

# Run all tests
test:
    cargo test --all-features

# Run tests with verbose output
test-verbose:
    cargo test --all-features -- --nocapture

# Check code formatting and linting
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Run security audit
audit:
    @command -v cargo-audit >/dev/null || (echo "Installing cargo-audit..." && cargo install cargo-audit)
    cargo audit

# Generate and upload code coverage
coverage:
    #!/bin/bash
    echo "🧪 Generating code coverage..."
    
    # Check if cargo-tarpaulin is installed
    if ! command -v cargo-tarpaulin >/dev/null; then
        echo "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin
    fi
    
    # Upload to Coveralls if token is available
    if [[ -n "$COVERALLS_REPO_TOKEN" ]]; then
        echo "📊 Generating coverage and uploading to Coveralls..."
        cargo tarpaulin --coveralls "$COVERALLS_REPO_TOKEN"
        echo "✅ Coverage uploaded to Coveralls"
    else
        echo "⚠️  COVERALLS_REPO_TOKEN not set - generating local coverage only"
        cargo tarpaulin
    fi

# Build in release mode
build-release:
    cargo build --release

# Clean build artifacts
clean:
    cargo clean

# Show current version
version:
    @echo "Current version: $(just _get-current-version)"

# Dry run cargo publish
publish-dry:
    cargo publish --dry-run

# Update version in Cargo.toml and run full release process
release version: (_check-clean) (_validate-version version) (_update-version version) (_test-and-check) (_publish-and-tag version)

# Internal: Check if working directory is clean
_check-clean:
    #!/bin/bash
    if [[ -n $(git status --porcelain) ]]; then
        echo "❌ Working directory is not clean. Please commit or stash changes first."
        git status --short
        exit 1
    fi
    echo "✅ Working directory is clean"

# Internal: Get current version from Cargo.toml
_get-current-version:
    @grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\([^"]*\)"/\1/'

# Internal: Validate new version is higher than current
_validate-version version:
    #!/bin/bash
    current_version=$(just _get-current-version)
    echo "📊 Current version: $current_version"
    echo "📊 New version: {{version}}"
    
    # Function to compare semantic versions
    version_compare() {
        if [[ $1 == $2 ]]; then
            return 1  # Equal
        fi
        
        local IFS=.
        local i ver1=($1) ver2=($2)
        
        # Fill empty fields with zeros
        for ((i=${#ver1[@]}; i<${#ver2[@]}; i++)); do
            ver1[i]=0
        done
        for ((i=${#ver2[@]}; i<${#ver1[@]}; i++)); do
            ver2[i]=0
        done
        
        for ((i=0; i<${#ver1[@]}; i++)); do
            if [[ -z ${ver2[i]} ]]; then
                ver2[i]=0
            fi
            if ((10#${ver1[i]} > 10#${ver2[i]})); then
                return 2  # ver1 > ver2
            fi
            if ((10#${ver1[i]} < 10#${ver2[i]})); then
                return 0  # ver1 < ver2
            fi
        done
        return 1  # Equal
    }
    
    version_compare "$current_version" "{{version}}"
    result=$?
    
    case $result in
        0)
            echo "✅ Version {{version}} is higher than current version $current_version"
            ;;
        1)
            echo "❌ Version {{version}} is the same as current version $current_version"
            echo "   Please use a higher version number"
            exit 1
            ;;
        2)
            echo "❌ Version {{version}} is lower than current version $current_version"
            echo "   Please use a higher version number"
            exit 1
            ;;
    esac
    
    # Also check if version already exists as a git tag
    if git rev-parse "v{{version}}" >/dev/null 2>&1; then
        echo "❌ Git tag v{{version}} already exists"
        echo "   Choose a different version number"
        exit 1
    fi
    
    # Check if version exists on crates.io
    echo "🔍 Checking if version {{version}} exists on crates.io..."
    if curl -s "https://crates.io/api/v1/crates/timeout-cli" | jq -e ".versions[] | select(.num == \"{{version}}\")" >/dev/null 2>&1; then
        echo "❌ Version {{version}} already exists on crates.io"
        echo "   Choose a different version number"
        exit 1
    fi
    
    echo "✅ Version {{version}} is valid for release"

# Internal: Update version in Cargo.toml
_update-version version:
    #!/bin/bash
    echo "📝 Updating version to {{version}}"
    
    # Update Cargo.toml
    sed -i.bak 's/^version = "[^"]*"/version = "{{version}}"/' Cargo.toml
    rm Cargo.toml.bak
    
    # Update Cargo.lock
    cargo check --quiet
    
    # Stage the version changes
    git add Cargo.toml Cargo.lock
    git commit -m "Bump version to {{version}}"
    
    echo "✅ Version updated to {{version}}"

# Internal: Run tests and checks before release
_test-and-check:
    #!/bin/bash
    echo "🧪 Running tests and checks..."
    
    # Run all tests
    echo "  → Running tests..."
    cargo test --all-features --quiet
    
    # Check formatting
    echo "  → Checking code formatting..."
    cargo fmt --check
    
    # Check linting
    echo "  → Running clippy..."
    cargo clippy --all-targets --all-features --quiet -- -D warnings
    
    # Security audit
    echo "  → Running security audit..."
    cargo audit --quiet
    
    # Dry run publish
    echo "  → Testing cargo publish..."
    cargo publish --dry-run --quiet
    
    echo "✅ All checks passed"

# Internal: Publish to crates.io and create git tag
_publish-and-tag version:
    #!/bin/bash
    echo "🚀 Publishing and tagging release {{version}}"
    
    # Push version commit first
    echo "  → Pushing version commit to GitHub..."
    git push origin main
    
    # Publish to crates.io
    echo "  → Publishing to crates.io..."
    cargo publish
    
    # Wait a moment for crates.io to process
    sleep 5
    
    # Create and push tag
    echo "  → Creating and pushing tag v{{version}}..."
    git tag "v{{version}}"
    git push origin "v{{version}}"
    
    echo "✅ Release {{version}} published successfully!"
    echo ""
    echo "📦 The GitHub Actions release workflow will now:"
    echo "   • Build binaries for all platforms"
    echo "   • Create GitHub release with changelog"
    echo "   • Upload binary assets for cargo-binstall"
    echo ""
    echo "🎉 Users can install with: cargo binstall timeout-cli"

# Quick release command that prompts for version
release-interactive:
    #!/bin/bash
    echo "🚀 Interactive Release Process"
    echo ""
    
    # Get current version from Cargo.toml
    current_version=$(just _get-current-version)
    echo "Current version: $current_version"
    echo ""
    
    # Suggest next versions
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    major=${VERSION_PARTS[0]}
    minor=${VERSION_PARTS[1]}
    patch=${VERSION_PARTS[2]}
    
    next_patch="$major.$minor.$((patch + 1))"
    next_minor="$major.$((minor + 1)).0"
    next_major="$((major + 1)).0.0"
    
    echo "Suggested versions:"
    echo "  1. Patch: $next_patch (bug fixes)"
    echo "  2. Minor: $next_minor (new features, backwards compatible)"
    echo "  3. Major: $next_major (breaking changes)"
    echo ""
    
    read -p "Enter new version (1/2/3 or custom version, Enter for patch): " choice
    
    case "$choice" in
        "1" | "")
            new_version="$next_patch"
            ;;
        "2")
            new_version="$next_minor"
            ;;
        "3")
            new_version="$next_major"
            ;;
        *)
            new_version="$choice"
            ;;
    esac
    
    echo ""
    echo "🎯 Selected version: $new_version"
    echo ""
    
    # Show what will happen
    echo "This will:"
    echo "  ✓ Validate version is higher than current ($current_version)"
    echo "  ✓ Update Cargo.toml to version $new_version"
    echo "  ✓ Run full test suite and checks"
    echo "  ✓ Publish to crates.io"
    echo "  ✓ Create and push git tag v$new_version"
    echo "  ✓ Trigger GitHub Actions for binary builds"
    echo ""
    
    read -p "Continue with release? (y/N): " confirm
    
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        just release "$new_version"
    else
        echo "❌ Release cancelled"
        exit 1
    fi

# Check release status after tagging
check-release version:
    #!/bin/bash
    echo "📊 Checking release status for v{{version}}"
    echo ""
    
    # Check if tag exists
    if git rev-parse "v{{version}}" >/dev/null 2>&1; then
        echo "✅ Git tag v{{version}} exists"
    else
        echo "❌ Git tag v{{version}} not found"
        exit 1
    fi
    
    # Check crates.io
    echo "🔍 Checking crates.io..."
    if curl -s "https://crates.io/api/v1/crates/timeout-cli" | jq -e ".versions[] | select(.num == \"{{version}}\")"; then
        echo "✅ Version {{version}} published on crates.io"
    else
        echo "⏳ Version {{version}} not yet available on crates.io (may take a few minutes)"
    fi
    
    # Check GitHub release
    echo "🔍 Checking GitHub release..."
    if curl -s "https://api.github.com/repos/yaleman/timeout-cli/releases/tags/v{{version}}" | jq -e ".tag_name"; then
        echo "✅ GitHub release v{{version}} created"
        echo "🔗 https://github.com/yaleman/timeout-cli/releases/tag/v{{version}}"
    else
        echo "⏳ GitHub release v{{version}} not yet created (Actions may still be running)"
        echo "🔗 Check: https://github.com/yaleman/timeout-cli/actions"
    fi

# Test cargo-binstall installation (after release)
test-binstall:
    #!/bin/bash
    echo "🧪 Testing cargo-binstall installation"
    echo ""
    
    # Create temporary directory
    temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    echo "  → Installing timeout-cli via cargo-binstall..."
    if cargo binstall timeout-cli --no-confirm; then
        echo "✅ Installation successful"
        
        echo "  → Testing basic functionality..."
        timeout --version
        timeout 1 echo "Hello from timeout-cli!"
        
        echo "✅ cargo-binstall installation working correctly!"
    else
        echo "❌ cargo-binstall installation failed"
        exit 1
    fi
    
    # Cleanup
    cd - > /dev/null
    rm -rf "$temp_dir"

# Show current project status
status:
    #!/bin/bash
    echo "📊 timeout-cli Project Status"
    echo "=============================="
    echo ""
    
    # Version info
    version=$(just _get-current-version)
    echo "📦 Current version: $version"
    
    # Git status
    echo "📝 Git status:"
    if [[ -n $(git status --porcelain) ]]; then
        echo "   ⚠️  Working directory has changes"
        git status --short | sed 's/^/   /'
    else
        echo "   ✅ Working directory clean"
    fi
    
    # Last commit
    echo "🔀 Last commit: $(git log -1 --pretty=format:'%h %s (%cr)')"
    
    # Remote status
    echo "🌐 Remote status:"
    if git remote get-url origin &>/dev/null; then
        echo "   📍 Origin: $(git remote get-url origin)"
        
        # Check if we're ahead/behind
        git fetch origin main --quiet
        ahead=$(git rev-list --count origin/main..HEAD 2>/dev/null || echo "0")
        behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
        
        if [[ "$ahead" -gt 0 ]]; then
            echo "   ⬆️  $ahead commits ahead of origin/main"
        fi
        if [[ "$behind" -gt 0 ]]; then
            echo "   ⬇️  $behind commits behind origin/main"
        fi
        if [[ "$ahead" -eq 0 && "$behind" -eq 0 ]]; then
            echo "   ✅ Up to date with origin/main"
        fi
    else
        echo "   ❌ No remote origin configured"
    fi
    
    echo ""
    echo "🚀 Ready to release? Run: just release-interactive"