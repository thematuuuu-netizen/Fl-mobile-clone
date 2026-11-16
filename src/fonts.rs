//! The bundled fonts. Three faces are committed and embedded at build time — Open Sans SemiCondensed SemiBold,
//! Inter SemiBold and Roboto Medium — and the settings pick two of them independently: one for the UI
//! (`Proportional`) and one for the main menu's browser (`Browser`). If the Open Sans file is removed, egui's
//! built-in fonts are used. See `assets/fonts/README.md`.
//!
//! Line height H = ascent − descent at the pixel size, which equals the pixel size exactly, and the baseline is
//! T + 0.7849·px for Open Sans. epaint scales a font so that its em (2048 units) is the `FontId` size, which
//! made every text 2789/2048 ≈ 1.36× too big. `FontTweak` maps the em back so that a `FontId` of `px` has
//! ascent − descent = `px`; ascent, descent, row height, glyphs and advances all follow it. Each face carries
//! its own scale (its em over its ascent − descent), and each family's fallbacks (egui's Ubuntu-Light and the
//! emoji fonts) are registered per family at that family's scale, so the invariant holds in both families
//! whatever the two picks are.

use eframe::egui;

/// The named font family the main menu's browser text is drawn in (the `Browser font:` setting).
pub const BROWSER_FAMILY: &str = "Browser";

