#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -eq 0 ]; then
    PREFIX="/usr/local"
    echo "==> Uninstalling nixobdo-pdf from system ($PREFIX)..."
else
    PREFIX="$HOME/.local"
    echo "==> Uninstalling nixobdo-pdf for user $(whoami) ($PREFIX)..."
fi

BIN_DIR="$PREFIX/bin"
LIB_DIR="$PREFIX/lib/nixobdo-pdf"
APPS_DIR="$PREFIX/share/applications"
ICONS_DIR="$PREFIX/share/icons/hicolor"

rm -f "$BIN_DIR/nixobdo-pdf"
rm -rf "$LIB_DIR"
rm -f "$APPS_DIR/nixobdo-pdf.desktop"
rm -f "$ICONS_DIR/scalable/apps/nixobdo-pdf.svg"
rm -f "$ICONS_DIR/512x512/apps/nixobdo-pdf.png"

if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPS_DIR" 2>/dev/null || true
fi
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f "$ICONS_DIR" 2>/dev/null || true
fi

echo "==> nixobdo-pdf successfully uninstalled."
