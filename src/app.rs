use eframe::egui;
use crate::models::{MyIDE, PendingAction};

impl eframe::App for MyIDE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global UI Styling
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 25, 25);
        ctx.set_visuals(visuals);

        let mut next_dir = None;
        let mut file_to_open = None;

        // --- BREADCRUMBS PANEL ---
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📍").color(egui::Color32::LIGHT_BLUE));
                ui.label(self.current_dir.display().to_string());
            });
        });

        // --- TERMINAL PANEL (Bottom) ---
        egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(true)
            .default_height(280.0) // Increased height for better visibility
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.heading("💻 Terminal (Bash)");
                    if ui.button("🗑 Clear").clicked() { self.terminal_output.clear(); }
                });
                ui.add_space(5.0);

                // Black container for the terminal, similar to VS Code console
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(10, 10, 10))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        // 1. Output History Area
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 45.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::multiline(&mut self.terminal_output)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .text_color(egui::Color32::from_rgb(210, 210, 210))
                                    .interactive(false));
                            });

                        ui.separator();

                        // 2. Interactive Input Line
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("➜").color(egui::Color32::GREEN).strong());

                            // High-visibility input field
                            let edit = egui::TextEdit::singleline(&mut self.terminal_input)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .frame(false) // Borderless for integrated look
                                .hint_text("Type a bash command and press Enter...");

                            let res = ui.add(edit);
                            if res.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                self.run_terminal_command();
                                res.request_focus(); // Refocus immediately for next command
                            }
                        });
                    });
            });

        // --- SIDE PANEL (File Explorer) ---
        egui::SidePanel::left("explorer").resizable(true).show(ctx, |ui| {
            ui.add_space(10.0);
            ui.heading("📁 Project");
            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("🔍 Search..."));

            ui.horizontal(|ui| {
                if ui.button("➕ New File").clicked() { self.show_new_file_dialog = true; }
                if ui.button("⬆ Back").clicked() {
                    next_dir = self.current_dir.parent().map(|p| p.to_path_buf());
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for path in &self.files {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if !self.search_query.is_empty() && !name.to_lowercase().contains(&self.search_query.to_lowercase()) { continue; }

                    let is_selected = self.selected_file.as_ref() == Some(path);
                    let is_dir = path.is_dir();
                    let icon = if is_dir { "📁" } else { "📄" };

                    let mut text = egui::RichText::new(format!("{} {}", icon, name));
                    if name.ends_with(".rs") { text = text.color(egui::Color32::from_rgb(222, 165, 132)); }
                    else if name.ends_with(".cpp") { text = text.color(egui::Color32::from_rgb(100, 150, 255)); }

                    if ui.selectable_label(is_selected, text).clicked() {
                        if is_dir { next_dir = Some(path.clone()); }
                        else { file_to_open = Some(path.clone()); }
                    }
                }
            });
        });

        // --- DIALOG MODALS ---
        if self.show_new_file_dialog {
            egui::Window::new("Create New File").anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_file_name).hint_text("filename.ext"));
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() { let _ = self.create_file(); }
                    if ui.button("Cancel").clicked() { self.show_new_file_dialog = false; }
                });
            });
        }

        // --- LOGIC EXECUTION ---
        if let Some(p) = next_dir { self.current_dir = p; self.refresh_files(); self.search_query.clear(); }
        if let Some(p) = file_to_open {
            if let Ok(c) = std::fs::read_to_string(&p) {
                self.code_buffer = c;
                self.selected_file = Some(p);
            }
        }

        // --- CENTRAL EDITOR PANEL ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Editor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💾 Save").clicked() { let _ = self.save_current_file(); }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.code_buffer)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .lock_focus(true));
            });
        });
    }
}