//! egui / eframe view layer: panels, widgets, and input routing.
//!
//! This file is the **immediate-mode UI**: every frame we describe the full interface.
//! There is no retained widget tree; egui diffs against the previous frame. For coursework,
//! the important ideas are: (1) panel order matters in egui (top/side before central),
//! (2) deferred actions (`next_dir`, `file_to_open`) collect clicks and apply after widgets,
//! (3) shell-like terminal input uses `lost_focus() && key_pressed(Enter)` for single-line
//! [`egui::TextEdit`] per upstream egui documentation.

use eframe::egui;

use crate::filesystem::read_utf8_file;
use crate::models::MyIDE;
use crate::theme::{self, Palette};

// ---------------------------------------------------------------------------
// Small helpers (pure functions, easy to unit test in isolation)
// ---------------------------------------------------------------------------

/// Line count, Unicode scalar count (`chars()`), and UTF-8 byte length for the status bar.
///
/// Rust `String` is always valid UTF-8; bytes ≥ chars for non-ASCII text.
fn buffer_stats(code: &str) -> (usize, usize, usize) {
    let lines = if code.is_empty() {
        0
    } else {
        code.lines().count()
    };
    let chars = code.chars().count();
    let bytes = code.len();
    (lines, chars, bytes)
}

/// Applies save shortcut and updates [`MyIDE::status_message`] for user feedback.
fn try_save(ide: &mut MyIDE) {
    match &ide.selected_file {
        None => {
            ide.status_message = "No file open — open or create a file first.".to_string();
        }
        Some(_) => match ide.save_current_file() {
            Ok(()) => ide.status_message = "File saved successfully.".to_string(),
            Err(e) => ide.status_message = format!("Save failed: {e}"),
        },
    }
}

// ---------------------------------------------------------------------------
// eframe application trait
// ---------------------------------------------------------------------------

