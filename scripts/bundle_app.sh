#!/bin/bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$DIR"

echo "🔨 Cargo release derlemesi yapılıyor..."
source "$HOME/.cargo/env"
cargo build --release --bin macos_screenshot
cargo run --release --bin generate_icon

APP_NAME="ScreenShot"
BUNDLE_DIR="target/$APP_NAME.app"
CONTENTS_DIR="$BUNDLE_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo "📦 .app paket yapısı hazırlanıyor ($BUNDLE_DIR)..."
rm -rf "$BUNDLE_DIR" "$APP_NAME.app"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# 1. Binary kopyalama
cp "target/release/macos_screenshot" "$MACOS_DIR/$APP_NAME"
chmod +x "$MACOS_DIR/$APP_NAME"

# 2. Iconset ve AppIcon.icns üretimi
ICONSET_DIR="icon.iconset"
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

sips -z 16 16     app_icon_1024.png --out "$ICONSET_DIR/icon_16x16.png" > /dev/null
sips -z 32 32     app_icon_1024.png --out "$ICONSET_DIR/icon_16x16@2x.png" > /dev/null
sips -z 32 32     app_icon_1024.png --out "$ICONSET_DIR/icon_32x32.png" > /dev/null
sips -z 64 64     app_icon_1024.png --out "$ICONSET_DIR/icon_32x32@2x.png" > /dev/null
sips -z 128 128   app_icon_1024.png --out "$ICONSET_DIR/icon_128x128.png" > /dev/null
sips -z 256 256   app_icon_1024.png --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null
sips -z 256 256   app_icon_1024.png --out "$ICONSET_DIR/icon_256x256.png" > /dev/null
sips -z 512 512   app_icon_1024.png --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null
sips -z 512 512   app_icon_1024.png --out "$ICONSET_DIR/icon_512x512.png" > /dev/null
sips -z 1024 1024 app_icon_1024.png --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null

iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES_DIR/AppIcon.icns"
rm -rf "$ICONSET_DIR" app_icon_1024.png

# 3. Info.plist
cat << 'EOF' > "$CONTENTS_DIR/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ScreenShot</string>
    <key>CFBundleDisplayName</key>
    <string>ScreenShot</string>
    <key>CFBundleIdentifier</key>
    <string>com.bysinbin.macos-screenshot</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleExecutable</key>
    <string>ScreenShot</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
EOF

# 4. Temizle ve Kod İmzalama
xattr -cr "$BUNDLE_DIR" 2>/dev/null || true
dot_clean "$BUNDLE_DIR" 2>/dev/null || true
if security find-certificate -c "ScreenShot Dev" >/dev/null 2>&1; then
    echo "🔏 'ScreenShot Dev' kalıcı geliştirici sertifikası ile imzalanıyor..."
    codesign --force --deep --sign "ScreenShot Dev" "$BUNDLE_DIR"
else
    echo "🔏 Ad-hoc imza uygulanıyor..."
    codesign --force --deep --sign - "$BUNDLE_DIR"
fi

cp -R "$BUNDLE_DIR" "$APP_NAME.app"

echo "🎉 Tebrikler! $APP_NAME.app paketi başarıyla oluşturuldu:"
echo "   -> $DIR/$APP_NAME.app"
