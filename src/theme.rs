//! Centralized visual design: **red and black** high-contrast theme.
//!
//! Backgrounds use near-black; accents use crimson / deep red. Text stays light for
//! readability (WCAG-style contrast on dark panels).

use eframe::egui;

/// Theme tokens for the red/black IDE skin.
pub struct Palette;

impl Palette {
    /// Deepest background (terminal inner, crust).
    pub const BLACK: egui::Color32 = egui::Color32::from_rgb(8, 8, 10);
    /// Panel backgrounds.
    pub const PANEL: egui::Color32 = egui::Color32::from_rgb(18, 14, 14);
    /// Raised surfaces, editor area.
    pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(28, 22, 22);
    /// Hover / subtle lift.
    pub const OVERLAY: egui::Color32 = egui::Color32::from_rgb(42, 30, 30);

    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(245, 240, 238);
    pub const SUBTEXT: egui::Color32 = egui::Color32::from_rgb(190, 170, 170);

    /// Primary accent (links, selection, primary buttons).
    pub const RED: egui::Color32 = egui::Color32::from_rgb(220, 45, 45);
    pub const RED_DIM: egui::Color32 = egui::Color32::from_rgb(140, 35, 35);
    pub const RED_GLOW: egui::Color32 = egui::Color32::from_rgba_premultiplied(220, 45, 45, 90);

    /// Text on saturated red buttons.
    pub const ON_RED: egui::Color32 = egui::Color32::from_rgb(255, 250, 250);

    /// Secondary highlights (file types) — still in the red family.
    pub const FILE_RS: egui::Color32 = egui::Color32::from_rgb(255, 160, 160);
    pub const FILE_C: egui::Color32 = egui::Color32::from_rgb(255, 200, 140);
    pub const FILE_TOML: egui::Color32 = egui::Color32::from_rgb(255, 220, 180);

    pub const STATUS_OK: egui::Color32 = egui::Color32::from_rgb(255, 200, 160);

    /// Aliases for layout code that names panels by role.
    pub const BASE: egui::Color32 = Self::SURFACE;
    pub const MANTLE: egui::Color32 = Self::PANEL;
    pub const CRUST: egui::Color32 = Self::BLACK;
}

/// Applies global egui style to match the red/black palette.
pub fn apply_ide_theme(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.window_margin = egui::Margin::same(12.0);
        style.spacing.menu_margin = egui::Margin::same(8.0);

        style.interaction.resize_grab_radius_side = 10.0;

        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = None;

        v.window_fill = Palette::SURFACE;
        v.panel_fill = Palette::PANEL;
        v.extreme_bg_color = Palette::BLACK;
        v.faint_bg_color = Palette::OVERLAY;
        v.window_rounding = egui::Rounding::same(8.0);
        v.menu_rounding = egui::Rounding::same(6.0);
        v.hyperlink_color = Palette::RED;
        v.warn_fg_color = egui::Color32::from_rgb(255, 210, 120);
        v.error_fg_color = egui::Color32::from_rgb(255, 100, 100);
        v.selection.bg_fill = Palette::RED_GLOW;
        v.selection.stroke = egui::Stroke::new(1.0, Palette::RED);
        v.widgets.noninteractive.bg_fill = Palette::SURFACE;
        v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Palette::TEXT);
        v.widgets.noninteractive.weak_bg_fill = Palette::PANEL;
        v.widgets.inactive.bg_fill = Palette::OVERLAY;
        v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Palette::SUBTEXT);
        v.widgets.inactive.weak_bg_fill = Palette::PANEL;
        v.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 38, 38);
        v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Palette::TEXT);
        v.widgets.hovered.weak_bg_fill = Palette::OVERLAY;
        v.widgets.active.bg_fill = egui::Color32::from_rgb(70, 45, 45);
        v.widgets.active.fg_stroke = egui::Stroke::new(1.0, Palette::TEXT);
        v.widgets.open.bg_fill = Palette::OVERLAY;
    });
}