impl eframe::App for MyIDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_ide_theme(ctx);

        // Global shortcut: Ctrl+S (egui maps "command" to Ctrl on Windows for many shortcuts;
        // we check `ctrl` explicitly for predictable behavior in the editor).
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && !i.modifiers.shift) {
            try_save(self);
        }

        let mut next_dir = None;
        let mut file_to_open = None;

        // --- Top bar: branding, cwd, quick actions (wrapped so buttons never overlap) ---
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::none()
                    .fill(Palette::MANTLE)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .stroke(egui::Stroke::new(1.0, Palette::RED_DIM)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    ui.label(
                        egui::RichText::new("IDE-Rust")
                            .strong()
                            .color(Palette::RED),
                    );
                    ui.separator();
                    if ui.small_button("About").clicked() {
                        self.show_about = true;
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("📍").color(Palette::RED));
                    ui.label(
                        egui::RichText::new(self.current_dir.display().to_string())
                            .monospace()
                            .color(Palette::TEXT),
                    );
                    ui.separator();
                    if ui
                        .small_button("Project root")
                        .on_hover_text(
                            "Reset working directory to the folder where the app was started",
                        )
                        .clicked()
                    {
                        self.reset_to_project_root();
                        self.status_message = "Working directory set to project root.".to_string();
                    }
                    if ui
                        .small_button("Copy path")
                        .on_hover_text("Copy current directory path to the clipboard")
                        .clicked()
                    {
                        ctx.copy_text(self.current_dir.display().to_string());
                        self.status_message = "Path copied to clipboard.".to_string();
                    }
                });
            });

        // --- Bottom terminal panel (resizable) ------------------------------------
        egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(true)
            .min_height(80.0)
            .default_height(280.0)
            .frame(
                egui::Frame::none()
                    .fill(Palette::MANTLE)
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.set_min_height(ui.available_height());
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Terminal")
                            .strong()
                            .size(15.0)
                            .color(Palette::TEXT),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Drag top edge to resize · Bash / PowerShell · ↑↓ history",
                        )
                        .small()
                        .color(Palette::SUBTEXT),
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                        if ui.small_button("🗑 Clear").clicked() {
                            self.terminal_output.clear();
                        }
                        if ui.small_button("check").clicked() {
                            self.run_terminal_line("cargo check");
                        }
                        if ui.small_button("build").clicked() {
                            self.run_terminal_line("cargo build");
                        }
                        if ui.small_button("fmt").clicked() {
                            self.run_terminal_line("cargo fmt");
                        }
                        if ui.small_button("run").clicked() {
                            self.run_terminal_line("cargo run");
                        }
                    });
                    ui.add_space(6.0);

                    egui::Frame::none()
                        .fill(Palette::CRUST)
                        .rounding(egui::Rounding::same(8.0))
                        .stroke(egui::Stroke::new(1.0, Palette::RED_DIM))
                        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                        .show(ui, |ui| {
                            let block_h = ui.available_height();
                            ui.set_min_height(block_h);

                            const INPUT_ROW: f32 = 44.0;
                            const GAP: f32 = 6.0;
                            let scroll_h = (block_h - INPUT_ROW - GAP).max(48.0);

                            egui::ScrollArea::vertical()
                                .max_height(scroll_h)
                                .auto_shrink([true, false])
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.terminal_output)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .text_color(Palette::TEXT)
                                            .interactive(false),
                                    );
                                });

                            ui.add_space(GAP);
                            // Fixed Run width + remaining width for input (no overlap on narrow windows).
                            const RUN_BTN_W: f32 = 76.0;
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("❯")
                                        .monospace()
                                        .color(Palette::RED),
                                );

                                let run_clicked = ui
                                    .add_sized(
                                        [RUN_BTN_W, 28.0],
                                        egui::Button::new(
                                            egui::RichText::new("▶ Run").color(Palette::ON_RED),
                                        )
                                        .fill(Palette::RED),
                                    )
                                    .clicked();

                                let input_w = ui.available_width().max(80.0);
                                let te_response = ui.add_sized(
                                    [input_w, 28.0],
                                    egui::TextEdit::singleline(&mut self.terminal_input)
                                        .id_source("ide_terminal_command")
                                        .font(egui::TextStyle::Monospace)
                                        .frame(true)
                                        .text_color(Palette::TEXT)
                                        .hint_text("Command (Enter or Run)…"),
                                );

                                if te_response.has_focus() {
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                        if !self.terminal_command_history.is_empty() {
                                            let idx = match self.terminal_history_browse {
                                                None => self.terminal_command_history.len() - 1,
                                                Some(i) => i.saturating_sub(1),
                                            };
                                            self.terminal_history_browse = Some(idx);
                                            self.terminal_input =
                                                self.terminal_command_history[idx].clone();
                                        }
                                    }
                                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                        if let Some(i) = self.terminal_history_browse {
                                            if i + 1 < self.terminal_command_history.len() {
                                                let ni = i + 1;
                                                self.terminal_history_browse = Some(ni);
                                                self.terminal_input =
                                                    self.terminal_command_history[ni].clone();
                                            } else {
                                                self.terminal_history_browse = None;
                                                self.terminal_input.clear();
                                            }
                                        }
                                    }
                                }

                                let enter_run = te_response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));

                                if enter_run || run_clicked {
                                    self.run_terminal_command();
                                    te_response.request_focus();
                                }
                            });
                        });
                });
            });

        // --- Left: file explorer --------------------------------------------------
        egui::SidePanel::left("explorer")
            .resizable(true)
            .default_width(270.0)
            .frame(
                egui::Frame::none()
                    .fill(Palette::MANTLE)
                    .inner_margin(egui::Margin::symmetric(12.0, 12.0))
                    .stroke(egui::Stroke::new(1.0, Palette::RED_DIM)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Project")
                        .strong()
                        .size(16.0)
                        .color(Palette::TEXT),
                );
                ui.add_space(8.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Filter files…")
                        .margin(egui::vec2(8.0, 6.0)),
                );

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    if ui.button("➕ New file").clicked() {
                        self.show_new_file_dialog = true;
                    }
                    if ui.button("⬆ Parent").clicked() {
                        next_dir = self.current_dir.parent().map(|p| p.to_path_buf());
                    }
                });
                ui.add_space(6.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for path in &self.files {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        if !self.search_query.is_empty()
                            && !name
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                        {
                            continue;
                        }

                        let is_selected = self.selected_file.as_ref() == Some(path);
                        let is_dir = path.is_dir();
                        let icon = if is_dir { "📁" } else { "📄" };

                        let mut text = egui::RichText::new(format!("{} {}", icon, name));
                        if name.ends_with(".rs") {
                            text = text.color(Palette::FILE_RS);
                        } else if name.ends_with(".cpp") || name.ends_with(".c") {
                            text = text.color(Palette::FILE_C);
                        } else if name.ends_with(".toml") {
                            text = text.color(Palette::FILE_TOML);
                        }

                        if ui.selectable_label(is_selected, text).clicked() {
                            if is_dir {
                                next_dir = Some(path.clone());
                            } else {
                                file_to_open = Some(path.clone());
                            }
                        }
                    }
                });
            });

        // --- Modal: new file ------------------------------------------------------
        if self.show_new_file_dialog {
            egui::Window::new("New file")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .frame(egui::Frame::window(&ctx.style()).rounding(egui::Rounding::same(10.0)))
                .show(ctx, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_file_name).hint_text("name.ext"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            let _ = self.create_file();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_file_dialog = false;
                        }
                    });
                });
        }

        // --- Modal: About (coursework / demo) -------------------------------------
        if self.show_about {
            egui::Window::new("About IDE-Rust")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(true)
                .default_width(420.0)
                .frame(egui::Frame::window(&ctx.style()).rounding(egui::Rounding::same(10.0)))
                .show(ctx, |ui| {
                    ui.heading("IDE-Rust");
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "A minimal Rust + egui desktop IDE demonstrating immediate-mode UI, \
                             integrated terminal (Bash / PowerShell fallback), and project-aware \
                             file operations.",
                        )
                        .color(Palette::SUBTEXT),
                    );
                    ui.add_space(8.0);
                    ui.label("Stack: Rust, eframe, egui.");
                    ui.add_space(4.0);
                    ui.label(
                        "Features: editor, explorer, built-in terminal commands, Cargo shortcuts, \
                             keyboard shortcuts, sorted file tree.",
                    );
                    ui.add_space(12.0);
                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }

        // Apply deferred navigation from explorer clicks
        if let Some(p) = next_dir {
            self.current_dir = p;
            self.refresh_files();
            self.search_query.clear();
        }
        if let Some(p) = file_to_open {
            match read_utf8_file(&p) {
                Ok(c) => {
                    self.code_buffer = c;
                    self.selected_file = Some(p);
                    self.status_message = "File loaded (UTF-8).".to_string();
                }
                Err(e) => {
                    self.status_message = e;
                }
            }
        }

        // --- Central: editor + status bar -----------------------------------------
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(14.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Editor")
                            .strong()
                            .size(18.0)
                            .color(Palette::TEXT),
                    );
                    if let Some(ref path) = self.selected_file {
                        ui.label(
                            egui::RichText::new(path.display().to_string())
                                .small()
                                .monospace()
                                .color(Palette::SUBTEXT),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [0.0, 30.0],
                                egui::Button::new(
                                    egui::RichText::new("💾 Save").color(Palette::ON_RED),
                                )
                                .fill(Palette::RED),
                            )
                            .clicked()
                        {
                            try_save(self);
                        }
                    });
                });
                ui.add_space(4.0);
                ui.separator();

                egui::Frame::none()
                    .fill(Palette::BASE)
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(12.0)
                    .stroke(egui::Stroke::new(1.0, Palette::RED_DIM))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut self.code_buffer)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .lock_focus(true),
                            );
                        });
                    });

                let (lines, chars, bytes) = buffer_stats(&self.code_buffer);
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(Palette::MANTLE)
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .stroke(egui::Stroke::new(1.0, Palette::RED_DIM))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
                            ui.label(
                                egui::RichText::new("Status")
                                    .small()
                                    .strong()
                                    .color(Palette::SUBTEXT),
                            );
                            ui.separator();
                            if !self.status_message.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.status_message)
                                        .small()
                                        .color(Palette::STATUS_OK),
                                );
                                ui.separator();
                            }
                            ui.label(
                                egui::RichText::new(format!(
                                    "Lines: {} · Unicode scalars: {} · UTF-8 bytes: {}",
                                    lines, chars, bytes
                                ))
                                .small()
                                .monospace()
                                .color(Palette::SUBTEXT),
                            );
                            ui.separator();
                            ui.label(
                                egui::RichText::new("Ctrl+S — save")
                                    .small()
                                    .italics()
                                    .color(Palette::RED),
                            );
                        });
                    });
            });
    }
}
