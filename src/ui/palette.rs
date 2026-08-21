//! Command palette - clean, minimal, instant.

use eframe::egui;
use crate::search::ranker::{SearchResult, ResultKind, ResultAction};
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Apps,
    Files,
    Clipboard,
    Actions,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Apps => "Apps",
            Filter::Files => "Files",
            Filter::Clipboard => "Clipboard",
            Filter::Actions => "Actions",
        }
    }

    pub fn matches(self, kind: &ResultKind) -> bool {
        match self {
            Filter::All => true,
            Filter::Apps => *kind == ResultKind::App,
            Filter::Files => *kind == ResultKind::File || *kind == ResultKind::Folder,
            Filter::Clipboard => *kind == ResultKind::Clipboard,
            Filter::Actions => *kind == ResultKind::Action || *kind == ResultKind::Calculator,
        }
    }
}

pub struct PaletteState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub visible: bool,
    pub filter: Filter,
    pub alpha: f32,
}

impl PaletteState {
    pub fn new() -> Self {
        Self { query: String::new(), results: Vec::new(), selected: 0, visible: false, filter: Filter::All, alpha: 0.0 }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
        self.filter = Filter::All;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
    }

    /// Animate alpha toward 1.0 (visible) or 0.0 (hidden). ~120ms fade.
    pub fn tick_alpha(&mut self) {
        let target = if self.visible { 1.0 } else { 0.0 };
        let speed = 0.15; // ~7 frames at 30ms = ~200ms total
        if self.alpha < target {
            self.alpha = (self.alpha + speed).min(target);
        } else if self.alpha > target {
            self.alpha = (self.alpha - speed).max(target);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }

    pub fn move_down(&mut self) {
        let count = self.filtered_results().len();
        if self.selected + 1 < count { self.selected += 1; }
    }

    pub fn filtered_results(&self) -> Vec<&SearchResult> {
        self.results.iter().filter(|r| self.filter.matches(&r.kind)).collect()
    }

    pub fn selected_action(&self) -> Option<ResultAction> {
        let filtered = self.filtered_results();
        filtered.get(self.selected).map(|r| r.action.clone())
    }

    pub fn ideal_height(&self) -> f32 {
        let input_h = 60.0;
        let filter_h = 36.0;
        let divider = 1.0;
        let result_h = 52.0;
        let footer_h = 24.0;
        let filtered = self.filtered_results().len();
        let visible = 8.min(filtered);

        if filtered == 0 && self.query.is_empty() {
            input_h + filter_h + footer_h
        } else if filtered == 0 {
            input_h + filter_h + divider + 44.0 + footer_h
        } else {
            input_h + filter_h + divider + (visible as f32 * result_h) + footer_h + 4.0
        }
    }
}

pub fn draw(ctx: &egui::Context, state: &mut PaletteState, _ctrl_held: bool, file_count: usize, indexing: bool) -> Option<ResultAction> {
    if !state.visible && state.alpha < 0.01 { return None; }

    let alpha = state.alpha;
    let mut executed_action = None;

    // Apply fade: multiply all colors by alpha
    let bg = with_alpha(theme::BG, alpha);
    let border = with_alpha(theme::BORDER, alpha);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(bg)
                .corner_radius(egui::CornerRadius::same(16))
                .stroke(egui::Stroke::new(1.0, border))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 32,
                    spread: 4,
                    color: egui::Color32::from_black_alpha((80.0 * alpha) as u8),
                })
        )
        .show(ctx, |ui| {
            let w = ui.available_width();

            // === Search input ===
            egui::Frame::NONE
                .fill(theme::INPUT_BG)
                .corner_radius(egui::CornerRadius { nw: 16, ne: 16, sw: 0, se: 0 })
                .inner_margin(egui::Margin { left: 20, right: 20, top: 16, bottom: 16 })
                .show(ui, |ui| {
                    ui.set_width(w);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.query)
                            .font(egui::FontId::new(22.0, egui::FontFamily::Proportional))
                            .text_color(theme::TEXT)
                            .hint_text(egui::RichText::new("Search...").color(theme::DIM).size(22.0))
                            .frame(false)
                            .desired_width(w - 40.0)
                            .return_key(None) // Don't let TextEdit swallow Enter
                    );
                    response.request_focus();
                });

            // === Filter tabs ===
            egui::Frame::NONE
                .fill(theme::SURFACE)
                .inner_margin(egui::Margin { left: 16, right: 16, top: 6, bottom: 6 })
                .show(ui, |ui| {
                    ui.set_width(w);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        for filter in [Filter::All, Filter::Apps, Filter::Files, Filter::Clipboard, Filter::Actions] {
                            let active = state.filter == filter;
                            let text = egui::RichText::new(filter.label())
                                .size(12.0)
                                .color(if active { theme::ACCENT } else { theme::DIM });

                            let btn = ui.add(
                                egui::Button::new(text)
                                    .fill(if active { theme::ACCENT.gamma_multiply(0.1) } else { egui::Color32::TRANSPARENT })
                                    .corner_radius(egui::CornerRadius::same(6))
                            );
                            if btn.clicked() {
                                state.filter = filter;
                                state.selected = 0;
                            }
                        }
                    });
                });

            // Thin divider
            let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, theme::BORDER);

            // === Keyboard ===
            let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));
            let ctrl_enter = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter));

            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) { state.hide(); }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) { state.move_up(); }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) { state.move_down(); }
            if (enter_pressed || ctrl_enter)
                && let Some(action) = state.selected_action()
            {
                executed_action = Some(action);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
                state.filter = match state.filter {
                    Filter::All => Filter::Apps,
                    Filter::Apps => Filter::Files,
                    Filter::Files => Filter::Clipboard,
                    Filter::Clipboard => Filter::Actions,
                    Filter::Actions => Filter::All,
                };
                state.selected = 0;
            }

            // === Results ===
            let filtered: Vec<SearchResult> = state.filtered_results().into_iter().cloned().collect();

            if !filtered.is_empty() {
                egui::Frame::NONE
                    .inner_margin(egui::Margin { left: 6, right: 6, top: 4, bottom: 4 })
                    .show(ui, |ui| {
                        let max = 8.min(filtered.len());
                        #[allow(clippy::needless_range_loop)]
                        for i in 0..max {
                            let r = &filtered[i];
                            let selected = i == state.selected;

                            let response = ui.add_sized(
                                [w - 12.0, 48.0],
                                ResultRow { result: r, selected },
                            );

                            if response.clicked() {
                                executed_action = Some(filtered[i].action.clone());
                            }
                            if response.hovered() {
                                state.selected = i;
                            }
                        }
                    });
            } else if !state.query.is_empty() {
                egui::Frame::NONE
                    .inner_margin(egui::Margin { left: 20, right: 20, top: 14, bottom: 14 })
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("No results").size(14.0).color(theme::DIM));
                    });
            }

            // Minimal footer
            let (footer_rect, _) = ui.allocate_exact_size(egui::vec2(w, 20.0), egui::Sense::hover());
            if ui.is_rect_visible(footer_rect) {
                let p = ui.painter();
                // Thin top line
                p.line_segment(
                    [egui::pos2(footer_rect.left() + 12.0, footer_rect.top()),
                     egui::pos2(footer_rect.right() - 12.0, footer_rect.top())],
                    egui::Stroke::new(0.5, theme::BORDER),
                );

                let status = if indexing {
                    format!("Indexing... {} files", file_count)
                } else {
                    format!("{} files indexed", file_count)
                };

                p.text(
                    egui::pos2(footer_rect.left() + 14.0, footer_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Reflex",
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    theme::ACCENT.gamma_multiply(0.6),
                );
                p.text(
                    egui::pos2(footer_rect.right() - 14.0, footer_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    status,
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    theme::DIM,
                );
            }
        });

    executed_action
}

