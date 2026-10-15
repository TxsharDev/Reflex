//! Reflex: a command palette for Windows. Alt+Space opens it.

#![windows_subsystem = "windows"]

mod actions;
mod clipboard;
mod config;
mod platform;
mod search;
mod ui;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::time::Duration;

use eframe::egui;

use actions::calculator;
use actions::system::{self, Command};
use clipboard::ClipboardHistory;
use config::Config;
use platform::{AppEntry, Window};
use search::fuzzy::FuzzyMatcher;
use search::index::FileIndex;
use search::ranker::{ResultAction, ResultKind, SearchResult, rank};
use ui::palette::{Event, PaletteState, Status};

const WIDTH: f32 = 660.0;
const FADE_SECONDS: f32 = 0.12;
const RECENT_APPS: usize = 6;
const RECENT_CLIPS: usize = 8;

struct Reflex {
    palette: PaletteState,
    window: Option<Window>,
    hotkey_pressed: Arc<AtomicBool>,
    hotkey_ok: bool,
    /// Window to hand focus back to when the palette closes.
    previous_focus: Arc<AtomicIsize>,
    had_focus: bool,
    parked: bool,
    height: f32,
    config: Config,
    apps: Vec<AppEntry>,
    launch_counts: HashMap<String, u32>,
    files: FileIndex,
    clipboard: ClipboardHistory,
    fuzzy: FuzzyMatcher,
    /// Query, clipboard version and file count the current results reflect.
    searched: Option<(String, u64, usize)>,
}

impl Reflex {
    fn new(cc: &eframe::CreationContext<'_>, launched_from: isize) -> Self {
        ui::theme::load_fonts(&cc.egui_ctx);
        ui::theme::apply(&cc.egui_ctx);

        let config = Config::load();
        let window = Window::from_handle(cc);
        let hotkey_pressed = Arc::new(AtomicBool::new(false));
        let previous_focus = Arc::new(AtomicIsize::new(launched_from));
        let hotkey_ok = window.is_some_and(|w| {
            w.round_corners();
            platform::register_hotkey(
                cc.egui_ctx.clone(),
                w,
                hotkey_pressed.clone(),
                previous_focus.clone(),
            )
        });

        let launch_counts = config::load_launch_counts();
        let mut apps = platform::discover_apps();
        for app in &mut apps {
            app.launch_count = launch_counts
                .get(&app.name.to_lowercase())
                .copied()
                .unwrap_or(0);
        }

        let roots = config.index_dirs.iter().map(PathBuf::from).collect();
        let mut reflex = Self {
            palette: PaletteState::new(),
            window,
            hotkey_pressed,
            hotkey_ok,
            previous_focus,
            had_focus: false,
            parked: false,
            height: 0.0,
            config,
            apps,
            launch_counts,
            files: FileIndex::start(roots),
            clipboard: ClipboardHistory::start(),
            fuzzy: FuzzyMatcher::new(),
            searched: None,
        };
        reflex.show();
        reflex
    }

    fn show(&mut self) {
        self.palette.open();
        self.searched = None;
        self.had_focus = false;
        self.parked = false;
        // Windows only lets a process take focus in response to user input.
        // Without focus (e.g. started at login) the palette would float
        // unusable over everything, so stay hidden until the hotkey.
        if !self.window.is_some_and(|w| w.present()) {
            self.palette.visible = false;
        }
    }

    fn hide(&mut self, restore_focus: bool) {
        self.palette.visible = false;
        self.palette.armed = None;
        if restore_focus {
            platform::restore_focus(self.previous_focus.load(Ordering::SeqCst));
        }
    }

    /// Re-runs the search when the query changed or new files or clips arrived.
    fn refresh(&mut self) {
        let query = self.palette.query.trim().to_string();
        let key = (query.clone(), self.clipboard.version(), self.files.count());
        if self.searched.as_ref() == Some(&key) {
            return;
        }
        let new_query = self.searched.as_ref().is_none_or(|(q, _, _)| *q != query);
        self.searched = Some(key);

        let mut results = if query.is_empty() {
            self.recent()
        } else {
            self.search(&query)
        };
        rank(&mut results);
        self.palette.set_results(results, new_query);
    }

    /// What the palette shows before anything is typed.
    fn recent(&self) -> Vec<SearchResult> {
        let mut used: Vec<&AppEntry> = self.apps.iter().filter(|a| a.launch_count > 0).collect();
        used.sort_by_key(|a| std::cmp::Reverse(a.launch_count));

        let apps = used
            .into_iter()
            .take(RECENT_APPS)
            .map(|app| app_result(app, 3000));
        let clips = self
            .clipboard
            .recent(RECENT_CLIPS)
            .into_iter()
            .map(clip_result);
        apps.chain(clips).collect()
    }

    fn search(&mut self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if let Some(value) = calculator::evaluate(query) {
            results.push(SearchResult {
                title: format!("= {value}"),
                subtitle: query.to_string(),
                kind: ResultKind::Calculator,
                score: 10_000,
                action: ResultAction::Copy(value),
            });
        }

        results.extend(system::search(query));

        let q = query.to_lowercase();
        for app in &self.apps {
            let name = app.name.to_lowercase();
            let score = if name.starts_with(&q) {
                Some(4000)
            } else if name.contains(&q) {
                Some(3000)
            } else {
                self.fuzzy
                    .score(query, &app.name)
                    .filter(|s| *s > 300)
                    .map(|s| 2000 + s / 10)
            };
            if let Some(score) = score {
                results.push(app_result(app, score));
            }
        }

        results.extend(
            self.clipboard
                .search(query, 20)
                .into_iter()
                .map(clip_result),
        );

        if query.chars().count() >= 2 {
            for file in self.files.search(query, 30) {
                let parent = file
                    .path
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                results.push(SearchResult {
                    title: file.name,
                    subtitle: parent,
                    kind: if file.is_dir {
                        ResultKind::Folder
                    } else {
                        ResultKind::File
                    },
                    score: 500,
                    action: ResultAction::Open(file.path),
                });
            }
        }

        results
    }

