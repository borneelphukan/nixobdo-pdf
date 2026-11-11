#!/usr/bin/env bash
#
# Build a portable tar.gz for a Linux build of nixobdo-pdf.
#
# Environment:
#   VERSION        package version (required)
#   ARCH           packaging architecture, e.g. x86_64 (required)
#   BINARY         path to the release binary (default: target/release/nixobdo-pdf)
#   LIBPDFIUM      path to libpdfium.so (default: ./libpdfium.so)
#   OUT_DIR        output directory (default: dist)
#
# Usage: VERSION=0.1.42 ARCH=x86_64 packaging/linux/build-tarball.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

VERSION="${VERSION:?VERSION is required}"
ARCH="${ARCH:?ARCH is required}"
BINARY="${BINARY:-$ROOT/target/release/nixobdo-pdf}"
LIBPDFIUM="${LIBPDFIUM:-$ROOT/libpdfium.so}"
OUT_DIR="${OUT_DIR:-$ROOT/dist}"

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

[ -f "$BINARY" ] || die "release binary not found at $BINARY"
[ -f "$LIBPDFIUM" ] || die "libpdfium.so not found at $LIBPDFIUM"
BINARY="$(cd -- "$(dirname -- "$BINARY")" && pwd)/$(basename -- "$BINARY")"
LIBPDFIUM="$(cd -- "$(dirname -- "$LIBPDFIUM")" && pwd)/$(basename -- "$LIBPDFIUM")"

NAME="nixobdo-pdf-${VERSION}-linux-${ARCH}"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

DEST="$STAGE/$NAME"
install -d -m 0755 "$DEST" "$DEST/assets/icons"
install -m 0755 "$BINARY" "$DEST/nixobdo-pdf"
# The application dlopen()s libpdfium.so from the directory containing the
# executable, so it must sit next to the binary.
install -m 0644 "$LIBPDFIUM" "$DEST/libpdfium.so"
install -m 0644 "$ROOT/README.md" "$DEST/README.md"
if [ -f "$ROOT/LICENSE" ]; then
    install -m 0644 "$ROOT/LICENSE" "$DEST/LICENSE"
fi
install -m 0755 "$ROOT/packaging/linux/install.sh" "$DEST/install.sh"
install -m 0755 "$ROOT/packaging/linux/uninstall.sh" "$DEST/uninstall.sh"
install -m 0644 "$ROOT/packaging/linux/nixobdo-pdf.desktop" "$DEST/nixobdo-pdf.desktop"
if [ -f "$ROOT/assets/logo.png" ]; then
    install -m 0644 "$ROOT/assets/logo.png" "$DEST/assets/logo.png"
fi
if [ -f "$ROOT/assets/icons/logo.svg" ]; then
    install -m 0644 "$ROOT/assets/icons/logo.svg" "$DEST/assets/icons/logo.svg"
fi

mkdir -p "$OUT_DIR"
ARCHIVE="$OUT_DIR/$NAME.tar.gz"
tar -C "$STAGE" --owner=0 --group=0 --numeric-owner -czf "$ARCHIVE" "$NAME"

printf 'built %s\n' "$ARCHIVE"
