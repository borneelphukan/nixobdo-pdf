#!/usr/bin/env bash
set -euo pipefail

# Usage: ./build-deb.sh [BINARY_PATH] [LIBPDFIUM_PATH] [VERSION] [OUTPUT_DIR]
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BINARY_PATH="${1:-$REPO_ROOT/target/release/nixobdo-pdf}"
LIBPDFIUM_PATH="${2:-$REPO_ROOT/libpdfium.so}"
VERSION="${3:-}"
OUTPUT_DIR="${4:-$REPO_ROOT/dist}"

if [ -z "$VERSION" ]; then
    VERSION=$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | cut -d '"' -f2)
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH" >&2
    exit 1
fi

if [ ! -f "$LIBPDFIUM_PATH" ]; then
    echo "Error: libpdfium.so not found at $LIBPDFIUM_PATH" >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

STAGE_DIR="$(mktemp -d -t nixobdo-deb-XXXXXX)"
trap 'rm -rf "$STAGE_DIR"' EXIT

echo "==> Staging Debian package structure for nixobdo-pdf v$VERSION..."

# Directory structure
PKG_USR="$STAGE_DIR/usr"
PKG_LIB="$PKG_USR/lib/nixobdo-pdf"
PKG_BIN="$PKG_USR/bin"
PKG_APPS="$PKG_USR/share/applications"
PKG_ICONS="$PKG_USR/share/icons/hicolor"
PKG_DEBIAN="$STAGE_DIR/DEBIAN"

mkdir -p "$PKG_LIB" "$PKG_BIN" "$PKG_APPS" "$PKG_ICONS/scalable/apps" "$PKG_ICONS/512x512/apps" "$PKG_DEBIAN"

# Copy binary & libpdfium
cp "$BINARY_PATH" "$PKG_LIB/nixobdo-pdf"
cp "$LIBPDFIUM_PATH" "$PKG_LIB/libpdfium.so"
ln -sf /usr/lib/nixobdo-pdf/nixobdo-pdf "$PKG_BIN/nixobdo-pdf"

# Copy desktop file and icons
cp "$SCRIPT_DIR/nixobdo-pdf.desktop" "$PKG_APPS/nixobdo-pdf.desktop"
if [ -f "$REPO_ROOT/assets/icons/logo.svg" ]; then
    cp "$REPO_ROOT/assets/icons/logo.svg" "$PKG_ICONS/scalable/apps/nixobdo-pdf.svg"
fi
if [ -f "$REPO_ROOT/assets/logo.png" ]; then
    cp "$REPO_ROOT/assets/logo.png" "$PKG_ICONS/512x512/apps/nixobdo-pdf.png"
fi

# Calculate Installed-Size in KiB
INSTALLED_SIZE=$(du -sk "$STAGE_DIR" | cut -f1)

# Generate control file
cat << EOF > "$PKG_DEBIAN/control"
Package: nixobdo-pdf
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Borneel Bikash Phukan <borneelphukan@gmail.com>
Installed-Size: $INSTALLED_SIZE
Depends: libc6, libgtk-3-0 | libgtk-3-0t64, libxcb1, libxcb-render0, libxcb-shape0, libxcb-xfixes0, libxkbcommon0
Homepage: https://borneelphukan.github.io/nixobdo-pdf/
Description: Distraction-free PDF viewer
 nixobdo-pdf is an open-source PDF viewer designed to provide a clean,
 distraction-free PDF experience reminiscent of the classic Adobe PDF reader.
 Features include multi-tab viewing, annotation tools (highlight, underline,
 strikethrough, redact, text), digital signatures, search, and file conversion.
EOF

# Post-install script
cat << 'EOF' > "$PKG_DEBIAN/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
EOF

# Post-remove script
cat << 'EOF' > "$PKG_DEBIAN/postrm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi
EOF

# Set permissions
find "$STAGE_DIR" -type d -exec chmod 755 {} +
find "$STAGE_DIR" -type f -exec chmod 644 {} +
chmod 755 "$PKG_LIB/nixobdo-pdf"
chmod 755 "$PKG_DEBIAN"
chmod 755 "$PKG_DEBIAN/postinst" "$PKG_DEBIAN/postrm"

DEB_NAME="nixobdo-pdf_${VERSION}_amd64.deb"
DEB_PATH="$OUTPUT_DIR/$DEB_NAME"

echo "==> Building .deb package with dpkg-deb..."
dpkg-deb --build --root-owner-group "$STAGE_DIR" "$DEB_PATH"

echo "==> Debian package successfully created at: $DEB_PATH"
