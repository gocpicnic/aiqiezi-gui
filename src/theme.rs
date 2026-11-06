/*
  src/theme.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use eframe::egui::{self, Color32, Stroke, Vec2, Visuals};

pub const CHROME: Color32 = Color32::from_rgb(0x2d, 0x2f, 0x31);
pub const CANVAS: Color32 = Color32::from_rgb(0x17, 0x19, 0x1b);
pub const GRID: Color32 = Color32::from_rgb(0x10, 0x11, 0x12);
pub const FIELD: Color32 = Color32::from_rgb(0x23, 0x25, 0x27);
pub const BORDER: Color32 = Color32::from_rgb(0x3b, 0x3e, 0x41);
pub const TEXT: Color32 = Color32::from_rgb(0xe4, 0xe6, 0xe8);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8e, 0x94, 0x99);
pub const ACCENT: Color32 = Color32::from_rgb(0x01, 0x65, 0xdd);
pub const CAUTION: Color32 = Color32::from_rgb(0xe3, 0xa8, 0x3c);
pub const ERROR: Color32 = Color32::from_rgb(0xe0, 0x4a, 0x4a);
pub const SUCCESS: Color32 = Color32::from_rgb(0x58, 0xb0, 0x5c);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = CHROME;
    visuals.window_fill = CHROME;
    visuals.extreme_bg_color = CANVAS;
    visuals.text_edit_bg_color = Some(FIELD);
    visuals.faint_bg_color = GRID;
    visuals.code_bg_color = FIELD;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = CAUTION;
    visuals.error_fg_color = ERROR;
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(TEXT_DIM);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = CHROME;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    widgets.inactive.bg_fill = FIELD;
    widgets.inactive.weak_bg_fill = FIELD;
    widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.hovered.bg_fill = Color32::from_rgb(0x32, 0x35, 0x38);
    widgets.hovered.weak_bg_fill = Color32::from_rgb(0x32, 0x35, 0x38);
    widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x54, 0x59, 0x5e));
    widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    widgets.active.bg_fill = ACCENT;
    widgets.active.weak_bg_fill = ACCENT;
    widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    widgets.open.bg_fill = FIELD;

    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.set_visuals(visuals);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.interact_size = Vec2::new(28.0, 24.0);
    });
}
