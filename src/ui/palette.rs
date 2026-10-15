//! Command palette — Lumen Night Foundry.

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
            Filter::All => "all",
            Filter::Apps => "apps",
            Filter::Files => "files",
            Filter::Clipboard => "clips",
            Filter::Actions => "actions",
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
    }

    /// Animate alpha toward 1.0 (visible) or 0.0 (hidden). ~120ms fade.
    pub fn tick_alpha(&mut self) {
        let target = if self.visible { 1.0 } else { 0.0 };
        let speed = 0.18;
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
        let input_h = 56.0;
        let filter_h = 32.0;
        let divider = 1.0;
        let result_h = 48.0;
        let footer_h = 28.0;
        let filtered = self.filtered_results().len();
        let visible = 8.min(filtered);

        if filtered == 0 && self.query.is_empty() {
            input_h + filter_h + footer_h
        } else if filtered == 0 {
            input_h + filter_h + divider + 40.0 + footer_h
        } else {
            input_h + filter_h + divider + (visible as f32 * result_h) + footer_h + 4.0
        }
    }
}

pub fn draw(ctx: &egui::Context, state: &mut PaletteState, _ctrl_held: bool, file_count: usize, indexing: bool) -> Option<ResultAction> {
    if !state.visible && state.alpha < 0.01 { return None; }

    let alpha = state.alpha;
    let mut executed_action = None;

    let bg = with_alpha(theme::BG, alpha);
    let border = with_alpha(theme::BORDER, alpha);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(bg)
                .corner_radius(egui::CornerRadius::same(12))
                .stroke(egui::Stroke::new(1.0_f32, border))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 4],
                    blur: 24,
                    spread: 2,
                    color: egui::Color32::from_black_alpha((60.0 * alpha) as u8),
                })
        )
        .show(ctx, |ui| {
            let w = ui.available_width();

            // === Search input ===
            egui::Frame::NONE
                .fill(with_alpha(theme::INPUT_BG, alpha))
                .corner_radius(egui::CornerRadius { nw: 12, ne: 12, sw: 0, se: 0 })
                .inner_margin(egui::Margin { left: 18, right: 18, top: 14, bottom: 14 })
                .show(ui, |ui| {
                    ui.set_width(w);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.query)
                            .font(egui::FontId::new(20.0, egui::FontFamily::Proportional))
                            .text_color(with_alpha(theme::TEXT, alpha))
                            .hint_text(egui::RichText::new("search...").color(with_alpha(theme::DIM, alpha)).size(20.0))
                            .frame(false)
                            .desired_width(w - 36.0)
                            .return_key(None)
                    );
                    response.request_focus();
                });

            // === Filter tabs (lowercase, subtle) ===
            egui::Frame::NONE
                .inner_margin(egui::Margin { left: 14, right: 14, top: 5, bottom: 5 })
                .show(ui, |ui| {
                    ui.set_width(w);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        for filter in [Filter::All, Filter::Apps, Filter::Files, Filter::Clipboard, Filter::Actions] {
                            let active = state.filter == filter;
                            let text = egui::RichText::new(filter.label())
                                .size(11.0)
                                .family(egui::FontFamily::Monospace)
                                .color(if active { with_alpha(theme::ACCENT, alpha) } else { with_alpha(theme::DIM, alpha) });

                            let btn = ui.add(
                                egui::Button::new(text)
                                    .fill(if active { with_alpha(theme::ACCENT.gamma_multiply(0.08), alpha) } else { egui::Color32::TRANSPARENT })
                                    .corner_radius(egui::CornerRadius::same(4))
                                    .stroke(if active { egui::Stroke::new(0.5_f32, with_alpha(theme::ACCENT.gamma_multiply(0.3), alpha)) } else { egui::Stroke::new(0.0_f32, egui::Color32::TRANSPARENT) })
                            );
                            if btn.clicked() {
                                state.filter = filter;
                                state.selected = 0;
                            }
                        }
                    });
                });

            // Hairline divider
            let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, with_alpha(theme::BORDER, alpha));

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
                                [w - 12.0, 44.0],
                                ResultRow { result: r, selected, alpha },
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
                    .inner_margin(egui::Margin { left: 18, right: 18, top: 12, bottom: 12 })
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("no results").size(13.0).color(with_alpha(theme::DIM, alpha)));
                    });
            }

            // === Footer (hairline top, mono) ===
            let (footer_rect, _) = ui.allocate_exact_size(egui::vec2(w, 24.0), egui::Sense::hover());
            if ui.is_rect_visible(footer_rect) {
                let p = ui.painter();
                p.line_segment(
                    [egui::pos2(footer_rect.left() + 10.0, footer_rect.top()),
                     egui::pos2(footer_rect.right() - 10.0, footer_rect.top())],
                    egui::Stroke::new(0.5_f32, with_alpha(theme::BORDER, alpha)),
                );

                let status = if indexing {
                    format!("indexing · {} files", file_count)
                } else {
                    format!("{} files", file_count)
                };

                p.text(
                    egui::pos2(footer_rect.left() + 12.0, footer_rect.center().y + 1.0),
                    egui::Align2::LEFT_CENTER,
                    "reflex",
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    with_alpha(theme::ACCENT_DIM, alpha),
                );
                p.text(
                    egui::pos2(footer_rect.right() - 12.0, footer_rect.center().y + 1.0),
                    egui::Align2::RIGHT_CENTER,
                    status,
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    with_alpha(theme::DIM, alpha),
                );
            }
        });

    executed_action
}

