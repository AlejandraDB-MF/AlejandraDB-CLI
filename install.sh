#!/bin/sh
# AlejandraDB installer
#   curl -fsSL https://raw.githubusercontent.com/AlejandraDB-MF/AlejandraDB-CLI/main/install.sh | sh
set -eu

REPO="AlejandraDB-MF/AlejandraDB-CLI"
BINARY="alejandradb"
VERSION="${1:-latest}"
INSTALL_DIR="${ALEJANDRADB_INSTALL_DIR:-${2:-$HOME/.local/bin}}"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin)
    echo "macOS builds are not published yet." >&2
    echo "Install from source: git clone https://github.com/${REPO}.git && cd AlejandraDB-CLI/adbcli && cargo install --path ." >&2
    exit 1
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "Windows: download alejandradb.exe from https://github.com/${REPO}/releases/latest" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_PART="x86_64" ;;
  aarch64|arm64) ARCH_PART="aarch64" ;;
  *)
    echo "unsupported architecture: $ARCH" >&2
    echo "published builds: linux x86_64, aarch64, windows x86_64" >&2
    exit 1
    ;;
esac

# Default to musl (static) for Linux
TARGET="${ARCH_PART}-unknown-linux-musl"
ASSET="${BINARY}-${TARGET}.tar.gz"

if [ "$VERSION" = "latest" ]; then
  BASE_URL="https://github.com/${REPO}/releases/latest/download"
else
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
fi
BASE_URL="${ALEJANDRADB_RELEASE_URL_BASE:-$BASE_URL}"
URL="${BASE_URL}/${ASSET}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$1" "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$1" "$2"
  else
    echo "curl or wget required" >&2
    exit 1
  fi
}

echo "Downloading $URL"
fetch "$TMPDIR/$ASSET" "$URL"

echo "Verifying checksum"
fetch "$TMPDIR/$ASSET.sha256" "${URL}.sha256"
(cd "$TMPDIR" && sha256sum -c "$ASSET.sha256") || {
  echo "checksum verification FAILED" >&2
  exit 1
}

echo "Extracting"
tar xzf "$TMPDIR/$ASSET" -C "$TMPDIR"

BIN_SRC=$(find "$TMPDIR" -name "$BINARY" -type f | head -n1)
if [ -z "$BIN_SRC" ]; then
  echo "binary not found in archive" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
install -Dm755 "$BIN_SRC" "$INSTALL_DIR/$BINARY"
echo "Installed $BINARY to $INSTALL_DIR/$BINARY"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "$INSTALL_DIR is not in PATH. Add it:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

"$INSTALL_DIR/$BINARY" --version
echo "Run '$BINARY --help' to get started."