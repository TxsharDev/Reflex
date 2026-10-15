//! The palette window: search field, filter tabs, result rows, status line.

use eframe::egui::{self, Color32, FontFamily, FontId, Key, Modifiers, Sense};

use crate::search::ranker::{ResultAction, ResultKind, SearchResult};
use crate::ui::theme;

const ROW_HEIGHT: f32 = 44.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Apps,
    Files,
    Clipboard,
    Actions,
}

impl Filter {
    const ORDER: [Filter; 5] = [
        Filter::All,
        Filter::Apps,
        Filter::Files,
        Filter::Clipboard,
        Filter::Actions,
    ];

    fn label(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Apps => "apps",
            Filter::Files => "files",
            Filter::Clipboard => "clips",
            Filter::Actions => "actions",
        }
    }

    fn matches(self, kind: ResultKind) -> bool {
        match self {
            Filter::All => true,
            Filter::Apps => kind == ResultKind::App,
            Filter::Files => matches!(kind, ResultKind::File | ResultKind::Folder),
            Filter::Clipboard => kind == ResultKind::Clipboard,
            Filter::Actions => matches!(kind, ResultKind::Action | ResultKind::Calculator),
        }
    }

    fn step(self, forward: bool) -> Filter {
        let i = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        let n = Self::ORDER.len();
        Self::ORDER[if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        }]
    }
}

pub enum Event {
    /// Run an action. `alternate` is true for Ctrl+Enter / Ctrl+click.
    Execute {
        action: ResultAction,
        alternate: bool,
    },
    Dismiss,
}

/// Facts shown in the status line.
pub struct Status {
    pub files: usize,
    pub indexing: bool,
    pub hotkey_ok: bool,
}

pub struct PaletteState {
    pub query: String,
    pub visible: bool,
    pub alpha: f32,
    /// Action waiting for a second Enter before it runs.
    pub armed: Option<ResultAction>,
    results: Vec<SearchResult>,
    selected: usize,
    filter: Filter,
}

impl PaletteState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            visible: false,
            alpha: 0.0,
            armed: None,
            results: Vec::new(),
            selected: 0,
            filter: Filter::All,
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results.clear();
        self.selected = 0;
        self.filter = Filter::All;
        self.armed = None;
    }

    /// Replaces the results. A new query resets the selection; a background
    /// refresh (new files or clips) keeps it.
    pub fn set_results(&mut self, results: Vec<SearchResult>, new_query: bool) {
        self.results = results;
        if new_query {
            self.selected = 0;
            self.armed = None;
        }
    }

    fn shown(&self, max_rows: usize) -> Vec<&SearchResult> {
        self.results
            .iter()
            .filter(|r| self.filter.matches(r.kind))
            .take(max_rows)
            .collect()
    }

    fn select(&mut self, index: usize) {
        if index != self.selected {
            self.selected = index;
            self.armed = None;
        }
    }

    fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
        self.selected = 0;
        self.armed = None;
    }
}

/// Draws the palette and returns what the user asked for, plus the height
/// the content needs so the window can be sized to fit.
pub fn draw(
    ctx: &egui::Context,
    state: &mut PaletteState,
    status: &Status,
    max_rows: usize,
) -> (Option<Event>, f32) {
    let mut event = handle_keys(ctx, state, max_rows);
    let a = state.alpha;
    let ctrl = ctx.input(|i| i.modifiers.ctrl);
    let mut height = 0.0;

    let panel = egui::Frame::NONE
        .fill(theme::BG.gamma_multiply(a))
        .corner_radius(12)
        .stroke(egui::Stroke::new(1.0_f32, theme::BORDER.gamma_multiply(a)));

    egui::CentralPanel::default().frame(panel).show(ctx, |ui| {
        // Dragging any empty part of the palette moves the window.
        let background = ui.interact(ui.max_rect(), ui.id().with("drag"), Sense::drag());
        if background.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let w = ui.available_width();
        // The panel always spans the whole window, so measure the content
        // in its own scope rather than the panel.
        height = ui
            .scope(|ui| {
                search_field(ui, state, w, a);
                filter_tabs(ui, state, w, a);
                divider(ui, w, a);

                let shown: Vec<SearchResult> = state.shown(max_rows).into_iter().cloned().collect();
                if state.selected >= shown.len() {
                    state.selected = shown.len().saturating_sub(1);
                }

                if shown.is_empty() {
                    let message = if state.query.trim().is_empty() {
                        "apps, files, clipboard, math, commands"
                    } else {
                        "no results"
                    };
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(18, 12))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(message)
                                    .size(13.0)
                                    .color(theme::DIM.gamma_multiply(a)),
                            );
                        });
                } else {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            let pointer_moved = ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO);
                            for (i, result) in shown.iter().enumerate() {
                                let row = Row {
                                    result,
                                    selected: i == state.selected,
                                    armed: state.armed.as_ref() == Some(&result.action),
                                    alpha: a,
                                };
                                let response = ui.add_sized([w - 12.0, ROW_HEIGHT], row);
                                if response.hovered() && pointer_moved {
                                    state.select(i);
                                }
                                if response.clicked() {
                                    state.select(i);
                                    event = Some(Event::Execute {
                                        action: result.action.clone(),
                                        alternate: ctrl,
                                    });
                                }
                            }
                        });
                }

                footer(ui, status, w, a);
            })
            .response
            .rect
            .height();
    });

    (event, height)
}

