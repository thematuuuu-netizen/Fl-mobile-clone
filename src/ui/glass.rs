//! Liquid Glass surfaces, simulated for a renderer with no backdrop blur: a translucent fill that lets the
//! content under it ghost through, a hairline border that catches the light, and a brighter highlight along
//! the top edge. Glass is for the layer that floats over content — buttons, sheets, menus, the rack tab —
//! never for the content itself, and never stacked on glass (Apple's rule, and it keeps the hierarchy clear).
//!
//! The colours come from the theme ([`ThemeColors::glass_fill`] and friends), so every theme gets its own
//! glass. A tinted surface mixes a colour into the fill the way coloured glass does: the tint stays
//! translucent, it never becomes a solid slab.

use eframe::egui::{Color32, Painter, Rect, Stroke, StrokeKind, pos2};

use crate::theme::ThemeColors;

/// Mixes `tint` into a glass fill at `share` of its colour, keeping the fill's alpha: tinted glass.
fn tinted(fill: Color32, tint: Color32, share: f32) -> Color32 {
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * share).round() as u8;
    Color32::from_rgba_unmultiplied(
        mix(fill.r(), tint.r()),
        mix(fill.g(), tint.g()),
        mix(fill.b(), tint.b()),
        fill.a(),
    )
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
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, colors.glass_border().to_color32()),
        StrokeKind::Inside,
    );
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
        assert!(out.b() > out.r(), "{out:?}");
        assert!(out.r() < 255 && out.b() > 140, "{out:?}");
    }
}
