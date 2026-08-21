//! Reflex - faster than thinking.
//!
//! Instant PC runtime for Windows. Alt+Space opens the command palette.
//! Search files, launch apps, clipboard history, calculator, system actions.

#![windows_subsystem = "windows"]

mod actions;
mod clipboard;
mod config;
mod platform;
mod search;
mod ui;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;

use eframe::egui;

use actions::{calculator, system};
use clipboard::ClipboardHistory;
use config::Config;
use platform::AppEntry;
use search::fuzzy::FuzzyMatcher;
use search::index::FileIndex;
use search::ranker::{SearchResult, ResultKind, ResultAction, rank_results};
use ui::palette::PaletteState;

const PALETTE_WIDTH: f32 = 660.0;

struct ReflexApp {
    palette: PaletteState,
    hotkey_triggered: Arc<AtomicBool>,
    apps: Vec<AppEntry>,
    file_index: FileIndex,
    clipboard: ClipboardHistory,
    config: Config,
    last_query: String,
    last_filter: ui::palette::Filter,
    fuzzy: FuzzyMatcher,
    launch_counts: Arc<Mutex<HashMap<String, u32>>>,
    drag_pos: Option<egui::Pos2>,
    window_pos: egui::Pos2,
}

impl ReflexApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::apply(&cc.egui_ctx);

        let config = Config::load();
        let hotkey_triggered = Arc::new(AtomicBool::new(false));
        platform::register_hotkey(hotkey_triggered.clone());

        let launch_counts = Arc::new(Mutex::new(Self::load_launch_counts()));
        let mut apps = platform::discover_apps();
        // Apply persisted launch counts
        {
            let counts = launch_counts.lock().unwrap();
            for app in &mut apps {
                if let Some(&c) = counts.get(&app.name.to_lowercase()) {
                    app.launch_count = c;
                }
            }
        }

        let file_index = FileIndex::new();
        let index_dirs: Vec<PathBuf> = config.index_dirs.iter().map(PathBuf::from).collect();
        file_index.start_indexing(index_dirs);

        let clipboard_hist = ClipboardHistory::new();
        clipboard_hist.start_watching();

        let mut palette = PaletteState::new();
        palette.show();

        #[cfg(target_os = "windows")]
        platform::apply_rounded_corners();

        Self {
            palette,
            hotkey_triggered,
            apps,
            file_index,
            clipboard: clipboard_hist,
            config,
            last_query: String::new(),
            last_filter: ui::palette::Filter::All,
            fuzzy: FuzzyMatcher::new(),
            launch_counts,
            drag_pos: None,
            window_pos: egui::pos2(0.0, 0.0),
        }
    }

    fn load_launch_counts() -> HashMap<String, u32> {
        let path = config::launch_counts_path();
        match std::fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => HashMap::new(),
        }
    }

    fn save_launch_count(&self, app_name: &str) {
        let mut counts = self.launch_counts.lock().unwrap();
        let key = app_name.to_lowercase();
        let entry = counts.entry(key).or_insert(0);
        *entry += 1;
        let path = config::launch_counts_path();
        if let Ok(json) = serde_json::to_string(&*counts) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, json);
        }
        drop(counts);
    }

    fn do_search(&mut self) {
        let query = self.palette.query.trim().to_string();
        let filter = self.palette.filter;

        // Re-search if query or filter changed
        if query == self.last_query && filter == self.last_filter { return; }
        self.last_query = query.clone();
        self.last_filter = filter;

        let mut results: Vec<SearchResult> = Vec::new();
        let q_lower = query.to_lowercase();

        // Empty query - show recent/top apps
        if query.is_empty() {
            for app in self.apps.iter().take(6) {
                results.push(SearchResult {
                    title: app.name.clone(),
                    subtitle: app.path.to_string_lossy().to_string(),
                    kind: ResultKind::App,
                    score: 3000 + app.launch_count * 100,
                    action: ResultAction::Launch(app.path.clone()),
                });
            }

            for clip in self.clipboard.entries().into_iter().take(8) {
                let preview: String = clip.text.chars().take(80).collect();
                results.push(SearchResult {
                    title: preview,
                    subtitle: "Clipboard".to_string(),
                    kind: ResultKind::Clipboard,
                    score: 1500,
                    action: ResultAction::PasteClip(clip.text),
                });
            }

            rank_results(&mut results);
            self.palette.results = results;
            self.palette.selected = 0;
            return;
        }

        // Calculator
        if let Some(calc_result) = calculator::evaluate(&query) {
            results.push(SearchResult {
                title: calc_result.clone(),
                subtitle: query.clone(),
                kind: ResultKind::Calculator,
                score: 10000,
                action: ResultAction::CopyResult(calc_result.trim_start_matches("= ").to_string()),
            });
        }

        // System actions
        results.extend(system::search_actions(&query));

        // Apps (using nucleo fuzzy matching)
        for app in &self.apps {
            let name_lower = app.name.to_lowercase();
            let score = if name_lower.starts_with(&q_lower) {
                Some(4000)
            } else if name_lower.contains(&q_lower) {
                Some(3000)
            } else if let Some(nucleo_score) = self.fuzzy.score(&query, &app.name) {
                if nucleo_score > 300 { Some(2000 + nucleo_score / 10) } else { None }
            } else {
                None
            };

            if let Some(base_score) = score {
                results.push(SearchResult {
                    title: app.name.clone(),
                    subtitle: app.path.to_string_lossy().to_string(),
                    kind: ResultKind::App,
                    score: base_score + app.launch_count * 100,
                    action: ResultAction::Launch(app.path.clone()),
                });
            }
        }

        // Clipboard
        let clips = self.clipboard.search(&query, 5);
        for clip in clips {
            let preview: String = clip.text.chars().take(80).collect();
            results.push(SearchResult {
                title: preview,
                subtitle: "Clipboard".to_string(),
                kind: ResultKind::Clipboard,
                score: 1500,
                action: ResultAction::PasteClip(clip.text),
            });
        }

        // Files (2+ chars)
        if query.len() >= 2 {
            let file_results = self.file_index.search(&query, 8);
            for file in file_results {
                let kind = if file.is_dir { ResultKind::Folder } else { ResultKind::File };
                let action = if file.is_dir {
                    ResultAction::OpenFolder(file.path.clone())
                } else {
                    ResultAction::OpenFile(file.path.clone())
                };

                results.push(SearchResult {
                    title: file.name,
                    subtitle: file.path.parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    kind,
                    score: 500,
                    action,
                });
            }
        }

        rank_results(&mut results);
        results.truncate(self.config.max_results);
        self.palette.results = results;
        self.palette.selected = 0;
    }

    fn execute_action(&mut self, action: ResultAction, ctrl: bool) {
        match action {
            ResultAction::Launch(ref path) => {
                platform::launch_app(path, ctrl);
                // Track launch count for frequency ranking
                if let Some(app) = self.apps.iter().find(|a| &a.path == path) {
                    self.save_launch_count(&app.name);
                    if let Some(entry) = self.apps.iter_mut().find(|a| &a.path == path) {
                        entry.launch_count += 1;
                    }
                }
            }
            ResultAction::OpenFile(ref path) => {
                if ctrl {
                    if let Some(parent) = path.parent() {
                        let _ = open::that(parent);
                    }
                } else {
                    let _ = open::that(path);
                }
            }
            ResultAction::OpenFolder(ref path) => {
                let _ = open::that(path);
            }
            ResultAction::PasteClip(ref text) => {
                self.clipboard.paste(text);
            }
            ResultAction::CopyResult(ref text) => {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(text);
                }
            }
            ResultAction::SystemAction(ref action_name) => {
                match action_name.as_str() {
                    "quit" => std::process::exit(0),
                    "ip" => {
                        let ip = platform::get_ip();
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(&ip);
                        }
                    }
                    "terminal" => {
                        let _ = std::process::Command::new("wt.exe").spawn()
                            .or_else(|_| std::process::Command::new("cmd.exe").spawn());
                    }
                    "screenshot" => {
                        let _ = std::process::Command::new("snippingtool.exe").spawn();
                    }
                    other => {
                        let _ = platform::system_action(other);
                    }
                }
            }
        }
        self.palette.hide();
    }

    fn show_palette(&mut self, ctx: &egui::Context) {
        self.palette.show();
        self.last_query = String::new();
        let h = self.palette.ideal_height();
        let monitor = ctx.input(|i| i.viewport().monitor_size).unwrap_or(egui::vec2(1920.0, 1080.0));
        // Center horizontally, sit at ~22% from top
        let x = (monitor.x - PALETTE_WIDTH) / 2.0;
        let y = (monitor.y - h) * 0.22;
        self.window_pos = egui::pos2(x, y);
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_pos));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(PALETTE_WIDTH, h)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn hide_palette(&mut self, ctx: &egui::Context) {
        self.palette.hide();
        // Don't resize here — the alpha fade + target_h in update() handles it
    }
}

