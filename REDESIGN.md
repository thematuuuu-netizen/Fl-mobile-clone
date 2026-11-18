# Honeydew — v14 Liquid Glass rollout (master handover doc)

> **This file is the portable source of truth for the current redesign round.**
> Any LLM must be able to continue from this file alone, with no extra
> explanation from the user. Every finished step updates this doc in the same
> pushed commit as its code, tests green — so the last commit is always a
> clean, defined point to continue from.
>
> Companion doc: `IOS_REDESIGN.md` (the finished rounds 1–6: iOS themes,
> rounding, SF-style icons, simulated glass, fonts, mixer cards, plugin
> backgrounds). This doc covers the round that follows them.

## 1. Vision

Honeydew is an open-source FL Studio Mobile clone in pure Rust
(upstream `daijro/honeydew`, egui/eframe UI). This fork is
`thematuuuu-netizen/Fl-mobile-clone`.

This round brings the approved web prototype's Liquid Glass component
designs into the app, piece by piece: glass buttons and controls with the
prototype's structure, motion and tuned settings, so the app feels like an
iPhone app. The audio engine and song format are untouched.

## 2. Standing rules (agreed with the user, do not break them)

1. **Prototype first, always.** Every piece starts from the approved
   prototype design; the user judges each app piece from real screenshots
   on their phone (POCO X6 Pro). Desktop/source verification does not
   count as acceptance.
2. **Reuse before rebuilding.** Mine the reference implementations (§9) for
   exact design values and behaviour. Kotlin/Compose code cannot compile
   inside Rust/egui — reuse means copying design values and porting
   behaviour, never drag-and-drop code.
3. **Piece by piece, with this doc always current.** One component group
   per pushed commit, code + doc together, tests green.
4. **No effect without a real reference implementation behind it.** No
   invented "AI slop" effects; every visual has a source in §9 or in the
   v14 prototype.
5. **Target arm64-v8a only until stable** (the user's phone). Widening
   ABIs later is only a config change.
6. **Working method:** no full local compiles on the weak VM (they OOM at
   6 GB). Edit, quick local `cargo fmt --check`, push, let GitHub CI
   compile and test, and use the Android workflow for APK builds.
7. **GitHub workflow files cannot be pushed by Muse** (GitHub Apps
   without the workflows permission get `403`). The user adds/edits
   workflow files via the GitHub web UI.
8. **Lemon Studio plugin panels keep their own look, untouched** — every
   piece of this rollout skips them.
9. **Motion is subtle.** Controls stay quiet at rest, lift and brighten on
   touch; stretch/squash is small — the knobs should just feel alive.

## 3. The design reference: prototype v14

The reference is **version v14 of the web Liquid Glass prototype** (the
"catalog" build), which the user named on 2026-11-19 as the best prototype
version so far and the reference for icons and icon structure in the app.
Its file: `~/workspace/your_files/prototype-versions/prototype-v14-catalog.html`
(also kept in the artifact's version history as
`liquid-glass-button-prototype` v14).

What v14 defines, per component group: circular glass lens buttons (the
play button is a wide pill and the only tinted/primary one), the Kyant0
repo Toggle, the tempo fader with a glass thumb, the glass tab bar, the
glass dialog and the magnifier — each group with its own optics settings.

### The user's standard settings (verbatim, tuned 2026-11-18)

Globals:

- Spring stiffness ×**1.15**, damping ×**1.00**
- Velocity stretch ×**1.00**
- Press intensity **2.00**
- Button pull **0.70**

Per group (refraction / dispersion / blur):

- Buttons: **1.40 / 0.060 / 0**
- Toggle: **1.00 / 0.035 / 0**
- Fader: **1.15 / 0.035 / 0**
- Tab bar: **0.35 / 0.090 / 0**
- Dialog: **0.85 / 0.035 / blur 1.00**
- Magnifier: **0.35 / 0.020 / 0**

These values live in code as the `Standard` table in `src/ui/glass.rs`
(`BUTTONS`, `TOGGLE`, `FADER`, `TAB`, `DIALOG`, `MAGNIFIER`, plus the
globals `SPRING_STIFFNESS`, `SPRING_DAMPING`, `PRESS_GAIN`, `BUTTON_PULL`),
pinned verbatim by the test `the_v14_standard_is_recorded_verbatim`.

## 4. Settings → app mapping

The prototype's optics drive a real refraction shader; the app's glass is
the simulated glass of rounds 1–6 (translucent fill + light-catching
border + top highlight — egui has no backdrop sampling). Each prototype
value maps onto the simulation as follows. Groups not yet rolled out keep
their mapping here so the next pieces stay consistent.

**Globals (all groups)**

- Spring ×1.15 / ×1.00 → the press spring runs at stiffness
  `340 × 1.15 = 391`, damping `24 × 1.00 = 24` against the app's spring
  (`anim::spring_with` / `anim::press_lift_with`; the app's other springs
  keep 340/24).
- Press intensity 2.00 → a press's lift effects (swell, fill brightening)
  run at twice the base amount.
- Velocity stretch ×1.00 → the press spring's own velocity response (its
  overshoot) is used as-is; no extra stretch term.
