//! Reflex theme — Lumen Night Foundry register.
//!
//! /* Hallmark · theme: lumen · drop: night · genre: technical
//!  * accent: molten brass (H 50) · canvas: cool-violet (H 265)
//!  * type: lowercase prose + UPPERCASE mono labels
//!  * contrast: pass
//!  */

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Visuals};

// Lumen Night Foundry palette — cool-violet canvas, molten brass accent
pub const ACCENT: Color32 = Color32::from_rgb(212, 160, 60);       // molten brass
pub const ACCENT_DIM: Color32 = Color32::from_rgb(160, 120, 50);   // brass at rest
pub const GREEN: Color32 = Color32::from_rgb(100, 195, 140);       // sage green
pub const YELLOW: Color32 = Color32::from_rgb(220, 190, 80);       // warm gold
pub const RED: Color32 = Color32::from_rgb(235, 110, 100);         // soft coral
pub const PURPLE: Color32 = Color32::from_rgb(170, 145, 235);      // cool violet
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(100, 160, 240); // cool blue (files)

pub const TEXT: Color32 = Color32::from_rgb(240, 238, 245);        // near-white, violet lift
#[allow(dead_code)]
pub const TEXT_SEC: Color32 = Color32::from_rgb(175, 172, 188);    // secondary
pub const DIM: Color32 = Color32::from_rgb(120, 118, 135);         // tertiary / hints

pub const BG: Color32 = Color32::from_rgb(26, 24, 34);             // cool-violet near-black
pub const SURFACE: Color32 = Color32::from_rgb(34, 32, 44);        // elevated
pub const BORDER: Color32 = Color32::from_rgb(50, 48, 64);         // hairline
pub const INPUT_BG: Color32 = Color32::from_rgb(20, 19, 27);       // input well
pub const SELECTED: Color32 = Color32::from_rgb(42, 40, 56);       // selected row
pub const HOVER: Color32 = Color32::from_rgb(36, 34, 48);          // hover row

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = SURFACE;

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DIM);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = SELECTED;
    visuals.widgets.active.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.12);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.window_stroke = Stroke::new(1.0, BORDER);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(egui::TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace, FontId::new(11.0, FontFamily::Monospace));
    style.spacing.item_spacing = egui::vec2(0.0, 0.0);
    ctx.set_style(style);
}