/// Keys are consumed before the text field sees them, so arrows and Tab
/// drive the list instead of moving the cursor or focus.
fn handle_keys(ctx: &egui::Context, state: &mut PaletteState, max_rows: usize) -> Option<Event> {
    let shown = state.shown(max_rows).len();
    let (escape, up, down, tab, back_tab, ctrl_enter, enter) = ctx.input_mut(|i| {
        (
            i.consume_key(Modifiers::NONE, Key::Escape),
            i.consume_key(Modifiers::NONE, Key::ArrowUp),
            i.consume_key(Modifiers::NONE, Key::ArrowDown),
            i.consume_key(Modifiers::NONE, Key::Tab),
            i.consume_key(Modifiers::SHIFT, Key::Tab),
            i.consume_key(Modifiers::CTRL, Key::Enter),
            i.consume_key(Modifiers::NONE, Key::Enter),
        )
    });

    if escape {
        return Some(Event::Dismiss);
    }
    if up && state.selected > 0 {
        state.select(state.selected - 1);
    }
    if down && state.selected + 1 < shown {
        state.select(state.selected + 1);
    }
    if tab || back_tab {
        state.set_filter(state.filter.step(tab));
    }
    if enter || ctrl_enter {
        let action = state
            .shown(max_rows)
            .get(state.selected)
            .map(|r| r.action.clone())?;
        return Some(Event::Execute {
            action,
            alternate: ctrl_enter,
        });
    }
    None
}

fn search_field(ui: &mut egui::Ui, state: &mut PaletteState, w: f32, a: f32) {
    egui::Frame::NONE
        .fill(theme::INPUT_BG.gamma_multiply(a))
        .corner_radius(egui::CornerRadius {
            nw: 12,
            ne: 12,
            sw: 0,
            se: 0,
        })
        .inner_margin(egui::Margin::symmetric(18, 14))
        .show(ui, |ui| {
            ui.set_width(w - 36.0);
            let field = egui::TextEdit::singleline(&mut state.query)
                .font(FontId::proportional(20.0))
                .text_color(theme::TEXT.gamma_multiply(a))
                .hint_text(
                    egui::RichText::new("Search")
                        .color(theme::DIM.gamma_multiply(a))
                        .size(20.0),
                )
                .frame(false)
                .desired_width(w - 36.0)
                .return_key(None);
            ui.add(field).request_focus();
        });
}

fn filter_tabs(ui: &mut egui::Ui, state: &mut PaletteState, w: f32, a: f32) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(14, 5))
        .show(ui, |ui| {
            ui.set_width(w - 28.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                for filter in Filter::ORDER {
                    let active = state.filter == filter;
                    let color = if active { theme::ACCENT } else { theme::DIM };
                    let label = egui::RichText::new(filter.label())
                        .size(11.0)
                        .family(FontFamily::Monospace)
                        .color(color.gamma_multiply(a));
                    let button = egui::Button::new(label)
                        .corner_radius(4)
                        .fill(if active {
                            theme::ACCENT.gamma_multiply(0.08 * a)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if active {
                            egui::Stroke::new(0.5_f32, theme::ACCENT.gamma_multiply(0.3 * a))
                        } else {
                            egui::Stroke::NONE
                        });
                    if ui.add(button).clicked() {
                        state.set_filter(filter);
                    }
                }
            });
        });
}

fn divider(ui: &mut egui::Ui, w: f32, a: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, theme::BORDER.gamma_multiply(a));
}