    fn execute(&mut self, ctx: &egui::Context, action: ResultAction, alternate: bool) {
        match action {
            ResultAction::Launch(path) => {
                platform::shell_open(&path, alternate);
                self.count_launch(&path);
                self.hide(false);
            }
            ResultAction::Open(path) => {
                if alternate {
                    platform::reveal_in_explorer(&path);
                } else {
                    platform::shell_open(&path, false);
                }
                self.hide(false);
            }
            ResultAction::Paste(text) => {
                clipboard::copy(&text);
                self.hide(true);
                if !alternate {
                    after(Duration::from_millis(80), platform::send_paste);
                }
            }
            ResultAction::Copy(text) => {
                clipboard::copy(&text);
                self.hide(true);
            }
            ResultAction::Run(command) => {
                if command.needs_confirmation()
                    && self.palette.armed != Some(ResultAction::Run(command))
                {
                    self.palette.armed = Some(ResultAction::Run(command));
                    return;
                }
                self.run(ctx, command);
            }
        }
    }

    fn run(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Command::Terminal => {
                platform::open_terminal();
                self.hide(false);
            }
            Command::CopyIp => {
                if let Some(ip) = platform::local_ip() {
                    clipboard::copy(&ip);
                }
                self.hide(true);
            }
            Command::Screenshot => {
                self.hide(true);
                // Let the fade finish so the palette is not in the capture.
                after(Duration::from_millis(250), || {
                    platform::shell_open(Path::new("ms-screenclip:"), false)
                });
            }
            Command::Sleep => {
                self.hide(true);
                platform::sleep();
            }
            Command::Lock => {
                self.hide(true);
                platform::lock();
            }
            Command::Restart | Command::Shutdown => {
                self.hide(true);
                platform::shutdown(command == Command::Restart);
            }
            Command::EmptyRecycleBin => {
                self.hide(true);
                platform::empty_recycle_bin();
            }
        }
    }

    fn count_launch(&mut self, path: &Path) {
        let Some(app) = self.apps.iter_mut().find(|a| a.path == path) else {
            return;
        };
        app.launch_count += 1;
        self.launch_counts
            .insert(app.name.to_lowercase(), app.launch_count);
        config::save_launch_counts(&self.launch_counts);
    }

    fn animate(&mut self, ctx: &egui::Context) {
        let target = if self.palette.visible { 1.0 } else { 0.0 };
        let step = ctx.input(|i| i.stable_dt).min(0.05) / FADE_SECONDS;
        let alpha = &mut self.palette.alpha;
        if *alpha < target {
            *alpha = (*alpha + step).min(target);
        } else if *alpha > target {
            *alpha = (*alpha - step).max(target);
        }
        if *alpha != target {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for Reflex {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.hotkey_pressed.swap(false, Ordering::SeqCst) {
            if self.palette.visible {
                self.hide(true);
            } else {
                self.show();
            }
        }

        // Clicking another window closes the palette, like the Start menu.
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(false);
        if self.palette.visible {
            if focused {
                self.had_focus = true;
            } else if self.had_focus {
                self.hide(false);
            }
        }

        self.animate(ctx);

        if !self.palette.visible && self.palette.alpha == 0.0 {
            if !self.parked {
                self.parked = true;
                if let Some(window) = self.window {
                    window.park();
                }
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |_| {});
            return;
        }

        if self.palette.visible {
            self.refresh();
            // Keeps the file count and new clips current while open.
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        let status = Status {
            files: self.files.count(),
            indexing: self.files.is_indexing(),
            hotkey_ok: self.hotkey_ok,
        };
        let (event, height) =
            ui::palette::draw(ctx, &mut self.palette, &status, self.config.max_results);

        if (height - self.height).abs() > 0.5 {
            self.height = height;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(WIDTH, height)));
        }

        match event {
            Some(Event::Execute { action, alternate }) => self.execute(ctx, action, alternate),
            Some(Event::Dismiss) => self.hide(true),
            None => {}
        }
    }
}

fn app_result(app: &AppEntry, score: u32) -> SearchResult {
    SearchResult {
        title: app.name.clone(),
        subtitle: String::new(),
        kind: ResultKind::App,
        score: score + app.launch_count * 100,
        action: ResultAction::Launch(app.path.clone()),
    }
}

fn clip_result(text: String) -> SearchResult {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let title = lines.next().unwrap_or_default().to_string();
    let more = lines.count();
    let subtitle = match more {
        0 => "Clipboard".to_string(),
        1 => "Clipboard · 1 more line".to_string(),
        n => format!("Clipboard · {n} more lines"),
    };
    SearchResult {
        title,
        subtitle,
        kind: ResultKind::Clipboard,
        score: 1500,
        action: ResultAction::Paste(text),
    }
}

fn after(delay: Duration, f: impl FnOnce() + Send + 'static) {
    std::thread::spawn(move || {
        std::thread::sleep(delay);
        f();
    });
}

fn main() -> eframe::Result<()> {
    if !platform::claim_single_instance() {
        return Ok(());
    }
    // Captured before our window exists and takes focus.
    let launched_from = platform::foreground();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Reflex")
            .with_inner_size([WIDTH, 120.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_resizable(false)
            .with_taskbar(false),
        ..Default::default()
    };

    eframe::run_native(
        "Reflex",
        options,
        Box::new(move |cc| Ok(Box::new(Reflex::new(cc, launched_from)))),
    )
}
