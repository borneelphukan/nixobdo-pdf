#!/usr/bin/env bash
set -euo pipefail

# Build a self-contained x86_64 RPM (Fedora, RHEL, openSUSE).
# Usage: ./build-rpm.sh [BINARY_PATH] [LIBPDFIUM_PATH] [VERSION] [OUTPUT_DIR]
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

if ! command -v rpmbuild >/dev/null 2>&1; then
    echo "Error: rpmbuild not found. Install the 'rpm' package (apt install rpm)." >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

TOP_DIR="$(mktemp -d -t nixobdo-rpm-XXXXXX)"
trap 'rm -rf "$TOP_DIR"' EXIT

mkdir -p "$TOP_DIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

echo "==> Staging RPM sources for nixobdo-pdf v$VERSION..."

cp "$BINARY_PATH" "$TOP_DIR/SOURCES/nixobdo-pdf"
cp "$LIBPDFIUM_PATH" "$TOP_DIR/SOURCES/libpdfium.so"
cp "$SCRIPT_DIR/../linux/nixobdo-pdf.desktop" "$TOP_DIR/SOURCES/nixobdo-pdf.desktop"
cp "$REPO_ROOT/LICENSE" "$TOP_DIR/SOURCES/LICENSE"

if [ -f "$REPO_ROOT/assets/icons/logo.svg" ]; then
    cp "$REPO_ROOT/assets/icons/logo.svg" "$TOP_DIR/SOURCES/nixobdo-pdf.svg"
fi
if [ -f "$REPO_ROOT/assets/logo.png" ]; then
    cp "$REPO_ROOT/assets/logo.png" "$TOP_DIR/SOURCES/nixobdo-pdf.png"
fi

sed "s|@VERSION@|$VERSION|g" "$SCRIPT_DIR/nixobdo-pdf.spec.in" > "$TOP_DIR/SPECS/nixobdo-pdf.spec"

echo "==> Building RPM with rpmbuild..."
rpmbuild -bb \
    --define "_topdir $TOP_DIR" \
    --define "_build_id_links none" \
    --target x86_64 \
    "$TOP_DIR/SPECS/nixobdo-pdf.spec"

BUILT_RPM=$(find "$TOP_DIR/RPMS" -name '*.rpm' -type f | head -n 1)
if [ -z "$BUILT_RPM" ]; then
    echo "Error: rpmbuild did not produce an RPM" >&2
    exit 1
fi

cp "$BUILT_RPM" "$OUTPUT_DIR/"
RPM_PATH="$OUTPUT_DIR/$(basename "$BUILT_RPM")"

echo "==> RPM package successfully created at: $RPM_PATH"
