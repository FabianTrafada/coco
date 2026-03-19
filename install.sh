#!/bin/sh
set -e

REPO="FabianTrafada/coco"
BINARY="coco"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
RESET='\033[0m'

info()  { echo "  $1"; }
ok()    { echo "  ${GREEN}✓${RESET} $1"; }
error() { echo "  ${RED}✗${RESET} $1"; exit 1; }

echo ""
echo "  Installing coco 🥥"
echo ""

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  OS="linux" ;;
  Darwin) OS="macos" ;;
  *) error "Unsupported OS: $OS" ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)        ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) error "Unsupported architecture: $ARCH" ;;
esac

info "Detected: $OS/$ARCH"

# Get latest release tag
info "Fetching latest version..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$TAG" ]; then
  error "Could not fetch latest release. Check your internet connection."
fi

info "Latest version: $TAG"

# Build download URL
ARTIFACT="coco-${OS}-${ARCH}"
URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"

# Download
info "Downloading $ARTIFACT..."
curl -fsSL "$URL" -o "/tmp/coco" || error "Download failed. URL: $URL"
chmod +x "/tmp/coco"

# Install
if mv "/tmp/coco" "$INSTALL_DIR/$BINARY" 2>/dev/null; then
  ok "Installed to $INSTALL_DIR/$BINARY"
else
  sudo mv "/tmp/coco" "$INSTALL_DIR/$BINARY"
  ok "Installed to $INSTALL_DIR/$BINARY"
fi

echo ""
ok "coco $TAG installed successfully!"
echo ""
echo "  Run 'coco --help' to get started."
echo ""