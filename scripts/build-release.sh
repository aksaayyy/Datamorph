#!/bin/bash
# build-release.sh — Build Datamorph binaries for all platforms
# Requires Rustup with cross-compilation targets installed

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔨 Building Datamorph v0.1.0 release binaries..."

# Clean previous builds
cargo clean --release 2>/dev/null || true

# Define targets
TARGETS=(
  "x86_64-unknown-linux-musl"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
)

# Install required targets
echo "📦 Installing cross-compilation targets..."
rustup target add "${TARGETS[@]}" 2>/dev/null || true

# Build for each target
for TARGET in "${TARGETS[@]}"; do
  echo ""
  echo "─────────────────────────────────────"
  echo "Building for: $TARGET"
  echo "─────────────────────────────────────"

  # Strip binary for size (except macOS which may have codesigning issues)
  if [[ "$TARGET" == "x86_64-unknown-linux-musl" ]]; then
    STRIP_CMD="strip"
  else
    STRIP_CMD=""
  fi

  # Build
  if cargo build --release --target "$TARGET"; then
    # Copy binary to release/ with platform-specific name
    case "$TARGET" in
      x86_64-unknown-linux-musl)
        BINARY_NAME="datamorph-linux-amd64"
        cp "target/$TARGET/release/datamorph" "release/$BINARY_NAME"
        $STRIP_CMD "release/$BINARY_NAME" 2>/dev/null || true
        ;;
      x86_64-apple-darwin)
        BINARY_NAME="datamorph-macos-amd64"
        cp "target/$TARGET/release/datamorph" "release/$BINARY_NAME"
        ;;
      aarch64-apple-darwin)
        BINARY_NAME="datamorph-macos-arm64"
        cp "target/$TARGET/release/datamorph" "release/$BINARY_NAME"
        ;;
      x86_64-pc-windows-msvc)
        BINARY_NAME="datamorph-windows-amd64.exe"
        cp "target/$TARGET/release/datamorph.exe" "release/$BINARY_NAME"
        ;;
    esac

    # Verify
    if [ -f "release/$BINARY_NAME" ]; then
      SIZE=$(du -h "release/$BINARY_NAME" | cut -f1)
      echo "✅ Built: release/$BINARY_NAME (${SIZE})"
    else
      echo "❌ Binary not found for $TARGET"
    fi
  else
    echo "❌ Build failed for $TARGET"
    exit 1
  fi
done

echo ""
echo "─────────────────────────────────────"
echo "✅ All binaries built successfully!"
echo "─────────────────────────────────────"
echo ""
echo "Binaries in ./release/ directory:"
ls -lh release/
echo ""
echo "Next: create GitHub release and upload assets"
echo "1. Go to https://github.com/aksaayyy/Datamorph/releases/new"
echo "2. Tag: v0.1.0"
echo "3. Upload all files from release/"
echo "4. Publish"
