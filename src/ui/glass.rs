//! Liquid Glass surfaces, simulated for a renderer with no backdrop blur: a translucent fill that lets the
//! content under it ghost through, a hairline border that catches the light, and a brighter highlight along
//! the top edge. Glass is for the layer that floats over content — buttons, sheets, menus, the rack tab —
//! never for the content itself, and never stacked on glass (Apple's rule, and it keeps the hierarchy clear).
//!
//! The colours come from the theme ([`ThemeColors::glass_fill`] and friends), so every theme gets its own
//! glass. A tinted surface mixes a colour into the fill the way coloured glass does: the tint stays
//! translucent, it never becomes a solid slab.

use eframe::egui::{self, Color32, Mesh, Painter, Rect, Shape, Stroke, pos2};

use crate::theme::ThemeColors;

/// Mixes `tint` into a glass fill at `share` of its colour, keeping the fill's alpha: tinted glass. The mix
/// runs on unmultiplied channels (a `Color32`'s own channels are premultiplied, which would mute the tint).
fn tinted(fill: Color32, tint: Color32, share: f32) -> Color32 {
    let f = fill.to_srgba_unmultiplied();
    let t = tint.to_srgba_unmultiplied();
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * share).round() as u8;
    Color32::from_rgba_unmultiplied(mix(f[0], t[0]), mix(f[1], t[1]), mix(f[2], t[2]), f[3])
}

/// Whether the rack cards' dynamic-gradient background may move this frame: the app copies the
/// `Animate plugin backgrounds` setting here each frame (the rack paints from contexts that never see
/// the settings). Unset reads as on, the setting's own default.
pub fn bg_anim(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new("plugin_bg_anim")))
        .unwrap_or(true)
}

/// [`bg_anim`]'s writer, called by the app shell once per frame.
pub fn set_bg_anim(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("plugin_bg_anim"), on));
}

/// Paints the rack cards' dynamic gradient over `rect` (already filled with the card's base colour): a
/// small vertex-colour mesh, two soft radial blobs — the channel colour low in one corner, a white
/// ambient lift in the other — fading to nothing at the edges so text keeps its contrast. `phase` drifts
/// the blob centres a little; at rest (or with the motion off) the same field is painted statically.
/// Apple's Aurora guidance: layered colour, a slow calm loop, never a distraction.
pub fn paint_mesh_glow(painter: &Painter, rect: Rect, radius: f32, tint: Color32, phase: f32) {
    const COLS: usize = 6;
    const ROWS: usize = 4;
    let inner = rect.shrink(radius.min(rect.width()).min(rect.height()) * 0.5);
    let a = pos2(
        inner.left() + inner.width() * (0.30 + 0.07 * phase.sin()),
        inner.top() + inner.height() * (0.38 + 0.06 * (phase * 0.9).cos()),
    );
    let b = pos2(
        inner.left() + inner.width() * (0.74 + 0.06 * (phase * 0.8).cos()),
        inner.top() + inner.height() * (0.68 + 0.05 * (phase * 1.1).sin()),
    );
    let blob = |p: egui::Pos2, c: egui::Pos2, peak: f32| {
        let d = ((p.x - c.x) / inner.width()).powi(2) + ((p.y - c.y) / inner.height()).powi(2);
        peak * (-3.2 * d).exp()
    };
    let mut mesh = Mesh::default();
    for r in 0..=ROWS {
        for c in 0..=COLS {
            let p = pos2(
                inner.left() + inner.width() * c as f32 / COLS as f32,
                inner.top() + inner.height() * r as f32 / ROWS as f32,
            );
            let ta = blob(p, a, 0.16);
            let tb = blob(p, b, 0.07);
            let mix = |ch: u8, peak: f32| (f32::from(ch) * peak).round().clamp(0.0, 255.0) as u8;
            let alpha = ((ta + tb) * 255.0).round().clamp(0.0, 255.0) as u8;
            let color = if alpha == 0 {
                Color32::TRANSPARENT
            } else {
                let share = ta / (ta + tb).max(f32::EPSILON);
                Color32::from_rgba_unmultiplied(
                    mix(tint.r(), share) + mix(255, 1.0 - share),
                    mix(tint.g(), share) + mix(255, 1.0 - share),
                    mix(tint.b(), share) + mix(255, 1.0 - share),
                    alpha,
                )
            };
            mesh.vertices.push(egui::epaint::Vertex {
                pos: p,
                uv: egui::Pos2::ZERO,
                color,
            });
        }
    }
    for r in 0..ROWS {
        for c in 0..COLS {
            let i = (r * (COLS + 1) + c) as u32;
            let w = (COLS + 1) as u32;
            mesh.indices
                .extend_from_slice(&[i, i + 1, i + w, i + 1, i + w + 1, i + w]);
        }
    }
    painter.add(Shape::mesh(mesh));
}

/// A glowing edge for the rack cards: the channel colour as a wide faint halo under a thin brighter
/// line, traced on the card's rounded outline.
pub fn paint_glow_edge(painter: &Painter, rect: Rect, radius: f32, tint: Color32) {
    let path = rounded_outline(rect, radius);
    let fade = |a: u8| Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), a);
    painter.add(egui::Shape::line(path.clone(), Stroke::new(3.0, fade(26))));
    painter.add(egui::Shape::line(path, Stroke::new(1.0, fade(90))));
}

