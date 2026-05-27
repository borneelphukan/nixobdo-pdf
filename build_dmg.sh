#!/bin/bash
set -e

APP_NAME="PDFViewer"
APP_DIR="${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "Building release binary..."
cargo build --release

echo "Creating .app bundle structure..."
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

echo "Copying executable and libraries..."
cp target/release/PDFViewer "${MACOS_DIR}/"
cp lib/libpdfium.dylib "${MACOS_DIR}/"

echo "Creating Info.plist..."
cat <<EOF > "${CONTENTS_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.borneelphukan.pdfviewer</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
</dict>
</plist>
EOF

echo "Generating macOS App Icon..."
ICONSET_DIR="AppIcon.iconset"
mkdir -p "${ICONSET_DIR}"
# Copy our 512x512 logo.png to various sizes required by macOS
cp assets/logo.png "${ICONSET_DIR}/icon_512x512.png"
sips -z 16 16 assets/logo.png --out "${ICONSET_DIR}/icon_16x16.png"
sips -z 32 32 assets/logo.png --out "${ICONSET_DIR}/icon_16x16@2x.png"
sips -z 32 32 assets/logo.png --out "${ICONSET_DIR}/icon_32x32.png"
sips -z 64 64 assets/logo.png --out "${ICONSET_DIR}/icon_32x32@2x.png"
sips -z 128 128 assets/logo.png --out "${ICONSET_DIR}/icon_128x128.png"
sips -z 256 256 assets/logo.png --out "${ICONSET_DIR}/icon_128x128@2x.png"
sips -z 256 256 assets/logo.png --out "${ICONSET_DIR}/icon_256x256.png"
sips -z 512 512 assets/logo.png --out "${ICONSET_DIR}/icon_256x256@2x.png"
# 1024x1024
sips -z 1024 1024 assets/logo.png --out "${ICONSET_DIR}/icon_512x512@2x.png"
iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES_DIR}/AppIcon.icns"
rm -rf "${ICONSET_DIR}"

echo "Creating .dmg file..."
rm -f "${APP_NAME}.dmg"
# Create a temporary folder to act as the DMG root
TMP_DMG_DIR="dmg_root"
rm -rf "${TMP_DMG_DIR}"
mkdir -p "${TMP_DMG_DIR}"
cp -R "${APP_DIR}" "${TMP_DMG_DIR}/"
ln -s /Applications "${TMP_DMG_DIR}/Applications"

hdiutil create -volname "${APP_NAME}" -srcfolder "${TMP_DMG_DIR}" -ov -format UDZO "${APP_NAME}.dmg"

rm -rf "${TMP_DMG_DIR}"
rm -rf "${APP_DIR}"

echo "Successfully created ${APP_NAME}.dmg!"
