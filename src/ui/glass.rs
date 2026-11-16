//! Liquid Glass surfaces, simulated for a renderer with no backdrop blur: a translucent fill that lets the
//! content under it ghost through, a hairline border that catches the light, and a brighter highlight along
//! the top edge. Glass is for the layer that floats over content — buttons, sheets, menus, the rack tab —
//! never for the content itself, and never stacked on glass (Apple's rule, and it keeps the hierarchy clear).
//!
//! The colours come from the theme ([`ThemeColors::glass_fill`] and friends), so every theme gets its own
//! glass. A tinted surface mixes a colour into the fill the way coloured glass does: the tint stays
//! translucent, it never becomes a solid slab.

use eframe::egui::{self, Color32, Painter, Rect, Stroke, pos2};

use crate::theme::ThemeColors;

/// Mixes `tint` into a glass fill at `share` of its colour, keeping the fill's alpha: tinted glass. The mix
/// runs on unmultiplied channels (a `Color32`'s own channels are premultiplied, which would mute the tint).
fn tinted(fill: Color32, tint: Color32, share: f32) -> Color32 {
    let f = fill.to_srgba_unmultiplied();
    let t = tint.to_srgba_unmultiplied();
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * share).round() as u8;
    Color32::from_rgba_unmultiplied(mix(f[0], t[0]), mix(f[1], t[1]), mix(f[2], t[2]), f[3])
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
}
