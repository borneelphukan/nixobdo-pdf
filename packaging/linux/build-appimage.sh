#!/usr/bin/env bash
#
# Build an AppImage for a Linux build of nixobdo-pdf.
#
# Environment:
#   VERSION        package version (required)
#   ARCH           x86_64 (required)
#   BINARY         path to the release binary (default: target/release/nixobdo-pdf)
#   LIBPDFIUM      path to libpdfium.so (default: ./libpdfium.so)
#   OUT_DIR        output directory (default: dist)
#   APPIMAGETOOL   path to a pre-downloaded appimagetool (optional)
#
# Usage: VERSION=0.1.42 ARCH=x86_64 packaging/linux/build-appimage.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/nixobdo-pdf}"
LIBPDFIUM="${LIBPDFIUM:-$ROOT/libpdfium.so}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

case "$ARCH" in
    x86_64|aarch64) ;;
    *) die "unsupported AppImage architecture: $ARCH" ;;
esac

[ -f "$BINARY" ] || die "release binary not found at $BINARY"
[ -f "$LIBPDFIUM" ] || die "libpdfium.so not found at $LIBPDFIUM"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"
LIBPDFIUM="$(cd -- "$(dirname -- "$LIBPDFIUM")" && pwd)/$(basename -- "$LIBPDFIUM")"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

APPDIR="$STAGE/AppDir"
install -d -m 0755 \
    "$APPDIR/usr/bin" \
    "$APPDIR/usr/lib/nixobdo-pdf" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/512x512/apps"

install -m 0755 "$BINARY" "$APPDIR/usr/bin/nixobdo-pdf"
# PDFium is loaded from the executable's directory first; usr/lib is a fallback
# for the `./lib/libpdfium.so` probe in the loader.
install -m 0644 "$LIBPDFIUM" "$APPDIR/usr/bin/libpdfium.so"
install -m 0644 "$LIBPDFIUM" "$APPDIR/usr/lib/nixobdo-pdf/libpdfium.so"
install -m 0644 "$ROOT/packaging/linux/nixobdo-pdf.desktop" "$APPDIR/nixobdo-pdf.desktop"
install -m 0644 "$ROOT/packaging/linux/nixobdo-pdf.desktop" \
    "$APPDIR/usr/share/applications/nixobdo-pdf.desktop"
if [ -f "$ROOT/assets/logo.png" ]; then
    install -m 0644 "$ROOT/assets/logo.png" "$APPDIR/nixobdo-pdf.png"
    install -m 0644 "$ROOT/assets/logo.png" \
        "$APPDIR/usr/share/icons/hicolor/512x512/apps/nixobdo-pdf.png"
fi

cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/nixobdo-pdf" "$@"
EOF
chmod 0755 "$APPDIR/AppRun"

TOOL="${APPIMAGETOOL:-}"
if [ -z "$TOOL" ]; then
    TOOL="$STAGE/appimagetool"
    URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    printf 'downloading %s\n' "$URL"
    curl -fsSL "$URL" -o "$TOOL"
    chmod 0755 "$TOOL"
fi

mkdir -p "$OUT_DIR"
OUTPUT="$OUT_DIR/nixobdo-pdf-${VERSION}-${ARCH}.AppImage"

ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$TOOL" --no-appstream "$APPDIR" "$OUTPUT"

printf 'built %s\n' "$OUTPUT"
