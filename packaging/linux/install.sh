#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "$EUID" -eq 0 ]; then
    PREFIX="/usr/local"
    echo "==> Installing nixobdo-pdf system-wide to $PREFIX..."
else
    PREFIX="$HOME/.local"
    echo "==> Installing nixobdo-pdf for user $(whoami) to $PREFIX..."
fi

BIN_DIR="$PREFIX/bin"
LIB_DIR="$PREFIX/lib/nixobdo-pdf"
APPS_DIR="$PREFIX/share/applications"
ICONS_DIR="$PREFIX/share/icons/hicolor"

mkdir -p "$BIN_DIR" "$LIB_DIR" "$APPS_DIR" "$ICONS_DIR/scalable/apps" "$ICONS_DIR/512x512/apps"

# Install binary and libpdfium
cp "$SCRIPT_DIR/nixobdo-pdf" "$LIB_DIR/nixobdo-pdf"
chmod 755 "$LIB_DIR/nixobdo-pdf"

if [ -f "$SCRIPT_DIR/libpdfium.so" ]; then
    cp "$SCRIPT_DIR/libpdfium.so" "$LIB_DIR/libpdfium.so"
    chmod 644 "$LIB_DIR/libpdfium.so"
fi

ln -sf "$LIB_DIR/nixobdo-pdf" "$BIN_DIR/nixobdo-pdf"

# Install desktop entry
if [ -f "$SCRIPT_DIR/nixobdo-pdf.desktop" ]; then
    cp "$SCRIPT_DIR/nixobdo-pdf.desktop" "$APPS_DIR/nixobdo-pdf.desktop"
    chmod 644 "$APPS_DIR/nixobdo-pdf.desktop"
fi

# Install icons
if [ -f "$SCRIPT_DIR/assets/icons/logo.svg" ]; then
    cp "$SCRIPT_DIR/assets/icons/logo.svg" "$ICONS_DIR/scalable/apps/nixobdo-pdf.svg"
elif [ -f "$SCRIPT_DIR/logo.svg" ]; then
    cp "$SCRIPT_DIR/logo.svg" "$ICONS_DIR/scalable/apps/nixobdo-pdf.svg"
fi

if [ -f "$SCRIPT_DIR/assets/logo.png" ]; then
    cp "$SCRIPT_DIR/assets/logo.png" "$ICONS_DIR/512x512/apps/nixobdo-pdf.png"
elif [ -f "$SCRIPT_DIR/logo.png" ]; then
    cp "$SCRIPT_DIR/logo.png" "$ICONS_DIR/512x512/apps/nixobdo-pdf.png"
fi

# Update desktop & icon caches
if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f "$ICONS_DIR" 2>/dev/null || true
fi

echo "==> Installation complete!"
echo "    Executable: $BIN_DIR/nixobdo-pdf"
if [ "$EUID" -ne 0 ]; then
    echo "    Note: Ensure $BIN_DIR is in your PATH."
fi
echo "    You can launch nixobdo-pdf from your application menu or terminal."