fn footer(ui: &mut egui::Ui, status: &Status, w: f32, a: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 24.0), Sense::hover());
    let painter = ui.painter();
    let font = FontId::monospace(9.0);
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(10.0, 0.0),
            rect.right_top() - egui::vec2(10.0, 0.0),
        ],
        egui::Stroke::new(0.5_f32, theme::BORDER.gamma_multiply(a)),
    );

    let (left, left_color) = if status.hotkey_ok {
        ("reflex".to_string(), theme::ACCENT_DIM)
    } else {
        ("alt+space is taken by another app".to_string(), theme::RED)
    };
    painter.text(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        left,
        font.clone(),
        left_color.gamma_multiply(a),
    );

    let files = if status.indexing {
        format!("indexing · {} files", status.files)
    } else {
        format!("{} files", status.files)
    };
    painter.text(
        egui::pos2(rect.right() - 12.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        files,
        font,
        theme::DIM.gamma_multiply(a),
    );
}

struct Row<'a> {
    result: &'a SearchResult,
    selected: bool,
    armed: bool,
    alpha: f32,
}

impl Row<'_> {
    fn hint(&self) -> &'static str {
        if self.armed {
            return "enter again to confirm";
        }
        match self.result.kind {
            ResultKind::App => "enter open · ctrl+enter admin",
            ResultKind::File | ResultKind::Folder => "enter open · ctrl+enter show in folder",
            ResultKind::Clipboard => "enter paste · ctrl+enter copy",
            ResultKind::Calculator => "enter copy",
            ResultKind::Action => "enter run",
        }
    }

    fn badge(&self) -> String {
        match (&self.result.kind, &self.result.action) {
            (ResultKind::File, ResultAction::Open(path)) => path
                .extension()
                .map(|e| e.to_string_lossy().to_uppercase())
                .filter(|e| e.len() <= 4)
                .unwrap_or_else(|| "FILE".into()),
            (ResultKind::App, _) => "APP".into(),
            (ResultKind::Folder, _) => "DIR".into(),
            (ResultKind::Clipboard, _) => "CLIP".into(),
            (ResultKind::Calculator, _) => "CALC".into(),
            _ => "CMD".into(),
        }
    }
}

impl egui::Widget for Row<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::click());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let a = self.alpha;
        let painter = ui.painter();

        if self.selected {
            painter.rect_filled(rect, 8, theme::SELECTED.gamma_multiply(a));
        } else if response.hovered() {
            painter.rect_filled(rect, 8, theme::HOVER.gamma_multiply(a));
        }

        let dot = match self.result.kind {
            ResultKind::App => theme::GREEN,
            ResultKind::File => theme::BLUE,
            ResultKind::Folder => theme::YELLOW,
            ResultKind::Clipboard => theme::PURPLE,
            ResultKind::Calculator => theme::ACCENT,
            ResultKind::Action => theme::RED,
        };
        let left = rect.left() + 12.0;
        painter.circle_filled(
            egui::pos2(left + 4.0, rect.center().y),
            3.5,
            dot.gamma_multiply(a),
        );

        // Right side first, so the text knows how much room it has.
        let (right_text, right_color) = if self.selected {
            (
                self.hint().to_string(),
                if self.armed { theme::RED } else { theme::DIM },
            )
        } else {
            (self.badge(), theme::DIM)
        };
        let right = painter.text(
            egui::pos2(rect.right() - 12.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            right_text,
            FontId::monospace(9.0),
            right_color.gamma_multiply(a),
        );

        let text_left = left + 18.0;
        let text_width = (right.left() - 16.0 - text_left).max(0.0);
        let has_subtitle = !self.result.subtitle.is_empty();
        let title_y = if has_subtitle {
            rect.center().y - 9.0
        } else {
            rect.center().y - 8.5
        };
        single_line(
            ui,
            egui::pos2(text_left, title_y),
            &self.result.title,
            FontId::proportional(13.5),
            theme::TEXT.gamma_multiply(a),
            text_width,
        );
        if has_subtitle {
            single_line(
                ui,
                egui::pos2(text_left, rect.center().y + 5.0),
                &self.result.subtitle,
                FontId::proportional(11.0),
                theme::DIM.gamma_multiply(a),
                text_width,
            );
        }

        response
    }
}

/// Paints `text` on one line, cut with an ellipsis if it is wider than `max_width`.
fn single_line(
    ui: &egui::Ui,
    pos: egui::Pos2,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width);
    let galley = ui.fonts(|f| f.layout_job(job));
    ui.painter().galley(pos, galley, color);
}
