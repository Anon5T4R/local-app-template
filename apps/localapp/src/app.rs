//! The window: title bar (when the app draws it), menus, the demonstration page, and the About
//! window that fronts the update check.
//!
//! Everything a born app adds goes in [`LocalApp::page`] (the central panel) and in
//! [`LocalApp::logic`] (work that is not drawing) - the rest here is the frame the suite shares:
//! the header bar both ways (desktop-drawn or app-drawn), the menu bar, the About window, the update
//! prompt and the file picker's answers.

use std::path::PathBuf;

use craft_update::updater::{Action, Status, Updater};

use crate::i18n;
use crate::pickers::{Picked, Pickers};
use crate::suite_entry::APP;
use crate::tl;

/// Extensions the demonstration picker offers; a born app replaces them with its own formats.
const DEMO_EXTENSIONS: &[&str] = &["txt", "md", "json"];

pub struct LocalApp {
    updater: Updater,
    pickers: Pickers,
    /// Who draws the window frame (probed before the window was created), so an app-drawn frame
    /// gets its header bar with the desktop's button layout.
    frame: craft_window::Frame,
    about_open: bool,
    /// The `language` setting: [`i18n::AUTO`] or a code from [`i18n::LANGUAGES`].
    language: String,
    /// The last path the demonstration picker returned.
    picked: Option<PathBuf>,
}

impl LocalApp {
    pub fn new(cc: &eframe::CreationContext<'_>, frame: craft_window::Frame) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        // A born app reads this setting from its own preferences file here.
        let language = i18n::AUTO.to_string();
        i18n::set_language(&language);
        Self { updater: Updater::new(&APP, env!("CARGO_PKG_VERSION")), pickers: Pickers::default(), frame, about_open: false, language, picked: None }
    }

    /// The menu bar: the app's commands (a born app grows File, Edit, . here) and Help.
    fn menus(&mut self, ui: &mut egui::Ui) {
        let mut quit = false;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(tl!("File"), |ui| {
                if ui.button(tl!("Open a file.")).clicked() {
                    self.open_file(ui.ctx());
                    ui.close();
                }
                ui.separator();
                if ui.button(tl!("Quit Local App")).clicked() {
                    quit = true;
                    ui.close();
                }
            });
            ui.menu_button(tl!("Help"), |ui| {
                if ui.button(tl!("Check for Updates.")).clicked() {
                    // The outcome shows in the About window, which is where a check belongs.
                    self.about_open = true;
                    self.updater.check_now(ui.ctx());
                    ui.close();
                }
                if ui.button(tl!("About Local App")).clicked() {
                    self.about_open = true;
                    ui.close();
                }
            });
        });
        if quit {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// The demonstration page: the minimum a born app replaces with its own interface.
    fn page(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.heading(tl!("Local App"));
        ui.add_space(4.0);
        ui.label(tl!(
            "This is the Local suite's desktop template: a window, a menu bar, an About window with an update check, translated interface, native file pickers and the craft-fonts faces. Replace this page with your app."
        ));
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.add_enabled(!self.pickers.busy(), egui::Button::new(tl!("Open a file."))).clicked() {
                self.open_file(ui.ctx());
            }
            if ui.add_enabled(!self.pickers.busy(), egui::Button::new(tl!("Choose a folder."))).clicked() {
                self.pickers.ask_folder(ui.ctx(), tl!("Choose a folder."), None);
            }
            match &self.picked {
                Some(path) => ui.label(i18n::tr_fmt("picked: {path}", &[("path", &path.display().to_string())])),
                None => ui.label(tl!("nothing picked yet")),
            };
        });

        ui.add_space(12.0);
        self.language_picker(ui);
    }

    /// The interface language, straight from [`i18n::LANGUAGES`] (no hard-coded list here: a new
    /// catalog only adds a row there).
    fn language_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(tl!("Language"));
            let current = i18n::code();
            egui::ComboBox::from_id_salt("language").selected_text(i18n::name_of(current)).show_ui(ui, |ui| {
                for (code, name) in i18n::languages() {
                    if ui.selectable_label(code == current, name).clicked() && code != current {
                        self.language = code.to_string();
                        i18n::set_lang(i18n::resolve(&self.language));
                    }
                }
            });
            if self.language == i18n::AUTO {
                ui.label(tl!("(following the system)"));
            }
        });
    }

    /// The app-drawn title bar, when the desktop leaves the frame to the app (GNOME on Wayland;
    /// see `craft-window`). The empty part drags the window; the buttons sit in the desktop's
    /// order, and the same strip carries the menus.
    fn header_bar(&mut self, ui: &mut egui::Ui, layout: &craft_window::Layout) {
        egui::Panel::top("title_bar").show(ui, |ui| {
            craft_window::title_bar::empty_area(ui, ui.max_rect());
            ui.horizontal(|ui| {
                for button in &layout.left {
                    window_button(ui, *button);
                }
                self.menus(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for button in layout.right.iter().rev() {
                        window_button(ui, *button);
                    }
                });
            });
        });
    }

    fn about(&mut self, ui: &mut egui::Ui) {
        let mut open = self.about_open;
        egui::Window::new(tl!("About Local App"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.heading(APP.name);
                ui.label(i18n::tr_fmt("version {version}", &[("version", env!("CARGO_PKG_VERSION"))]));
                ui.label(APP.tagline);
                ui.add_space(6.0);
                ui.hyperlink_to(APP.repo, APP.repo_url());
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let checking = matches!(self.updater.status(), Status::Checking);
                    if ui.add_enabled(!checking, egui::Button::new(tl!("Check for Updates."))).clicked() {
                        self.updater.check_now(ui.ctx());
                    }
                    match self.updater.status() {
                        Status::Idle => {}
                        Status::Checking => {
                            ui.spinner();
                        }
                        Status::UpToDate => {
                            ui.label(tl!("You are on the newest version."));
                        }
                        Status::Failed(reason) => {
                            ui.label(i18n::tr_fmt("checking for updates failed: {reason}", &[("reason", reason)]));
                        }
                    };
                });
            });
        self.about_open = open;
    }

    fn open_file(&self, ctx: &egui::Context) {
        if !self.pickers.ask_file(ctx, tl!("Open a file."), Some((tl!("Text files"), DEMO_EXTENSIONS))) {
            log::debug!("a file picker is already open");
        }
    }
}