- Button pull 0.70 → while pressed, a button leans toward the pointer at
  0.70 of the prototype's magnetic lean (the prototype's ×0.5 factor),
  clamped to a quarter of the disc radius and gated by the lift so it
  eases with the spring.

**Buttons (piece 1, done)**

- Refraction 1.40 → the disc's border and top-highlight alphas are scaled
  ×1.40 — the strongest edge highlight of any group.
- Dispersion 0.060 → a chromatic fringe on the rim: the highlight arc is
  drawn twice more, shifted `radius × 0.060` to each side, warm left /
  cool right, at 30% of the highlight's strength, under the main arc.
- Blur 0 → the fill is the theme's glass fill with no frosting or dimming
  added — the clearest fill of any group; a press brightens it from
  within at the press gain.
- Structure: REC, REWIND and PLAY are glass discs of radius 0.4 × slot
  width under their glyphs. In v14 the play lens is a wide pill; the
  app's transport slots are fixed squares with heavy layout tests, so
  play keeps the app disc geometry and takes v14's material instead:
  it is the only tinted button (the theme accent mixed into its fill at
  the tinted-glass share 0.45). v14's press scales (rec/rewind 1.14,
  play 1.12) map onto the app's existing 4% press swell × press gain.

**Later groups (mapping agreed, not yet built)**

- Toggle (1.00 / 0.035 / 0): the Kyant0 construction — 64×28 track, 40×24
  pill knob, solid white at rest with refraction 0, glass only while
  pressed (see the Toggle piece in §6).
- Fader (1.15 / 0.035 / 0): the stage-3 glass slider thumb, re-tuned to
  this row when the fader piece lands.
- Tab bar (0.35 / 0.090 / 0): the weakest refraction, the strongest
  dispersion — a gentle lens; blur/refraction apply only to the glass,
  never to icons or text (the Kyant0 three-layer rule).
- Dialog (0.85 / 0.035 / blur 1): the one group with blur — a light frost
  over the sheet's fill when it lands.
- Magnifier (0.35 / 0.020 / 0): no app counterpart yet.

## 5. Icon glyphs — PENDING

**Icon glyph set PENDING — to be rebuilt minimal/iPhone-style from a
user-supplied repo; current stage-2 glyphs are placeholders.**

The user decided on 2026-11-19 that the glyphs themselves (play, stop,
record, prev/next and the rest) must be minimal and iPhone-style, defined
by a repository they are about to supply. Until that repo arrives: do NOT
design or redraw any glyph; the stage-2 SF-style icons stay exactly as
they are, as placeholders, inside the new glass structure.

## 6. Rollout scope and checklist

