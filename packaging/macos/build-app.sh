#!/usr/bin/env bash
set -euo pipefail

# Usage: ./build-app.sh [BINARY_PATH] [LIBPDFIUM_PATH] [VERSION] [OUTPUT_DIR] [ARCH]
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BINARY_PATH="${1:-$REPO_ROOT/target/release/nixobdo-pdf}"
LIBPDFIUM_PATH="${2:-$REPO_ROOT/libpdfium.dylib}"
VERSION="${3:-}"
OUTPUT_DIR="${4:-$REPO_ROOT/dist}"
ARCH="${5:-$(uname -m)}"

if [ -z "$VERSION" ]; then
    VERSION=$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | cut -d '"' -f2)
fi

case "$ARCH" in
    arm64) ARCH_LABEL="arm64" ;;
    x86_64) ARCH_LABEL="x64" ;;
    *) ARCH_LABEL="$ARCH" ;;
esac

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH" >&2
    exit 1
fi

if [ ! -f "$LIBPDFIUM_PATH" ]; then
    echo "Error: libpdfium.dylib not found at $LIBPDFIUM_PATH" >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

APP_NAME="nixobdo-pdf"
APP_BUNDLE="$OUTPUT_DIR/$APP_NAME.app"
CONTENTS="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

echo "==> Staging macOS app bundle for nixobdo-pdf v$VERSION ($ARCH_LABEL)..."
cp "$BINARY_PATH" "$MACOS_DIR/$APP_NAME"
chmod +x "$MACOS_DIR/$APP_NAME"

# The application looks for the dylib next to the executable first.
cp "$LIBPDFIUM_PATH" "$MACOS_DIR/libpdfium.dylib"
cp "$LIBPDFIUM_PATH" "$RESOURCES_DIR/libpdfium.dylib"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>nixobdo-pdf</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.borneelphukan.nixobdo-pdf</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>nixobdo-pdf</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

LOGO="$REPO_ROOT/assets/logo.png"
if [ -f "$LOGO" ]; then
    echo "==> Building AppIcon.icns..."
    ICON_TMP_DIR="$(mktemp -d)"
    ICONSET_DIR="$ICON_TMP_DIR/AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"
    for size in 16 32 128 256 512; do
        sips -z "$size" "$size" "$LOGO" --out "$ICONSET_DIR/icon_${size}x${size}.png" >/dev/null
        retina=$((size * 2))
        sips -z "$retina" "$retina" "$LOGO" --out "$ICONSET_DIR/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES_DIR/AppIcon.icns"
    rm -rf "$ICON_TMP_DIR"
else
    echo "Warning: $LOGO not found; skipping app icon" >&2
fi

# Ad-hoc signature so Gatekeeper does not treat the bundle as damaged.
# Real distribution still requires an Apple Developer ID + notarization.
echo "==> Ad-hoc signing app bundle..."
codesign --force --deep --sign - "$APP_BUNDLE"
codesign --verify --deep --strict "$APP_BUNDLE"

DMG_PATH="$OUTPUT_DIR/${APP_NAME}-${VERSION}-macos-${ARCH_LABEL}.dmg"
echo "==> Creating $DMG_PATH..."
hdiutil create \
    -volname "$APP_NAME $VERSION" \
    -srcfolder "$APP_BUNDLE" \
    -ov \
    -format UDZO \
    "$DMG_PATH" >/dev/null

echo "==> Wrote $DMG_PATH"