// Custom widget for a single result row
struct ResultRow<'a> {
    result: &'a SearchResult,
    selected: bool,
}

impl<'a> egui::Widget for ResultRow<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background
            if self.selected {
                painter.rect_filled(rect, egui::CornerRadius::same(10), theme::SELECTED);
            } else if response.hovered() {
                painter.rect_filled(rect, egui::CornerRadius::same(10), theme::HOVER);
            }

            let left = rect.left() + 14.0;
            let mid_y = rect.center().y;

            // Color indicator dot
            let dot_color = match self.result.kind {
                ResultKind::App => theme::GREEN,
                ResultKind::File => theme::ACCENT,
                ResultKind::Folder => theme::YELLOW,
                ResultKind::Clipboard => theme::PURPLE,
                ResultKind::Calculator => theme::YELLOW,
                ResultKind::Action => theme::RED,
            };
            painter.circle_filled(egui::pos2(left + 4.0, mid_y), 4.0, dot_color);

            // Title
            let title_pos = egui::pos2(left + 20.0, mid_y - 10.0);
            painter.text(
                title_pos,
                egui::Align2::LEFT_TOP,
                &self.result.title,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
                theme::TEXT,
            );

            // Subtitle
            if !self.result.subtitle.is_empty() {
                let sub: String = self.result.subtitle.chars().take(60).collect();
                painter.text(
                    egui::pos2(left + 20.0, mid_y + 4.0),
                    egui::Align2::LEFT_TOP,
                    sub,
                    egui::FontId::new(11.0, egui::FontFamily::Proportional),
                    theme::DIM,
                );
            }

            // Right side
            let right_x = rect.right() - 14.0;

            if self.selected {
                // Keyboard hint
                let hint = match self.result.kind {
                    ResultKind::App => "Enter open | Ctrl+Enter admin",
                    ResultKind::File => "Enter open | Ctrl+Enter folder",
                    ResultKind::Clipboard => "Enter paste to clipboard",
                    ResultKind::Calculator => "Enter copy result",
                    ResultKind::Action => "Enter execute",
                    ResultKind::Folder => "Enter open folder",
                };
                painter.text(
                    egui::pos2(right_x, mid_y),
                    egui::Align2::RIGHT_CENTER,
                    hint,
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    theme::DIM,
                );
            } else {
                // Type badge
                let badge = match self.result.kind {
                    ResultKind::App => "APP",
                    ResultKind::File => {
                        // Show file extension as badge
                        let ext = self.result.subtitle.rsplit('.').next().unwrap_or("FILE");
                        if ext.len() <= 4 { ext } else { "FILE" }
                    }
                    ResultKind::Folder => "DIR",
                    ResultKind::Clipboard => "CLIP",
                    ResultKind::Calculator => "CALC",
                    ResultKind::Action => "CMD",
                };
                painter.text(
                    egui::pos2(right_x, mid_y),
                    egui::Align2::RIGHT_CENTER,
                    badge.to_uppercase(),
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    theme::DIM,
                );
            }
        }

        response
    }
}

/// Multiply a color's alpha channel by a factor (for fade in/out).
fn with_alpha(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    egui::Color32::from_rgba_premultiplied(r, g, b, (a as f32 * factor) as u8)
}
