#!/bin/zsh
# Builds the game for iOS, packages it as a signed .app and runs it on a connected iPhone.
#
# Signing reuses the provisioning profile Xcode creates for BUNDLE_ID: make any Xcode iOS app
# with that bundle identifier and run it on the phone once (free profiles last 7 days).
#
# Usage: ./build_ios.sh [--no-run]
# Overrides: BUNDLE_ID, SIGN_IDENTITY, DEVICE (UDID from `xcrun devicectl list devices`).
set -euo pipefail
cd "${0:A:h}"

BUNDLE_ID="${BUNDLE_ID:-com.ryanbrownswell.FlappyBird}"
APP_NAME="FlappyBird"
TARGET="aarch64-apple-ios"
APP="target/ios/$APP_NAME.app"
PROFILES_DIR="$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles"

# Desktop-only features (dynamic linking, file watcher) don't work on iOS.
cargo build --release --target "$TARGET" --no-default-features

rm -rf "$APP"
mkdir -p "$APP"
# UIKit kills apps linked against the iOS 27+ SDK that don't adopt the UIScene lifecycle,
# which winit doesn't do yet. Recording an older SDK version keeps the legacy app lifecycle.
xcrun vtool -set-build-version ios 16.0 18.0 -replace \
    -output "$APP/$APP_NAME" "target/$TARGET/release/flappy_bird"
# Bevy looks for `assets/` next to the executable.
cp -R assets "$APP/assets"

# An empty UILaunchScreen is what makes iOS use the full native screen size.
cat > "$APP/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>$APP_NAME</string>
    <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
    <key>CFBundleName</key><string>Flappy Bird</string>
    <key>CFBundleDisplayName</key><string>Flappy Bird</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleSupportedPlatforms</key><array><string>iPhoneOS</string></array>
    <key>DTPlatformName</key><string>iphoneos</string>
    <key>LSRequiresIPhoneOS</key><true/>
    <key>MinimumOSVersion</key><string>16.0</string>
    <key>UIDeviceFamily</key><array><integer>1</integer><integer>2</integer></array>
    <key>UIRequiredDeviceCapabilities</key><array><string>arm64</string><string>metal</string></array>
    <key>UILaunchScreen</key><dict/>
    <key>UIRequiresFullScreen</key><true/>
    <key>UIStatusBarHidden</key><true/>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationLandscapeLeft</string>
        <string>UIInterfaceOrientationLandscapeRight</string>
    </array>
</dict>
</plist>
EOF

# Newest provisioning profile issued for this bundle identifier.
PROFILE=""
DECODED="$(mktemp)"
ENTITLEMENTS="$(mktemp)"
trap 'rm -f "$DECODED" "$ENTITLEMENTS"' EXIT
for candidate in "$PROFILES_DIR"/*.mobileprovision(Nom); do
    security cms -D -i "$candidate" > "$DECODED" 2>/dev/null || continue
    app_id="$(/usr/libexec/PlistBuddy -c "Print :Entitlements:application-identifier" "$DECODED" 2>/dev/null || true)"
    if [[ "$app_id" == *".$BUNDLE_ID" ]]; then
        PROFILE="$candidate"
        break
    fi
done
if [[ -z "$PROFILE" ]]; then
    echo "No provisioning profile for $BUNDLE_ID in $PROFILES_DIR." >&2
    echo "Run an Xcode app with that bundle identifier on your iPhone once, then retry." >&2
    exit 1
fi
security cms -D -i "$PROFILE" > "$DECODED"
/usr/libexec/PlistBuddy -x -c "Print :Entitlements" "$DECODED" > "$ENTITLEMENTS"
cp "$PROFILE" "$APP/embedded.mobileprovision"

SIGN_IDENTITY="${SIGN_IDENTITY:-$(security find-identity -v -p codesigning \
    | grep -m1 -Eo '[0-9A-F]{40} "Apple Development' | cut -d' ' -f1 || true)}"
if [[ -z "$SIGN_IDENTITY" ]]; then
    echo "No 'Apple Development' signing identity found; sign in to Xcode with your Apple ID." >&2
    exit 1
fi
codesign --force --sign "$SIGN_IDENTITY" --entitlements "$ENTITLEMENTS" --timestamp=none "$APP"
echo "Built and signed $APP"

[[ "${1:-}" == "--no-run" ]] && exit 0

DEVICE="${DEVICE:-$(xcrun devicectl list devices 2>/dev/null \
    | grep physical | grep connected \
    | grep -m1 -Eo '[0-9A-F]{8}-[0-9A-F]{16}' || true)}"
if [[ -z "$DEVICE" ]]; then
    echo "No connected iPhone found; plug it in and unlock it." >&2
    exit 1
fi
xcrun devicectl device install app --device "$DEVICE" "$APP"
xcrun devicectl device process launch --device "$DEVICE" --terminate-existing "$BUNDLE_ID"