impl eframe::App for LocalApp {
    /// Work that is not drawing: the check at start (once), the state a finished check or
    /// installation leaves, and the picker's answers.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.updater.check_at_start(ctx);
        self.updater.tick(ctx);
        for picked in self.pickers.take() {
            match picked {
                Picked::File(path) | Picked::Folder(path) => self.picked = Some(path),
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.frame.clone() {
            craft_window::Frame::App(layout) => self.header_bar(ui, &layout),
            // The desktop draws the frame; the menus still need a strip inside the window.
            craft_window::Frame::Desktop => {
                egui::Panel::top("menu_bar").show(ui, |ui| self.menus(ui));
            }
        }

        egui::CentralPanel::default_margins().show(ui, |ui| self.page(ui));

        if self.about_open {
            self.about(ui);
        }

        // The update prompt and its progress, drawn last so it sits above the window's content.
        match self.updater.show(ui.ctx()) {
            Action::None => {}
            Action::OpenUrl(url) => {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
            // The update is installed: close through the normal path, and the helper restarts us.
            Action::Close => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}

/// One window button of the app-drawn header bar.
fn window_button(ui: &mut egui::Ui, button: craft_window::Button) {
    let (label, tooltip) = match button {
        craft_window::Button::Minimize => ("-", tl!("Minimize")),
        craft_window::Button::Maximize => ("[]", tl!("Maximize")),
        craft_window::Button::Close => ("x", tl!("Close")),
    };
    if ui.button(label).on_hover_text(tooltip).clicked() {
        let command = match button {
            craft_window::Button::Minimize => egui::ViewportCommand::Minimized(true),
            craft_window::Button::Maximize => {
                let maximized = ui.input(|i| i.viewport().maximized).unwrap_or(false);
                egui::ViewportCommand::Maximized(!maximized)
            }
            craft_window::Button::Close => egui::ViewportCommand::Close,
        };
        ui.ctx().send_viewport_cmd(command);
    }
}
