#!/usr/bin/env bash
set -euo pipefail

REPO="fwdai/terse"
BINARY="terse"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ── Detect platform ───────────────────────────────────────────────────────────

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

# ── Resolve version ───────────────────────────────────────────────────────────

if [ -z "${VERSION:-}" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
fi

echo "Installing $BINARY $VERSION for $TARGET..."

# ── Download and verify ───────────────────────────────────────────────────────

BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
ARCHIVE="$BINARY-$TARGET.tar.gz"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$BASE_URL/$ARCHIVE"        -o "$TMP/$ARCHIVE"
curl -fsSL "$BASE_URL/$ARCHIVE.sha256" -o "$TMP/$ARCHIVE.sha256"

# Verify checksum
cd "$TMP"
if command -v shasum &>/dev/null; then
  shasum -a 256 -c "$ARCHIVE.sha256"
elif command -v sha256sum &>/dev/null; then
  sha256sum -c "$ARCHIVE.sha256"
else
  echo "Warning: no checksum tool found, skipping verification" >&2
fi
cd - >/dev/null

tar xzf "$TMP/$ARCHIVE" -C "$TMP"

# ── Install ───────────────────────────────────────────────────────────────────

if [ -w "$INSTALL_DIR" ]; then
  install -m 755 "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
else
  echo "Installing to $INSTALL_DIR (sudo required)..."
  sudo install -m 755 "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
fi

echo "$BINARY installed to $INSTALL_DIR/$BINARY"
$INSTALL_DIR/$BINARY --version
