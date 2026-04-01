# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

ABC music notation tools: a browser-based editor (abc2svg), a native Android tablet app (harpdrills) for scrolling drill notation, and a WebView Android app (abc2stripchart) that uses abc2svg for high-quality strip-chart scrolling.

## Structure

- `abc2svg/abc2svg-1.23.0/` — Offline abc2svg editor. Open `abc2svg.xhtml` in a browser.
- `examples/` — Sample MIDI/MP3 output and `abcplus_en.pdf` (ABC notation guide).
- `data/` — ABC drill files (SATB harp drills from Grossi and Rodriguez methods). Embedded at compile time by harpdrills.
- `harpdrills/` — Native Android tablet app (Rust + eframe/egui).
- `abc2stripchart/` — WebView Android app using abc2svg for notation rendering.

## harpdrills App

Rust eframe/egui NativeActivity app. Renders 4-voice SATB notation on a grand staff with continuous horizontal scrolling (player piano / strip chart style). Portrait mode on 13" Android tablet (1200x1920).

### Build Commands

```bash
cd harpdrills

# Desktop (for development)
cargo run
cargo test

# Android APK — ARM64 tablet
bash build-android.sh
bash build-android.sh --release

# Android APK — x86_64 emulator
bash build-android.sh --emulator

# Install on connected device
adb install target/android-apk/harpdrills.apk

# Run emulator (tablet_13 AVD, 1200x1920 portrait)
$ANDROID_HOME/emulator/emulator -avd tablet_13 -gpu swiftshader_indirect &
```

### Architecture

Same tech stack as `../harp/harp-player/` (Rust + eframe + cargo-ndk + NativeActivity).

- `src/abc.rs` — ABC parser for `[V: S1V1]` inline voice format. Produces `Score` with merged SATB `ScoreEvent`s. No voicing engine.
- `src/notation.rs` — Grand staff renderer using egui painter. 4 voices: soprano/tenor stems up, alto/bass stems down. Proportional spacing with beat-to-pixel interpolation.
- `src/lib.rs` — App UI (drill selector, play/pause, BPM), playback timing, scroll logic, `android_main` entry point.
- `src/music.rs` — Minimal pitch helpers (`key_to_pc`, `key_sig_accidentals`).
- `build-android.sh` — Builds APK with `cargo-ndk` + `aapt2`. Manifest sets `screenOrientation="portrait"`. Package: `com.harp.drills`.

### Dependencies

- Android SDK/NDK: `$HOME/Android/Sdk`, NDK 27.2. Setup via `../harp/setup-android.sh`.
- Rust target: `aarch64-linux-android` (tablet) or `x86_64-linux-android` (emulator).
- `cargo-ndk` for cross-compilation.
- eframe 0.31 with `android-native-activity` feature.

### Emulator Notes

- AVD `tablet_13`: Pixel C, 1200x1920, x86_64, API 35.
- Use `-gpu swiftshader_indirect` to avoid GPU segfaults on headless/remote machines.
- Unicode music symbols (clefs, rests) don't render on Android; the app uses text labels and geometric shapes instead.

## abc2stripchart App

WebView Android app that uses abc2svg JavaScript engine for high-quality music notation with strip-chart horizontal scrolling. Portrait mode.

### Build Commands

```bash
cd abc2stripchart

# Build debug APK (requires Java 21 + Android SDK)
ANDROID_HOME=$HOME/Android/Sdk JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64 ./gradlew assembleDebug

# Install on emulator
adb install app/build/outputs/apk/debug/app-debug.apk
```

### Architecture

- `index.html` — Self-contained HTML app with embedded ABC drill data, abc2svg rendering, strip-chart scroll logic, and playback via abc2svg's `AbcPlay`/`snd-1.js`.
- `app/src/main/assets/` — Bundled HTML + abc2svg JS files + SoundFont data.
- `app/src/main/java/com/harp/stripchart/MainActivity.java` — Minimal WebView activity.
- Standard Gradle Android project. Package: `com.harp.stripchart`.

### Key Design

- abc2svg renders ABC to SVG with `%%pagewidth 200cm` + `%%continueall 1` to force all music onto one horizontal line.
- SVG is CSS-scaled to fill viewport height, CSS `translateX` scrolls horizontally.
- `follow()` from snd-1.js creates `<rect class="abcr _istart_">` overlays on each note.
- `onnote(istart, on)` callback highlights notes and drives scroll position during playback.
- When editing ABC data: update the DRILLS array in `index.html`, then copy to `app/src/main/assets/index.html` before building.
