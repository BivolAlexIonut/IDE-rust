//! Entry point: window creation and the native event loop (`eframe`).
//!
//! The binary is intentionally thin: it wires modules (`models`, `filesystem`, `theme`, `app`)
//! and hands control to [`eframe::run_native`]. This matches the usual Rust pattern of keeping
//! `main` free of business logic so tests and documentation can focus on library-like modules.

mod app;
mod filesystem;
mod models;
mod theme;

use models::MyIDE;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("IDE-Rust — mini IDE"),
        ..Default::default()
    };

    eframe::run_native(
        "IDE-Rust",
        native_options,
        Box::new(|_cc| {
            let mut app = MyIDE::default();
            app.refresh_files();
            Box::new(app) as Box<dyn eframe::App>
        }),
    )
}