/// Paints a glass surface over `rect` with corners `radius`: fill, border, and the top highlight (a short
/// gradient-free arc: a 1 px line inset by the radius at both ends, brightest at the top edge).
pub fn paint_glass(painter: &Painter, rect: Rect, radius: f32, colors: &ThemeColors) {
    paint_glass_tinted(painter, rect, radius, colors, None);
}

/// [`paint_glass`] with a colour mixed into the fill: for a primary action (the play button, record), the
/// one tinted element on a surface — tinting everything is tinting nothing.
pub fn paint_glass_tinted(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    colors: &ThemeColors,
    tint: Option<Color32>,
) {
    let mut fill = colors.glass_fill().to_color32();
    if let Some(tint) = tint {
        fill = tinted(fill, tint, 0.45);
    }
    painter.rect_filled(rect, radius, fill);
    paint_glass_frame(painter, rect, radius, colors);
}

/// Paints only a glass frame — the border and the top highlight, no fill — for a surface that paints its
/// own body under one frame (the main menu's sheet: its body and header are separate fills).
pub fn paint_glass_frame(painter: &Painter, rect: Rect, radius: f32, colors: &ThemeColors) {
    // The border is a stroked path along the outline, not a rect shape: the app's shape archaeology (and
    // its tests) reads rects as surfaces, and a glass edge is not one.
    painter.add(egui::Shape::closed_line(
        rounded_outline(rect, radius),
        Stroke::new(1.0, colors.glass_border().to_color32()),
    ));
    // The highlight: along the top edge, inset by the corners, fading at its ends is faked by two shorter,
    // fainter lines under one brighter middle line.
    let inset = radius.max(2.0);
    let y = rect.top() + 0.5;
    let highlight = colors.glass_highlight().to_color32();
    painter.line_segment(
        [pos2(rect.left() + inset, y), pos2(rect.right() - inset, y)],
        Stroke::new(1.0, highlight),
    );
}

/// Points along a rounded rect's outline, clockwise from the top edge: four corner arcs joined by the
/// straight edges (the closing segment is the top edge).
fn rounded_outline(rect: Rect, radius: f32) -> Vec<egui::Pos2> {
    let r = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let corners = [
        (rect.right() - r, rect.top() + r, -90.0),
        (rect.right() - r, rect.bottom() - r, 0.0),
        (rect.left() + r, rect.bottom() - r, 90.0),
        (rect.left() + r, rect.top() + r, 180.0),
    ];
    let mut pts = Vec::new();
    for (cx, cy, from) in corners {
        for i in 0..=6 {
            let a = (from + 90.0 * i as f32 / 6.0).to_radians();
            pts.push(pos2(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    pts
}

/// Paints a glass disc: the transport's play button and the round floating buttons. Fill, border, and a
/// short highlight arc along the top edge where the light lands.
pub fn paint_glass_disc(painter: &Painter, center: egui::Pos2, r: f32, colors: &ThemeColors) {
    painter.circle_filled(center, r, colors.glass_fill().to_color32());
    painter.circle_stroke(
        center,
        r,
        Stroke::new(1.0, colors.glass_border().to_color32()),
    );
    // The highlight: an arc across the top, inset from the rim.
    let inner = r * 0.80;
    let points: Vec<egui::Pos2> = (0..=12)
        .map(|i| {
            let a = (200.0 + 140.0 * i as f32 / 12.0).to_radians();
            center + egui::vec2(inner * a.cos(), inner * a.sin())
        })
        .collect();
    painter.add(egui::Shape::line(
        points,
        Stroke::new(1.5, colors.glass_highlight().to_color32()),
    ));
}

/// Brightens a glass fill by `lift` (a press spring's output, ~0..1): the material lights from within where
/// the finger lands. Returns the fill to paint with; the caller repaints the surface with it.
pub fn lifted_fill(colors: &ThemeColors, lift: f32) -> Color32 {
    let fill = colors.glass_fill();
    let a = (fill.a + 0.10 * lift.clamp(0.0, 1.2)).min(0.9);
    crate::theme::Rgba { a, ..fill }.to_color32()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tinting_keeps_the_alpha_and_moves_the_colour() {
        let fill = Color32::from_rgba_unmultiplied(255, 255, 255, 26);
        let out = tinted(fill, Color32::from_rgb(0, 122, 255), 0.45);
        assert_eq!(out.a(), 26);
        let [r, _, b, _] = out.to_srgba_unmultiplied();
        assert!(b > r && r < 255 && b > 140, "{out:?}");
    }

    #[test]
    fn the_background_animation_flag_defaults_on_and_follows_the_writer() {
        let ctx = egui::Context::default();
        assert!(super::bg_anim(&ctx));
        super::set_bg_anim(&ctx, false);
        assert!(!super::bg_anim(&ctx));
        super::set_bg_anim(&ctx, true);
        assert!(super::bg_anim(&ctx));
    }
}
