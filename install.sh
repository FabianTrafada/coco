#!/bin/sh
set -eu

red="$((/usr/bin/tput bold || :; /usr/bin/tput setaf 1 || :) 2>&-)"
plain="$((/usr/bin/tput sgr0 || :) 2>&-)"

status()  { echo ">>> $*"; }
error()   { echo "${red}ERROR:${plain} $*"; exit 1; }
warning() { echo "${red}WARNING:${plain} $*"; }

REPO="FabianTrafada/coco"
INSTALL_DIR="/usr/local/bin"

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

# Sudo
SUDO=
if [ "$(id -u)" -ne 0 ]; then
  if ! command -v sudo >/dev/null; then
    error "This script requires superuser permissions. Please re-run as root."
  fi
  SUDO="sudo"
fi

# Get latest release tag
status "Fetching latest version..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

[ -z "$TAG" ] && error "Could not fetch latest version. Check your internet connection."

# Download
ARTIFACT="coco-${OS}-${ARCH}"
URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"

status "Downloading coco $TAG..."
curl --fail --show-error --location --progress-bar "$URL" -o "/tmp/coco" \
  || error "Download failed: $URL"
chmod +x "/tmp/coco"

# Install
status "Installing coco to $INSTALL_DIR..."
$SUDO mv "/tmp/coco" "$INSTALL_DIR/coco"

status "Install complete. Run 'coco --help' to get started."