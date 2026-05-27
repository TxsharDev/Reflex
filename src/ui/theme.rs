//! Reflex theme - clean, minimal, premium.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Visuals};

// Premium dark palette - inspired by Spotlight/Raycast but darker
pub const ACCENT: Color32 = Color32::from_rgb(88, 166, 255);     // clean blue
pub const GREEN: Color32 = Color32::from_rgb(63, 185, 120);
pub const YELLOW: Color32 = Color32::from_rgb(230, 192, 60);
pub const RED: Color32 = Color32::from_rgb(240, 95, 95);
pub const PURPLE: Color32 = Color32::from_rgb(175, 140, 255);
pub const TEXT: Color32 = Color32::from_rgb(242, 242, 247);       // iOS-style white
pub const TEXT_SEC: Color32 = Color32::from_rgb(155, 155, 170);
pub const DIM: Color32 = Color32::from_rgb(98, 98, 112);
pub const BG: Color32 = Color32::from_rgb(28, 28, 36);           // main background
pub const SURFACE: Color32 = Color32::from_rgb(36, 36, 46);      // elevated surface
pub const BORDER: Color32 = Color32::from_rgb(52, 52, 66);
pub const INPUT_BG: Color32 = Color32::from_rgb(22, 22, 30);     // search input
pub const SELECTED: Color32 = Color32::from_rgb(44, 44, 62);     // selected row
pub const HOVER: Color32 = Color32::from_rgb(38, 38, 52);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = SURFACE;

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DIM);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);

    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.bg_fill = SELECTED;
    visuals.widgets.active.fg_stroke = Stroke::new(0.0, Color32::TRANSPARENT);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.12);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);

    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.window_stroke = Stroke::new(1.0, BORDER);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(egui::TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Small, FontId::new(11.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional));
    style.text_styles.insert(egui::TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace));
    style.spacing.item_spacing = egui::vec2(0.0, 0.0);
    ctx.set_style(style);
}
