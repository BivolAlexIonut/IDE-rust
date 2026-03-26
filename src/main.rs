mod models;
mod filesystem;
mod app;

use models::MyIDE;

fn main() -> eframe::Result<()> {
    // Configure window properties and initial size
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Professional Rust IDE",
        native_options,
        Box::new(|_cc| {
            let mut app = MyIDE::default();
            // Initial scan of the current directory
            app.refresh_files();
            // Casting the custom struct to a dyn App for eframe to run it
            Box::new(app) as Box<dyn eframe::App>
        }),
    )
}