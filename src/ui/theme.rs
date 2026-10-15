//! Colors and base style.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(212, 160, 60);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(160, 120, 50);

// Result kinds
pub const GREEN: Color32 = Color32::from_rgb(100, 195, 140);
pub const BLUE: Color32 = Color32::from_rgb(100, 160, 240);
pub const YELLOW: Color32 = Color32::from_rgb(220, 190, 80);
pub const PURPLE: Color32 = Color32::from_rgb(170, 145, 235);
pub const RED: Color32 = Color32::from_rgb(235, 110, 100);

pub const TEXT: Color32 = Color32::from_rgb(240, 238, 245);
pub const DIM: Color32 = Color32::from_rgb(120, 118, 135);

pub const BG: Color32 = Color32::from_rgb(26, 24, 34);
pub const SURFACE: Color32 = Color32::from_rgb(34, 32, 44);
pub const BORDER: Color32 = Color32::from_rgb(50, 48, 64);
pub const INPUT_BG: Color32 = Color32::from_rgb(20, 19, 27);
pub const SELECTED: Color32 = Color32::from_rgb(42, 40, 56);
pub const HOVER: Color32 = Color32::from_rgb(36, 34, 48);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = BG;
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = SURFACE;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.text_cursor.stroke = Stroke::new(1.5_f32, ACCENT);

    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.corner_radius = CornerRadius::same(6);
        widget.fg_stroke = Stroke::NONE;
    }
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.weak_bg_fill = HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.active.bg_fill = SELECTED;
    visuals.widgets.active.weak_bg_fill = SELECTED;
    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new(11.0, FontFamily::Monospace),
        );
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
    });
}

/// Segoe UI and Cascadia Mono when installed, ahead of egui's built-in
/// fonts, which stay as fallbacks for missing glyphs.
pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for (name, file, family) in [
        (
            "segoe-ui",
            r"C:\Windows\Fonts\segoeui.ttf",
            FontFamily::Proportional,
        ),
        (
            "cascadia-mono",
            r"C:\Windows\Fonts\CascadiaMono.ttf",
            FontFamily::Monospace,
        ),
    ] {
        if let Ok(data) = std::fs::read(file) {
            fonts
                .font_data
                .insert(name.to_owned(), egui::FontData::from_owned(data).into());
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, name.to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