impl eframe::App for ReflexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Animate palette alpha
        self.palette.tick_alpha();

        if self.hotkey_triggered.swap(false, Ordering::SeqCst) {
            if self.palette.visible {
                self.hide_palette(ctx);
            } else {
                self.show_palette(ctx);
            }
        }

        // Drag: click-and-hold in the input area (top 50px) to move the window
        if self.palette.visible {
            let (pos, delta, pressed) = ctx.input(|i| {
                (i.pointer.interact_pos(), i.pointer.delta(), i.pointer.button_pressed(egui::PointerButton::Primary))
            });
            if let Some(p) = pos {
                if p.y < 50.0 && pressed {
                    if self.drag_pos.is_none() {
                        self.drag_pos = Some(p);
                    }
                    self.window_pos += delta;
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_pos));
                } else {
                    self.drag_pos = None;
                }
            }
        } else {
            self.drag_pos = None;
        }

        ctx.request_repaint_after(Duration::from_millis(30));

        if !self.palette.visible && self.palette.alpha < 0.01 {
            // Fully faded out — draw nothing (transparent window)
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(egui::Color32::TRANSPARENT))
                .show(ctx, |_ui| {});
            return;
        }

        self.do_search();

        let ideal_h = self.palette.ideal_height();
        let target_h = if self.palette.visible { ideal_h } else { ideal_h * self.palette.alpha };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(PALETTE_WIDTH, target_h.max(1.0))));

        let ctrl_held = ctx.input(|i| i.modifiers.ctrl);
        let file_count = self.file_index.count();
        let indexing = self.file_index.is_indexing();

        if let Some(action) = ui::palette::draw(ctx, &mut self.palette, ctrl_held, file_count, indexing) {
            self.execute_action(action, ctrl_held);
            self.hide_palette(ctx);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([PALETTE_WIDTH, 80.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_title("Reflex"),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Reflex",
        options,
        Box::new(|cc| {
            // Load better system fonts
            load_fonts(&cc.egui_ctx);
            Ok(Box::new(ReflexApp::new(&cc)))
        }),
    )
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Proportional: Segoe UI (Windows-native, clean)
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\seguiui.ttf") {
        fonts.font_data.insert(
            "segoe_ui".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["segoe_ui".to_owned(), "Arial".to_owned()],
        );
    }

    // Monospace: Cascadia Mono (Windows Terminal font, modern)
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\CascadiaMono.ttf") {
        fonts.font_data.insert(
            "cascadia".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        fonts.families.insert(
            egui::FontFamily::Monospace,
            vec!["cascadia".to_owned(), "Consolas".to_owned()],
        );
    }

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_matches_subsequence() {
        let mut fm = FuzzyMatcher::new();
        assert!(fm.score("vs", "Visual Studio Code").is_some());
        assert!(fm.score("vsc", "Visual Studio Code").is_some());
        assert!(fm.score("xyz", "Visual Studio Code").is_none());
    }

    #[test]
    fn test_fuzzy_prefers_prefix() {
        let mut fm = FuzzyMatcher::new();
        let prefix = fm.score("chr", "Chrome").unwrap();
        let middle = fm.score("hrm", "Chrome").unwrap();
        assert!(prefix > middle);
    }
}