Scope (user, 2026-11-19): glass buttons in the app based on v14 — the
play/transport buttons, the Toggle where switches are needed, the toolbar
if needed, and faders where the app needs them (e.g. the main
compressor's faders) — **except Lemon plugins**, which keep their own
look.

- [x] **Piece 1 — glass transport buttons** (this commit): REC, REWIND
      and PLAY on the tracks screen are v14 glass discs (play tinted),
      pressed on the standard's spring/gain/pull; `REDESIGN.md` created;
      the v14 standard recorded in `src/ui/glass.rs`.
- [ ] **Piece 2 — the Toggle**: rebuild the app's switches on the Kyant0
      LiquidToggle construction with the Toggle row of the standard,
      where switches are needed. **← next step**
- [ ] Piece 3 — the toolbar, if needed, on the buttons row.
- [ ] Piece 4 — faders where the app needs them (e.g. the main
      compressor's faders), on the fader row; Lemon plugins excluded.
- [ ] Icon glyph set rebuilt from the user-supplied repo (§5), once it
      arrives.

## 7. What is already done (rounds 1–6, do not redo)

Full detail in `IOS_REDESIGN.md`. In short: iOS Dark/Light themes with
the iOS blue accent `#007AFF`; iOS rounding on sheets, dialogs, cards and
pill sliders; SF-style icon pass (stage 2 — now placeholders, §5);
simulated glass helpers in `src/ui/glass.rs` with the press-lift spring
in `src/ui/anim.rs`; glass slider thumb, iOS switches and glass sheets
(stage 3); floating rounded main menu with draggable width (stage 4);
two independent font settings (stage 5); mixer channel cards and the
non-Lemon plugin mesh-gradient/glow (stage 6). Final suite at stage 6:
4530 passed / 0 failed.

## 8. Decisions already settled

- **No framework/language switch.** The glass effect is GPU shader code,
  not host-language code; switching to Kotlin/Compose would mean
  rewriting the whole app for no gain. This rollout uses the app's
  simulated glass; the real backdrop-shader route (porting egui-glass's
  approach to egui 0.33.3 + glow) stays an open, separate decision, not
  part of this rollout.
- **Glass only on the floating layer**, never content-on-glass or
  glass-on-glass; tint only the primary action (Apple's WWDC25 session
  219 rules, which v14 also follows).
- **The transport slot layout does not change** in this rollout (fixed
  square slots, heavily tested); v14 contributes material, motion and
  settings, not slot geometry.

## 9. Vital references

Pruned by the user to the essential only:

- **Kyant0/AndroidLiquidGlass** — the standard for the components
  (LiquidButton/Toggle/Slider/BottomTabs constructions and springs).
  https://github.com/Kyant0/AndroidLiquidGlass
- **archisvaze/liquid-glass** — the most precise web glass material
  (Snell-law refraction). https://github.com/archisvaze/liquid-glass
- **samasante/liquid-glass** — control feel and morph patterns.
  https://github.com/samasante/liquid-glass
- **QWEA0/Liquid-Glass-Android** — Liquid Press/Merge behaviour; the
  parameter model the prototype's engine follows.
  https://github.com/QWEA0/Liquid-Glass-Android
- **heonny/egui-glass** — the only egui-native liquid glass; the
  reference if the real-shader route is ever taken.
  https://github.com/heonny/egui-glass
- Skills: `liquid-glass` (deepika-builds), `liquid-glass-physics`
  (custom, archisvaze's model), `swiftui-liquid-glass` (Dimillian).

## 10. Build & test workflow

- Edit → quick local fmt check → push → GitHub CI compiles/tests → the
  Android workflow builds the APK **only on a manual "Run workflow"**
  tap (its push trigger is paths-limited to the workflow file).
- Android workflow: `.github/workflows/android-test-build.yml`, artifact
  `honeydew-android-base-debug` (base APK, no sounds, ~50 MB),
  ~13–15 min.
- The user installs the APK on their POCO X6 Pro and reviews via
  screenshots; their verdict is the acceptance test for every piece.