// Result row widget
struct ResultRow<'a> {
    result: &'a SearchResult,
    selected: bool,
    alpha: f32,
}

impl<'a> egui::Widget for ResultRow<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.available_size_before_wrap();
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background — subtle, no heavy fills
            if self.selected {
                painter.rect_filled(rect, egui::CornerRadius::same(8), with_alpha(theme::SELECTED, self.alpha));
            } else if response.hovered() {
                painter.rect_filled(rect, egui::CornerRadius::same(8), with_alpha(theme::HOVER, self.alpha));
            }

            let left = rect.left() + 12.0;
            let mid_y = rect.center().y;

            // Colour dot (small, precise)
            let dot_color = match self.result.kind {
                ResultKind::App => theme::GREEN,
                ResultKind::File => theme::ACCENT_BLUE,
                ResultKind::Folder => theme::YELLOW,
                ResultKind::Clipboard => theme::PURPLE,
                ResultKind::Calculator => theme::ACCENT,
                ResultKind::Action => theme::RED,
            };
            painter.circle_filled(egui::pos2(left + 4.0, mid_y), 3.5, with_alpha(dot_color, self.alpha));

            // Title (lowercase feel, 13.5px)
            let title_pos = egui::pos2(left + 18.0, mid_y - 9.0);
            painter.text(
                title_pos,
                egui::Align2::LEFT_TOP,
                &self.result.title,
                egui::FontId::new(13.5, egui::FontFamily::Proportional),
                with_alpha(theme::TEXT, self.alpha),
            );

            // Subtitle (dim, 11px)
            if !self.result.subtitle.is_empty() {
                let sub: String = self.result.subtitle.chars().take(60).collect();
                painter.text(
                    egui::pos2(left + 18.0, mid_y + 5.0),
                    egui::Align2::LEFT_TOP,
                    sub,
                    egui::FontId::new(11.0, egui::FontFamily::Proportional),
                    with_alpha(theme::DIM, self.alpha),
                );
            }

            // Right side — UPPERCASE mono badge or keyboard hint
            let right_x = rect.right() - 12.0;

            if self.selected {
                let hint = match self.result.kind {
                    ResultKind::App => "enter · ctrl+enter admin",
                    ResultKind::File => "enter · ctrl+enter folder",
                    ResultKind::Clipboard => "enter · paste",
                    ResultKind::Calculator => "enter · copy",
                    ResultKind::Action => "enter · run",
                    ResultKind::Folder => "enter · open",
                };
                painter.text(
                    egui::pos2(right_x, mid_y),
                    egui::Align2::RIGHT_CENTER,
                    hint,
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    with_alpha(theme::DIM, self.alpha),
                );
            } else {
                let badge = match self.result.kind {
                    ResultKind::App => "app",
                    ResultKind::File => {
                        let ext = self.result.subtitle.rsplit('.').next().unwrap_or("file");
                        if ext.len() <= 4 { ext } else { "file" }
                    }
                    ResultKind::Folder => "dir",
                    ResultKind::Clipboard => "clip",
                    ResultKind::Calculator => "calc",
                    ResultKind::Action => "cmd",
                };
                painter.text(
                    egui::pos2(right_x, mid_y),
                    egui::Align2::RIGHT_CENTER,
                    badge.to_uppercase(),
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    with_alpha(theme::DIM, self.alpha),
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
