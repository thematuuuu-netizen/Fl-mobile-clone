# iOS-style redesign of this fork

This fork (`thematuuuu-netizen/Fl-mobile-clone`, from `daijro/honeydew`) restyles the
app's egui UI to an iPhone / iOS look. The audio engine and song format are untouched.

## Orientation

`android/app/src/main/AndroidManifest.xml`: the activity is locked to
`android:screenOrientation="sensorLandscape"`.

## Themes (`src/theme.rs`)

- Theme 0 is now **iOS Dark** (the default): background `#000000`, control fill
  `#1C1C1E`, module fill `#2C2C2E`, menu fill `#3A3A3C`, text `#FFFFFF`.
- Theme 3 is now **iOS Light**: background `#F2F2F7`, white cards, text `#1C1C1E`.
- Every built-in theme's accent changed from FL orange `#FB8B3F` to iOS system
  blue `#007AFF`; the fallback colours in `RawTheme::resolve` match iOS Dark.
- Tests in `src/theme.rs` and `src/ui/settings_tab.rs` were updated to the new
  names and values.

## Rounded corners (iOS sheets and controls)

All radii are clamped to at most half the painted rect's short side, so small or
animating shapes never distort:

- Dialog panels: 0.5 U, dialog buttons 0.3 U (`src/ui/dialog.rs`)
- Rack module cards: 0.5 U (`src/ui/rack.rs`)
- Popup menus: 0.35 U (`src/ui/popup.rs`)
- Search bar and the Save dialog's name field: rounded fields
  (`src/ui/search_bar.rs`, `src/ui/save_dialog.rs`)
- Module dropdown boxes: 0.2 U (`src/ui/widgets/menu_button.rs`)
- Module display placeholders: 0.15 U (`src/ui/widgets/plain.rs`)
- Sliders: pill tracks and fills (`src/ui/widgets/slider.rs`)

Deliberately not rounded: piano-roll notes, step-sequencer cells, clips, meters,
piano keys and tiled tab rows — those tile against each other, and rounding them
would misalign the grid.

## Building the Android app

Requirements (from `README.md`): Java 21, Android SDK platform 36,
Build Tools 36.0.0, NDK 28.0.13004108, Python 3 and `unzip`, with
`ANDROID_HOME` set. Then:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk --version 4.1.2 --locked
tools/android-dist.sh
```

This writes `target/honeydew-android-<version>-arm-debug.apk` (one APK for
64-bit and 32-bit ARM, Android 8+). The debug app is called “Honeydew Debug” and
installs alongside the release app. Add `--base` for the APK without sounds,
which downloads them on first launch.
