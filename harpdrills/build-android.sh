#!/usr/bin/env bash
# build-android.sh — Cross-compile harpdrills as an Android APK
#
# Prerequisites: run ../harp/setup-android.sh first (or have Android SDK/NDK)
# Target: ARM64 tablet (default) or x86_64 emulator (--emulator)
#
# Usage:
#   bash build-android.sh              # build debug APK for ARM64 tablet
#   bash build-android.sh --release    # build release APK for ARM64 tablet
#   bash build-android.sh --emulator   # build debug APK for x86_64 emulator

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# ── Config ──
ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
NDK_VERSION="27.2.12479018"
ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/$NDK_VERSION}"
BUILD_TOOLS="$ANDROID_HOME/build-tools/35.0.0"
PLATFORM_JAR="$ANDROID_HOME/platforms/android-35/android.jar"
MIN_SDK=28
TARGET_SDK=35
OUT_DIR="$SCRIPT_DIR/target/android-apk"

PROFILE="debug"
CARGO_PROFILE_FLAG=""
NDK_TARGET="arm64-v8a"
RUST_TARGET="aarch64-linux-android"

for arg in "$@"; do
    case "$arg" in
        --release) PROFILE="release"; CARGO_PROFILE_FLAG="--release" ;;
        --emulator)
            NDK_TARGET="x86_64"
            RUST_TARGET="x86_64-linux-android"
            ;;
    esac
done

# ── Verify tools ──
check_tool() {
    if [ ! -f "$1" ] && ! command -v "$1" &>/dev/null; then
        echo "ERROR: $1 not found. Run setup-android.sh first."
        exit 1
    fi
}

check_tool "$BUILD_TOOLS/aapt2"
check_tool "$BUILD_TOOLS/apksigner"
check_tool "$BUILD_TOOLS/zipalign"
check_tool "$PLATFORM_JAR"
check_tool cargo-ndk

# Ensure rust target is installed
if ! rustup target list --installed | grep -q "$RUST_TARGET"; then
    echo "Installing Rust target $RUST_TARGET..."
    rustup target add "$RUST_TARGET"
fi

mkdir -p "$OUT_DIR"

CRATE_NAME="harpdrills"
LIB_NAME="harpdrills"
APP_LABEL="Harp Drills"
PACKAGE="com.harp.drills"

echo ""
echo "========================================="
echo "  Building $APP_LABEL ($PROFILE, $NDK_TARGET)"
echo "========================================="

# Step 1: Compile native library
echo ">>> Compiling Rust -> $RUST_TARGET..."
cargo ndk \
    --target "$NDK_TARGET" \
    --platform $MIN_SDK \
    -- build --lib --features android $CARGO_PROFILE_FLAG

SO_PATH="$SCRIPT_DIR/target/$RUST_TARGET/$PROFILE/lib${LIB_NAME}.so"
if [ ! -f "$SO_PATH" ]; then
    echo "ERROR: Expected .so not found at $SO_PATH"
    find "$SCRIPT_DIR/target/$RUST_TARGET/$PROFILE/" -name "*.so" 2>/dev/null
    exit 1
fi
echo ">>> Built: $SO_PATH ($(du -h "$SO_PATH" | cut -f1))"

# Step 2: APK structure
WORK="$OUT_DIR/${CRATE_NAME}-work"
rm -rf "$WORK"
mkdir -p "$WORK/lib/$NDK_TARGET"
cp "$SO_PATH" "$WORK/lib/$NDK_TARGET/libmain.so"

# Step 3: AndroidManifest.xml (portrait orientation)
cat > "$WORK/AndroidManifest.xml" <<MANIFEST
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="$PACKAGE"
    android:versionCode="1"
    android:versionName="0.1.0">

    <uses-sdk android:minSdkVersion="$MIN_SDK" android:targetSdkVersion="$TARGET_SDK" />

    <uses-permission android:name="android.permission.WAKE_LOCK" />
    <uses-feature android:glEsVersion="0x00030000" android:required="true" />

    <application
        android:label="$APP_LABEL"
        android:icon="@mipmap/ic_launcher"
        android:roundIcon="@mipmap/ic_launcher_round"
        android:hasCode="false"
        android:debuggable="$([ "$PROFILE" = "debug" ] && echo true || echo false)">

        <activity
            android:name="android.app.NativeActivity"
            android:label="$APP_LABEL"
            android:configChanges="orientation|screenSize|screenLayout|keyboardHidden"
            android:exported="true"
            android:screenOrientation="portrait">

            <meta-data
                android:name="android.app.lib_name"
                android:value="main" />

            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
MANIFEST

# Step 4: Build APK
echo ">>> Packaging APK..."

UNSIGNED_APK="$WORK/${CRATE_NAME}-unsigned.apk"
ALIGNED_APK="$WORK/${CRATE_NAME}-aligned.apk"
FINAL_APK="$OUT_DIR/${CRATE_NAME}.apk"

RES_DIR="$SCRIPT_DIR/res"
LINK_RES=""
if [ -d "$RES_DIR" ]; then
    RES_COMPILED="$WORK/compiled_res"
    mkdir -p "$RES_COMPILED"
    "$BUILD_TOOLS/aapt2" compile --dir "$RES_DIR" -o "$RES_COMPILED/res.zip"
    LINK_RES="-R $RES_COMPILED/res.zip"
fi

"$BUILD_TOOLS/aapt2" link \
    -o "$UNSIGNED_APK" \
    --manifest "$WORK/AndroidManifest.xml" \
    -I "$PLATFORM_JAR" \
    $LINK_RES \
    --auto-add-overlay \
    --min-sdk-version $MIN_SDK \
    --target-sdk-version $TARGET_SDK

cd "$WORK"
zip -r "$UNSIGNED_APK" lib/

"$BUILD_TOOLS/zipalign" -f 4 "$UNSIGNED_APK" "$ALIGNED_APK"

KEYSTORE="$HOME/.android/debug.keystore"
if [ ! -f "$KEYSTORE" ]; then
    mkdir -p "$HOME/.android"
    keytool -genkeypair \
        -keystore "$KEYSTORE" \
        -storepass android \
        -alias androiddebugkey \
        -keypass android \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -dname "CN=Debug,O=Android,C=US"
fi

"$BUILD_TOOLS/apksigner" sign \
    --ks "$KEYSTORE" \
    --ks-pass pass:android \
    --key-pass pass:android \
    --ks-key-alias androiddebugkey \
    --out "$FINAL_APK" \
    "$ALIGNED_APK"

rm -rf "$WORK"

echo ">>> APK ready: $FINAL_APK ($(du -h "$FINAL_APK" | cut -f1))"
echo ""
echo "To install on emulator:"
echo "  \$ANDROID_HOME/emulator/emulator -avd tablet_13 &"
echo "  adb wait-for-device && adb install $FINAL_APK"
echo ""
echo "To install on tablet (USB):"
echo "  adb install $FINAL_APK"
