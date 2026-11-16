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

## Test fixes (CI green)

The first CI runs failed on 13 unit tests whose expectations were tied to the old
palette or pre-rounding geometry. Fixed without changing the redesign's look:

- `theme.rs`: iOS Dark `text` is now `F2F2F7` and `whiteKey` is `E5E5EA` (also the
  resolve defaults). Pure `FFFFFF` collided with hard-coded white elements — the
  rack-tab icon read as a selection handle, and a pressed piano key was
  indistinguishable from an unpressed one. Both are Apple system greys, visually
  unchanged.
- `ui/widgets/tests/kit_c.rs`: the menu-button box now expects the 0.2 U corner
  radius (6 pt at test scale) instead of square corners.
- `panels/fx/equalizer.rs`, `panels/fx/note_echo.rs`: hard-coded colour snapshots
  updated to the values the new palette produces.

Local result: `cargo test -p honeydew --lib` — 4524 passed, 0 failed.

## Liquid Glass round

A second round, following Apple's WWDC25 "Meet Liquid Glass" guidance (glass only on the
floating layer, never glass on glass, tint only for primary actions) and the iOS system
palette. Real backdrop blur/refraction is not possible in egui, so the glass is simulated
with translucent fills, hairline borders, top highlights and damped-spring motion.

- **Channel colours** snap to the iOS system palette (red, orange, yellow, green, light
  blue, blue, purple, pink) instead of raw HSL rotation.
- **Icons** (`src/icons.rs`): an SF-Symbols-style consistency pass — one grid, one stroke
  weight family, rounded joins — verified with the repo's icon-sheet render harness.
- **Controls**: sliders get a glass thumb that takes the channel colour as a lens and
  stretches subtly while held; toggles are iOS switches; popup sheets and floating
  buttons are dark glass; the transport button is ringed in glass.
- **Main menu** is a floating sheet (inset, 0.5 U corners, glass frame, grip pill) whose
  width is draggable from its left edge and remembered (`menuWidth` setting); its big
  white circle buttons became dark glass discs. It no longer rigidly takes half the
  screen.
- **Fonts**: two independent settings, `UI font:` and `Browser font:`, each Open Sans /
  Inter / Roboto. Inter and Roboto are bundled; they are stored base64-encoded
  (`assets/fonts/*.ttf.b64`) because this repo's tooling carries text only, and decoded
  once at startup. Font picks are matched in line height, so switching never reflows
  rows. The song browser, modules browser and search surfaces use the browser font.
- **Mixer**: every channel strip stands on its own card (app background, rounded,
  separated); the selected strip is ringed in its channel colour.
- **Rack card backgrounds**: every card except the Lemon Studio ones (their embedded
  editors keep their own look, untouched) gets an Apple-style dynamic gradient — a
  vertex-colour mesh with the channel colour and a white ambient blob drifting slowly
  (an ~11 s calm loop, per Aurora-style guidance) plus a glowing channel-colour edge.
  Alphas stay low so text contrast is preserved. The `Animate plugin backgrounds`
  setting (`pluginAnim`) turns the motion off (the glow then paints statically), and
  the global Animation setting gates it too.
- **Orientation**: the in-app default is now Landscape, matching the manifest.

Local result: `cargo test -p honeydew --lib` — 4530 passed, 0 failed; clippy and
rustfmt clean.

## Android test build

`.github/workflows/android-test-build.yml` builds the base (no-sounds) debug APK
on GitHub's runners and uploads it as the `honeydew-android-base-debug` artifact:
Actions → "Android test build (base APK)" → Run workflow → download the artifact
→ install the APK (Android 8+, debug-signed as "Honeydew Debug").