/// Standard base64, whitespace ignored and `=` padding ending the data: the decoder for the two faces
/// stored as text (see [`face`]).
#[cfg(has_app_font)]
fn decode_b64(text: &str) -> Vec<u8> {
    let mut table = [255u8; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
    {
        table[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0);
    for &b in text.as_bytes() {
        if b == b'=' {
            break;
        }
        let v = table[b as usize];
        if v == 255 {
            continue;
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// One bundled face: its font-data name, its bytes and its em scale (em / (ascent − descent)). Inter and
/// Roboto are stored as `.ttf.b64` text (the repo's tooling carries text only) and decoded once.
#[cfg(has_app_font)]
fn face(index: u8) -> (&'static str, &'static [u8], f32) {
    match index {
        // Inter SemiBold: em 2048, ascent − descent 1984 − (−494) = 2478.
        1 => {
            static BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
            (
                "Inter-SemiBold",
                BYTES.get_or_init(|| {
                    decode_b64(include_str!("../assets/fonts/Inter-SemiBold.ttf.b64"))
                }),
                2048.0 / 2478.0,
            )
        }
        // Roboto Medium: em 2048, ascent − descent 1900 − (−500) = 2400.
        2 => {
            static BYTES: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
            (
                "Roboto-Medium",
                BYTES.get_or_init(|| {
                    decode_b64(include_str!("../assets/fonts/Roboto-Medium.ttf.b64"))
                }),
                2048.0 / 2400.0,
            )
        }
        // Open Sans SemiCondensed SemiBold: em 2048, ascent − descent 2189 − (−600) = 2789.
        _ => (
            "OpenSans_SemiCondensed-SemiBold",
            include_bytes!("../assets/fonts/OpenSans_SemiCondensed-SemiBold.ttf"),
            2048.0 / 2789.0,
        ),
    }
}

/// Fills `fonts`' family `family` with face `index` first, then the default family's fallbacks cloned at the
/// face's scale under `suffix`ed names (font data is shared between families, so each family needs its own
/// copies to hold its own scale). Returns the family's name list.
#[cfg(has_app_font)]
fn fill_family(
    fonts: &mut egui::FontDefinitions,
    defaults: &egui::FontDefinitions,
    family: egui::FontFamily,
    index: u8,
    suffix: &str,
) -> Vec<String> {
    use std::sync::Arc;

    let (name, bytes, scale) = face(index);
    fonts.font_data.insert(
        name.to_owned(),
        Arc::new(egui::FontData::from_static(bytes).tweak(egui::FontTweak {
            scale,
            ..Default::default()
        })),
    );
    let mut list = vec![name.to_owned()];
    for fallback in &defaults.families[&egui::FontFamily::Proportional] {
        if fallback == name {
            continue;
        }
        let Some(data) = defaults.font_data.get(fallback) else {
            continue;
        };
        let mut data = (**data).clone();
        data.tweak.scale *= scale;
        let key = format!("{fallback}#{suffix}");
        fonts.font_data.insert(key.clone(), Arc::new(data));
        list.push(key);
    }
    fonts.families.insert(family, list.clone());
    list
}

/// Makes the picked faces the first fonts of their families: `ui_font` for `Proportional` (and `Monospace`,
/// which shares the UI face's scale, as it shared the app font's before), `browser_font` for the `Browser`
/// family. The picks are indices into [`crate::settings::FONT_NAMES`].
#[cfg(has_app_font)]
pub fn install_fonts(ctx: &egui::Context, ui_font: u8, browser_font: u8) {
    let defaults = egui::FontDefinitions::default();
    let mut fonts = egui::FontDefinitions::default();
    let ui_list = fill_family(
        &mut fonts,
        &defaults,
        egui::FontFamily::Proportional,
        ui_font,
        "ui",
    );
    fill_family(
        &mut fonts,
        &defaults,
        egui::FontFamily::Name(BROWSER_FAMILY.into()),
        browser_font,
        "browser",
    );
    // Monospace keeps the default family's order at the UI face's scale.
    let (_, _, ui_scale) = face(ui_font);
    let mut mono = Vec::new();
    for fallback in &defaults.families[&egui::FontFamily::Monospace] {
        if let Some(pos) = ui_list.iter().position(|n| {
            n == fallback || n.strip_suffix("#ui").is_some_and(|base| base == fallback)
        }) {
            mono.push(ui_list[pos].clone());
            continue;
        }
        let Some(data) = defaults.font_data.get(fallback) else {
            continue;
        };
        let mut data = (**data).clone();
        data.tweak.scale *= ui_scale;
        let key = format!("{fallback}#ui");
        fonts
            .font_data
            .insert(key.clone(), std::sync::Arc::new(data));
        mono.push(key);
    }
    fonts.families.insert(egui::FontFamily::Monospace, mono);
    ctx.set_fonts(fonts);
}

/// The default picks: Open Sans for the UI, Inter for the browser.
#[cfg(has_app_font)]
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx, 0, 1);
}

/// No app font available: egui's defaults, with the browser family bound to them (an unbound family panics
/// at layout time).
#[cfg(not(has_app_font))]
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx, 0, 1);
}

/// No app font available: egui's defaults, with the browser family bound to them (an unbound family panics
/// at layout time).
#[cfg(not(has_app_font))]
pub fn install_fonts(ctx: &egui::Context, _ui_font: u8, _browser_font: u8) {
    let mut fonts = egui::FontDefinitions::default();
    let proportional = fonts.families[&egui::FontFamily::Proportional].clone();
    fonts
        .families
        .insert(egui::FontFamily::Name(BROWSER_FAMILY.into()), proportional);
    ctx.set_fonts(fonts);
}

#[cfg(all(test, has_app_font))]
mod tests {
    use eframe::egui::{
        self, Align2, Color32, Context, FontId, Rect, TextureId, epaint::Primitive, pos2, vec2,
    };

    /// A context with the app font at `ppp`; `f` runs inside a frame (the second: the first loads the fonts).
    fn with_fonts<R>(ppp: f32, mut f: impl FnMut(&Context) -> R) -> (Context, egui::FullOutput, R) {
        let ctx = Context::default();
        super::install(&ctx);
        let mut raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 100.0))),
            ..Default::default()
        };
        raw.viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(ppp);
        let _ = ctx.run(raw.clone(), |_| {});
        let mut r = None;
        let out = ctx.run(raw, |ctx| r = Some(f(ctx)));
        (ctx, out, r.unwrap())
    }

    #[test]
    fn a_font_size_is_its_line_height() {
        // H = ascent − descent at the pixel size equals the pixel size.
        for ppp in [1.0f32, 2.0] {
            for px in [8.0f32, 13.0, 21.0, 50.0, 85.0] {
                let font = FontId::proportional(px / ppp);
                let (_, _, (h, row)) = with_fonts(ppp, |ctx| {
                    ctx.fonts_mut(|f| {
                        let g = f.layout_no_wrap("Hg".into(), font.clone(), Color32::WHITE);
                        (g.size().y, f.row_height(&font))
                    })
                });
                assert_eq!(h * ppp, px, "galley at {px} px, ppp {ppp}");
                assert!(
                    (row * ppp - px).abs() <= 1.0 / 16.0,
                    "row height {row} at {px} px"
                );
            }
        }
    }

    #[test]
    fn the_stored_faces_decode_to_their_ttf_bytes() {
        assert_eq!(super::decode_b64("aGVsbG8="), b"hello");
        // The two text-stored faces decode to the full TTFs (their byte lengths and the sfnt magic).
        for (index, len) in [(1u8, 419_744usize), (2, 306_536)] {
            let bytes = super::face(index).1;
            assert_eq!(bytes.len(), len, "face {index}");
            assert_eq!(&bytes[..4], &[0, 1, 0, 0], "face {index}");
        }
    }

    #[test]
    fn every_pick_gives_both_families_the_same_line_height_rule() {
        // Whichever faces are picked, a `FontId` of `px` has row height `px` in the UI family and in the
        // browser family alike.
        for (ui, browser) in [(0u8, 1u8), (1, 0), (2, 2), (1, 2)] {
            let ctx = Context::default();
            super::install_fonts(&ctx, ui, browser);
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 100.0))),
                ..Default::default()
            };
            let _ = ctx.run(raw.clone(), |_| {});
            let mut rows = None;
            let _ = ctx.run(raw, |ctx| {
                rows = Some(ctx.fonts_mut(|f| {
                    (
                        f.row_height(&FontId::proportional(21.0)),
                        f.row_height(&FontId::new(
                            21.0,
                            egui::FontFamily::Name(super::BROWSER_FAMILY.into()),
                        )),
                    )
                }));
            });
            let (ui_row, browser_row) = rows.unwrap();
            assert!((ui_row - 21.0).abs() <= 1.0 / 16.0, "ui {ui}: row {ui_row}");
            assert!(
                (browser_row - 21.0).abs() <= 1.0 / 16.0,
                "browser {browser}: row {browser_row}"
            );
        }
    }

    /// The width and first glyph's ink box size of `text` at `size` points, in a context with the app font or not.
    fn glyph_box(app_font: bool, text: &str, size: f32) -> (f32, egui::Vec2) {
        let ctx = Context::default();
        if app_font {
            super::install(&ctx);
        }
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let mut out = None;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            let g = ctx.fonts_mut(|f| {
                f.layout_no_wrap(text.into(), FontId::proportional(size), Color32::WHITE)
            });
            out = Some((g.size().x, g.rows[0].glyphs[0].uv_rect.size));
        });
        out.unwrap()
    }

    #[test]
    fn fallback_glyphs_are_drawn_at_the_app_fonts_em() {
        // `→` and `😀` are not in the app font: egui's fallbacks (Ubuntu-Light, NotoEmoji) draw them at the same
        // em as the app font, 2048/2789 of the size, with their own tweaks kept.
        let scale = 2048.0 / 2789.0;
        for c in ["→", "😀"] {
            assert_eq!(
                glyph_box(true, c, 50.0),
                glyph_box(false, c, 50.0 * scale),
                "{c}"
            );
        }
    }

    /// The device-pixel rows holding ink of `text` at `px` pixels, drawn centred in an `row_px` tall row (the
    /// glyph quads rasterised from the font atlas, nearest texel: glyphs sit on whole pixels).
    fn ink_rows(ppp: f32, text: &str, px: f32, row_px: f32) -> (usize, usize) {
        let row = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, row_px / ppp));
        let (ctx, out, ()) = with_fonts(ppp, |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::background());
            let font = FontId::proportional(px / ppp);
            painter.text(
                row.center(),
                Align2::CENTER_CENTER,
                text,
                font,
                Color32::WHITE,
            );
        });
        let atlas = ctx.fonts(|f| f.image());
        let mut rows = Vec::new();
        for prim in ctx.tessellate(out.shapes, ppp) {
            let Primitive::Mesh(mesh) = prim.primitive else {
                continue;
            };
            assert_eq!(mesh.texture_id, TextureId::default());
            for q in mesh.vertices.as_chunks::<4>().0 {
                let (a, b) = ((q[0].pos * ppp).round(), (q[3].pos * ppp).round());
                let (ua, ub) = (q[0].uv, q[3].uv);
                for y in a.y as usize..b.y as usize {
                    for x in a.x as usize..b.x as usize {
                        let fx = (x as f32 + 0.5 - a.x) / (b.x - a.x);
                        let fy = (y as f32 + 0.5 - a.y) / (b.y - a.y);
                        let tx = ((ua.x + fx * (ub.x - ua.x)) * atlas.size[0] as f32) as usize;
                        let ty = ((ua.y + fy * (ub.y - ua.y)) * atlas.size[1] as f32) as usize;
                        if atlas.pixels[ty * atlas.size[0] + tx].a() > 0 {
                            rows.push(y);
                        }
                    }
                }
            }
        }
        (*rows.iter().min().unwrap(), *rows.iter().max().unwrap())
    }

    #[test]
    fn the_worked_example_puts_h_on_rows_27_to_53() {
        // U = 80, 10 pt = 50 px centred in an 80 px row: T = 15, baseline 54.243,
        // "H" ink rows 27–53.
        for ppp in [1.0f32, 2.0] {
            assert_eq!(ink_rows(ppp, "H", 50.0, 80.0), (27, 53), "ppp {ppp}");
        }
    }
}
