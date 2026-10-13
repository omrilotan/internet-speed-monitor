//! Main window layout: menu bar, header, controls, stat cards, chart and table.

use std::sync::Arc;

use chrono::Local;
use egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Layout, RichText, Stroke, TextStyle, Vec2,
};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, PlotPoints, Points};

use super::format;
use super::{App, ChartMode, ScheduleType, StatusKind, TABLE_ROWS};
use crate::schedule::{self, CronDescription};
use crate::{CURRENT_VERSION, DOCUMENTATION_URL, REPOSITORY_URL};

pub const PAGE_BG: Color32 = Color32::from_rgb(0xf5, 0xf5, 0xf5);
pub const CARD_BG: Color32 = Color32::WHITE;
pub const TEXT: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const HEADING: Color32 = Color32::from_rgb(0x2c, 0x3e, 0x50);
pub const MUTED: Color32 = Color32::from_rgb(0x66, 0x66, 0x66);
pub const SUBTLE: Color32 = Color32::from_rgb(0x7f, 0x8c, 0x8d);
pub const PRIMARY: Color32 = Color32::from_rgb(0x34, 0x98, 0xdb);
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0x29, 0x80, 0xb9);
pub const SECONDARY: Color32 = Color32::from_rgb(0x95, 0xa5, 0xa6);
pub const DANGER: Color32 = Color32::from_rgb(0xdc, 0x35, 0x45);
pub const SUCCESS: Color32 = Color32::from_rgb(0x28, 0xa7, 0x45);
pub const LINK: Color32 = Color32::from_rgb(0x00, 0x7b, 0xff);
pub const RUNNING: Color32 = Color32::from_rgb(0x27, 0xae, 0x60);
pub const SLEEPING: Color32 = Color32::from_rgb(0xf3, 0x9c, 0x12);
pub const STOPPED: Color32 = Color32::from_rgb(0xe7, 0x4c, 0x3c);
pub const DOWNLOAD: Color32 = Color32::from_rgb(0x34, 0x98, 0xdb);
pub const UPLOAD: Color32 = Color32::from_rgb(0xe7, 0x4c, 0x3c);
pub const BORDER: Color32 = Color32::from_rgb(0xdd, 0xdd, 0xdd);
pub const TABLE_HEADER: Color32 = Color32::from_rgb(0xf8, 0xf9, 0xfa);
pub const BANNER: Color32 = Color32::from_rgb(0x4a, 0x90, 0xe2);
pub const BANNER_EDGE: Color32 = Color32::from_rgb(0x35, 0x7a, 0xbd);

pub const FONT_BOLD: &str = "bold";
pub const FONT_SEMIBOLD: &str = "semibold";

pub fn bold_family() -> FontFamily {
    FontFamily::Name(FONT_BOLD.into())
}

pub fn semibold_family() -> FontFamily {
    FontFamily::Name(FONT_SEMIBOLD.into())
}

pub fn bold(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(FontId::new(size, bold_family()))
}

pub fn semibold(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(FontId::new(size, semibold_family()))
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter-Regular".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "Inter-SemiBold".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "Inter-Bold".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../fonts/Inter-Bold.ttf"
        ))),
    );

    let fallbacks = [
        "Ubuntu-Light".to_owned(),
        "NotoEmoji-Regular".to_owned(),
        "emoji-icon-font".to_owned(),
    ];
    let with_fallbacks = |primary: &str| -> Vec<String> {
        std::iter::once(primary.to_owned())
            .chain(fallbacks.iter().cloned())
            .collect()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, with_fallbacks("Inter-Regular"));
    fonts
        .families
        .insert(bold_family(), with_fallbacks("Inter-Bold"));
    fonts
        .families
        .insert(semibold_family(), with_fallbacks("Inter-SemiBold"));
    ctx.set_fonts(fonts);
}

pub fn install_style(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_theme(egui::Theme::Light);

    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = PAGE_BG;
    visuals.window_fill = CARD_BG;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = TABLE_HEADER;
    visuals.override_text_color = Some(TEXT);
    visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xf1, 0xf3, 0xf5);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xf1, 0xf3, 0xf5);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0xad, 0xb5, 0xbd));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.active.bg_fill = Color32::from_rgb(0xe9, 0xec, 0xef);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0xe9, 0xec, 0xef);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.open.bg_fill = Color32::WHITE;
    visuals.widgets.open.weak_bg_fill = Color32::WHITE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.selection.bg_fill = Color32::from_rgb(0xcf, 0xe4, 0xf7);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY_DARK);
    visuals.hyperlink_color = LINK;
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
        &mut visuals.widgets.noninteractive,
    ] {
        widget.corner_radius = CornerRadius::same(4);
    }
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(40),
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.menu_corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.menu_margin = egui::Margin::same(6);
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(24.0, bold_family()));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(13.0));
    });
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD_BG)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::same(20))
        .shadow(egui::epaint::Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha(22),
        })
}

/// Solid button in the style of the old `.btn` classes.
pub fn solid_button(label: impl Into<String>, fill: Color32) -> egui::Button<'static> {
    egui::Button::new(semibold(label, 14.0).color(Color32::WHITE))
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(4))
        .min_size(Vec2::new(0.0, 34.0))
}

pub fn draw(app: &mut App, root: &mut egui::Ui) {
    egui::Panel::top("menu-bar").show(root, |ui| {
        egui::MenuBar::new().ui(ui, |ui| draw_menu_bar(app, ui));
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(PAGE_BG))
        .show(root, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        let width = (ui.available_width() - 40.0).clamp(320.0, 1200.0);
                        ui.set_max_width(width);
                        ui.set_width(width);
                        ui.spacing_mut().item_spacing.y = 20.0;
                        draw_header(app, ui);
                        draw_update_banner(app, ui);
                        draw_controls(app, ui);
                        draw_stat_cards(app, ui);
                        draw_chart_section(app, ui);
                        ui.add_space(20.0);
                    });
                });
        });
}

fn draw_menu_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Quit").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
    ui.menu_button("View", |ui| {
        if ui.button("Zoom In").clicked() {
            let zoom = ui.ctx().zoom_factor();
            ui.ctx().set_zoom_factor((zoom * 1.1).min(3.0));
            ui.close();
        }
        if ui.button("Zoom Out").clicked() {
            let zoom = ui.ctx().zoom_factor();
            ui.ctx().set_zoom_factor((zoom / 1.1).max(0.5));
            ui.close();
        }
        if ui.button("Actual Size").clicked() {
            ui.ctx().set_zoom_factor(1.0);
            ui.close();
        }
        ui.separator();
        if ui.button("Toggle Fullscreen").clicked() {
            let fullscreen = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
            ui.close();
        }
    });
    ui.menu_button("Help", |ui| {
        if ui.button("About Internet Speed Monitor").clicked() {
            app.about_open = true;
            ui.close();
        }
        ui.separator();
        if ui.button("Show Debug Log").clicked() {
            app.show_debug_log();
            ui.close();
        }
        if ui.button("Clear Debug Log").clicked() {
            app.clear_debug_log();
            ui.close();
        }
        if ui.button("Open App Storage Folder").clicked() {
            app.open_storage_folder();
            ui.close();
        }
        ui.separator();
        if ui.button("GitHub Repository").clicked() {
            app.open_external(REPOSITORY_URL);
            ui.close();
        }
        if ui.button("Documentation").clicked() {
            app.open_external(DOCUMENTATION_URL);
            ui.close();
        }
    });
}

fn draw_header(app: &mut App, ui: &mut egui::Ui) {
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        // Below this width the status moves to its own row (like the old CSS breakpoints).
        let narrow = ui.available_width() < 640.0;
        ui.horizontal(|ui| {
            ui.label(bold("Internet Speed Monitor", 24.0).color(HEADING));
            ui.add_space(4.0);
            ui.label(semibold(format!("Version {CURRENT_VERSION}"), 12.0).color(SUBTLE));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                draw_hamburger(app, ui);
                if !narrow {
                    ui.add_space(12.0);
                    draw_status(ui, app);
                }
            });
        });
        if narrow {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&app.status_text).size(15.0).color(TEXT));
                draw_status_dot(ui, app.status_kind);
            });
        }
    });
}

/// Status text followed by the coloured dot, laid out right to left.
fn draw_status(ui: &mut egui::Ui, app: &App) {
    draw_status_dot(ui, app.status_kind);
    ui.label(RichText::new(&app.status_text).size(15.0).color(TEXT));
}

fn draw_status_dot(ui: &mut egui::Ui, kind: StatusKind) {
    let (color, glow) = match kind {
        StatusKind::Running => (
            RUNNING,
            Some(Color32::from_rgba_unmultiplied(39, 174, 96, 90)),
        ),
        StatusKind::Sleeping => (
            SLEEPING,
            Some(Color32::from_rgba_unmultiplied(243, 156, 18, 80)),
        ),
        StatusKind::Stopped => (STOPPED, None),
    };
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
    let painter = ui.painter();
    if let Some(glow) = glow {
        painter.circle_filled(rect.center(), 8.0, glow);
    }
    painter.circle_filled(rect.center(), 6.0, color);
}

fn draw_hamburger(app: &mut App, ui: &mut egui::Ui) {
    let button = egui::Button::new(RichText::new("☰").size(16.0).color(Color32::WHITE))
        .fill(SECONDARY)
        .stroke(Stroke::NONE)
        .min_size(Vec2::new(40.0, 34.0));
    let config = egui::containers::menu::MenuConfig::new()
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
    let (response, _) = egui::containers::menu::MenuButton::from_button(button)
        .config(config)
        .ui(ui, |ui| {
            ui.set_min_width(240.0);
            ui.spacing_mut().item_spacing.y = 2.0;
            if menu_item(ui, "💾  Export as CSV")
                .on_hover_text("Download speed test data as CSV file")
                .clicked()
            {
                app.open_export_dialog();
                ui.close();
            }
            ui.separator();
            if toggle_item(ui, "🚀  Auto-start monitoring", &mut app.auto_start)
                .on_hover_text("Automatically start monitoring when app opens")
                .changed()
            {
                app.toggle_auto_start();
            }
            if toggle_item(ui, "🔄  Start with computer", &mut app.launch_at_startup)
                .on_hover_text("Launch app when computer starts")
                .changed()
            {
                app.toggle_launch_at_startup();
            }
            ui.separator();
            if menu_item(ui, "⚠  Clear All History")
                .on_hover_text("Delete all speed test history permanently")
                .clicked()
            {
                app.confirm_clear_history = true;
                ui.close();
            }
            if menu_item(ui, "🗑  Clear Data Until...")
                .on_hover_text("Remove data older than a specific date")
                .clicked()
            {
                app.open_clear_dialog();
                ui.close();
            }
        });
    response.on_hover_text("More options");
}

fn menu_item(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let desired = Vec2::new(ui.available_width(), 36.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(rect, 4.0, TABLE_HEADER);
        }
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            HEADING,
        );
    }
    response
}

/// Menu row with a label and a toggle switch on the right.
fn toggle_item(ui: &mut egui::Ui, label: &str, value: &mut bool) -> egui::Response {
    let desired = Vec2::new(ui.available_width(), 36.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, egui::Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *value, label)
    });
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *value);
        let painter = ui.painter();
        if response.hovered() {
            painter.rect_filled(rect, 4.0, TABLE_HEADER);
        }
        painter.text(
            egui::pos2(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(14.0),
            HEADING,
        );
        let track = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 32.0, rect.center().y),
            Vec2::new(44.0, 24.0),
        );
        let off = Color32::from_rgb(0xcc, 0xcc, 0xcc);
        let track_color = Color32::from_rgb(
            egui::lerp(f32::from(off.r())..=f32::from(SUCCESS.r()), how_on) as u8,
            egui::lerp(f32::from(off.g())..=f32::from(SUCCESS.g()), how_on) as u8,
            egui::lerp(f32::from(off.b())..=f32::from(SUCCESS.b()), how_on) as u8,
        );
        painter.rect_filled(track, 12.0, track_color);
        let knob_x = egui::lerp((track.left() + 12.0)..=(track.right() - 12.0), how_on);
        painter.circle_filled(egui::pos2(knob_x, track.center().y), 10.0, Color32::WHITE);
    }
    response
}

fn draw_update_banner(app: &mut App, ui: &mut egui::Ui) {
    let Some(info) = app.update_info.clone() else {
        return;
    };
    if app.update_dismissed {
        return;
    }
    egui::Frame::new()
        .fill(BANNER)
        .stroke(Stroke::new(1.0, BANNER_EDGE))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(20, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    semibold(format!("🎉 v{} available!", info.latest_version), 14.0)
                        .color(Color32::WHITE),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if mini_button(ui, "✕").clicked() {
                        app.update_dismissed = true;
                    }
                    if mini_button(ui, "Download").clicked() {
                        app.open_external(crate::RELEASES_URL);
                    }
                });
            });
        });
}

fn mini_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(semibold(label, 11.0).color(Color32::WHITE))
            .fill(Color32::from_white_alpha(30))
            .stroke(Stroke::new(1.0, Color32::from_white_alpha(80))),
    )
}

/// Large square action button (start/stop/test now).
fn action_button(ui: &mut egui::Ui, glyph: &str, label: &str, enabled: bool) -> egui::Response {
    let size = Vec2::splat(44.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    if ui.is_rect_visible(rect) {
        let fill = if !enabled {
            PRIMARY.gamma_multiply(0.45)
        } else if response.hovered() {
            PRIMARY_DARK
        } else {
            PRIMARY
        };
        let painter = ui.painter();
        painter.rect_filled(rect, 6.0, fill);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(20.0),
            Color32::WHITE,
        );
    }
    response.on_hover_text(label)
}

fn draw_controls(app: &mut App, ui: &mut egui::Ui) {
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            // Schedule type
            let previous = app.schedule_type;
            egui::ComboBox::from_id_salt("schedule-type")
                .selected_text(match app.schedule_type {
                    ScheduleType::Interval => "Interval",
                    ScheduleType::Cron => "Cron",
                })
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut app.schedule_type, ScheduleType::Interval, "Interval");
                    ui.selectable_value(&mut app.schedule_type, ScheduleType::Cron, "Cron");
                });
            if previous != app.schedule_type {
                if app.schedule_type == ScheduleType::Cron && app.cron_expression.trim().is_empty()
                {
                    app.cron_expression = crate::settings::DEFAULT_CRON_EXPRESSION.to_string();
                }
                app.save_schedule_settings();
            }

            ui.add_space(8.0);
            match app.schedule_type {
                ScheduleType::Interval => draw_interval_controls(app, ui),
                ScheduleType::Cron => draw_cron_controls(app, ui),
            }

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                let test_enabled = !app.test_now_busy && !app.test_running;
                let test_label = if test_enabled {
                    "Run a single speed test now"
                } else {
                    "Test in progress..."
                };
                if action_button(ui, "●", test_label, test_enabled).clicked() && test_enabled {
                    app.test_once_now();
                }

                let (glyph, tip) = if app.is_monitoring {
                    ("■", "Stop automatic speed testing")
                } else {
                    ("▶", "Start automatic speed testing")
                };
                if action_button(ui, glyph, tip, true).clicked() {
                    app.toggle_monitoring();
                }
            });
        });
    });
}

fn draw_interval_controls(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.add_enabled_ui(!app.is_monitoring, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui
                    .add(step_button("−"))
                    .on_hover_text("Previous step (1, 5, 10, 15, 30, 60)")
                    .clicked()
                {
                    app.step_interval(false);
                }
                let id = ui.make_persistent_id("interval-input");
                let focused = ui.memory(|m| m.has_focus(id));
                if focused {
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp))
                    {
                        app.step_interval(true);
                    }
                    if ui
                        .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown))
                    {
                        app.step_interval(false);
                    }
                }
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.interval_text)
                        .id(id)
                        .desired_width(48.0)
                        .margin(egui::Margin::symmetric(8, 7))
                        .horizontal_align(Align::Center),
                );
                if response.lost_focus() {
                    app.commit_interval_text();
                } else if response.changed()
                    && let Ok(value) = app.interval_text.trim().parse::<u32>()
                    && (1..=60).contains(&value)
                {
                    app.set_interval(value);
                }
                if response.hovered() {
                    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll > 0.0 {
                        app.step_interval(true);
                    } else if scroll < 0.0 {
                        app.step_interval(false);
                    }
                }
                response.on_hover_text(
                    "Minutes between tests (1-60). Arrow keys and scroll step through 1, 5, 10, 15, 30, 60.",
                );
                if ui
                    .add(step_button("+"))
                    .on_hover_text("Next step (1, 5, 10, 15, 30, 60)")
                    .clicked()
                {
                    app.step_interval(true);
                }
            });
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.label(RichText::new("Every").size(11.0).italics().color(MUTED));
            ui.label(semibold(app.interval_minutes.to_string(), 11.0).color(LINK));
            ui.label(RichText::new("minutes").size(11.0).italics().color(MUTED));
        });
    });
}

fn step_button(label: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(label).size(14.0))
        .min_size(Vec2::new(30.0, 30.0))
        .fill(TABLE_HEADER)
}

fn draw_cron_controls(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.set_max_width(400.0);
        ui.spacing_mut().item_spacing.y = 6.0;
        let response = ui.add(
            egui::TextEdit::singleline(&mut app.cron_expression)
                .font(TextStyle::Monospace)
                .hint_text("*/5 * * * *")
                .margin(egui::Margin::symmetric(12, 8))
                .desired_width(f32::INFINITY),
        );
        let cron_input_id = response.id;
        let response = response.on_hover_text("Enter cron expression");
        if response.lost_focus() {
            app.save_schedule_settings();
        }

        match schedule::classify_cron(&app.cron_expression) {
            CronDescription::Empty => cron_hint(ui, "Enter a cron expression", MUTED),
            CronDescription::Known(description) => cron_hint(ui, description, SUCCESS),
            CronDescription::Custom => cron_hint(ui, "Custom schedule", SUCCESS),
            CronDescription::Invalid => cron_hint(ui, "Invalid cron expression", DANGER),
        }

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            for (label, expression, tip) in schedule::CRON_PRESETS {
                if ui
                    .add(preset_button(label, false))
                    .on_hover_text(tip)
                    .clicked()
                {
                    app.cron_expression = expression.to_string();
                    app.save_schedule_settings();
                }
            }
            if ui
                .add(preset_button("Custom", true))
                .on_hover_text("Enter custom cron expression")
                .clicked()
            {
                app.cron_expression.clear();
                ui.memory_mut(|m| m.request_focus(cron_input_id));
            }
        });
    });
}

fn cron_hint(ui: &mut egui::Ui, text: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(semibold(text, 11.0).italics().color(color));
        if text == "Custom schedule" || text == "Enter a cron expression" {
            ui.label(RichText::new("-").size(11.0).color(MUTED));
            ui.hyperlink_to(
                RichText::new("Create a custom schedule").size(11.0),
                "https://crontab.guru/",
            );
        }
    });
}

fn preset_button(label: &str, custom: bool) -> egui::Button<'_> {
    let (fill, text, border) = if custom {
        (
            Color32::from_rgb(0xff, 0xf3, 0xcd),
            Color32::from_rgb(0x85, 0x64, 0x04),
            Color32::from_rgb(0xff, 0xc1, 0x07),
        )
    } else {
        (
            Color32::from_rgb(0xf1, 0xf3, 0xf5),
            Color32::from_rgb(0x49, 0x50, 0x57),
            BORDER,
        )
    };
    egui::Button::new(RichText::new(label).size(12.0).color(text))
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .min_size(Vec2::new(0.0, 28.0))
}

type StatCard<'a> = Box<dyn FnOnce(&mut egui::Ui) + 'a>;

fn draw_stat_cards(app: &mut App, ui: &mut egui::Ui) {
    let latest = app.latest.clone();
    let has_data = !app.recent.is_empty();
    let median_download = App::median(app.recent.iter().map(|t| t.download));
    let median_upload = App::median(app.recent.iter().map(|t| t.upload));
    let median_ping = App::median(app.recent.iter().map(|t| t.ping));

    let columns = if ui.available_width() < 700.0 { 2 } else { 4 };
    let next_test = app.next_test_time();
    let test_running = app.test_running;

    let mut cards: Vec<StatCard<'_>> = Vec::new();
    cards.push(Box::new(move |ui: &mut egui::Ui| {
        let value = latest
            .as_ref()
            .map(|t| format::time_hms(t.local_time()))
            .unwrap_or_else(|| "Never".to_string());
        stat_card(ui, "Last Test", &value, |ui| {
            if test_running {
                let t = ui.input(|i| i.time);
                let pulse = 0.6 + 0.4 * (((t * std::f64::consts::TAU / 1.5).sin() * 0.5) + 0.5);
                let color = LINK.gamma_multiply(pulse as f32);
                ui.label(semibold("🔄 Running speed test...", 12.0).color(color));
            } else if let Some(next) = next_test {
                label_value_line(
                    ui,
                    "Next test:",
                    &format::time_hm(next.with_timezone(&Local)),
                    Color32::from_rgb(0x49, 0x50, 0x57),
                );
            }
        });
    }));
    let latest_dl = app.latest.as_ref().map(|t| t.download);
    cards.push(Box::new(move |ui: &mut egui::Ui| {
        let value = latest_dl
            .map(format::mbps)
            .unwrap_or_else(|| "-- Mbps".to_string());
        stat_card(ui, "Download Speed", &value, |ui| {
            if has_data && let Some(median) = median_download {
                median_line(ui, &format::mbps(median));
            }
        });
    }));
    let latest_ul = app.latest.as_ref().map(|t| t.upload);
    cards.push(Box::new(move |ui: &mut egui::Ui| {
        let value = latest_ul
            .map(format::mbps)
            .unwrap_or_else(|| "-- Mbps".to_string());
        stat_card(ui, "Upload Speed", &value, |ui| {
            if has_data && let Some(median) = median_upload {
                median_line(ui, &format::mbps(median));
            }
        });
    }));
    let latest_ping = app.latest.as_ref().map(|t| t.ping);
    cards.push(Box::new(move |ui: &mut egui::Ui| {
        let value = latest_ping
            .map(format::ms)
            .unwrap_or_else(|| "-- ms".to_string());
        stat_card(ui, "Ping", &value, |ui| {
            if has_data && let Some(median) = median_ping {
                median_line(ui, &format::ms(median));
            }
        });
    }));

    let mut cards = cards.into_iter();
    while cards.len() > 0 {
        ui.columns(columns, |cols| {
            for col in cols.iter_mut() {
                if let Some(card_fn) = cards.next() {
                    card_fn(col);
                }
            }
        });
    }
}

fn stat_card(ui: &mut egui::Ui, title: &str, value: &str, extra: impl FnOnce(&mut egui::Ui)) {
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.label(
                bold(title.to_uppercase(), 13.0)
                    .color(MUTED)
                    .extra_letter_spacing(1.0),
            );
            ui.label(bold(value, 24.0).color(HEADING));
            // Reserve the info row even when empty so all cards share one height.
            let row = Vec2::new(ui.available_width(), 18.0);
            ui.allocate_ui_with_layout(row, Layout::top_down(Align::Center), |ui| {
                ui.set_min_size(row);
                extra(ui);
            });
        });
    });
}

fn median_line(ui: &mut egui::Ui, value: &str) {
    ui.label(
        RichText::new(format!("Median: {value}"))
            .size(12.0)
            .color(SUBTLE),
    );
}

/// One centred line made of a muted label and an emphasised value.
fn label_value_line(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32) {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        label,
        0.0,
        egui::TextFormat {
            font_id: FontId::proportional(12.0),
            color: SUBTLE,
            ..Default::default()
        },
    );
    job.append(
        value,
        4.0,
        egui::TextFormat {
            font_id: FontId::new(12.0, semibold_family()),
            color: value_color,
            ..Default::default()
        },
    );
    ui.label(job);
}

fn draw_chart_section(app: &mut App, ui: &mut egui::Ui) {
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(bold("Speed History & Recent Tests", 22.0).color(HEADING));
            ui.label(RichText::new(&app.chart_subtitle).size(15.0).color(MUTED));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(semibold("View:", 14.0));
            let mut mode = app.chart_mode;
            ui.radio_value(&mut mode, ChartMode::Last20, "Last 20 data points");
            ui.radio_value(&mut mode, ChartMode::LastDay, "Last Day");
            ui.radio_value(&mut mode, ChartMode::Last7, "Last 7 days");
            ui.radio_value(&mut mode, ChartMode::Range, "Date range");
            app.set_chart_mode(mode);
        });

        if app.chart_mode == ChartMode::Range {
            ui.horizontal_wrapped(|ui| {
                ui.label("Start:");
                let min = app.chart_min;
                let max = app.chart_max;
                ui.add(
                    egui_extras::DatePickerButton::new(&mut app.chart_start)
                        .id_salt("chart-start")
                        .show_icon(false)
                        .calendar_week(false)
                        .start_end_years(min.year()..=max.year()),
                );
                ui.label("End:");
                ui.add(
                    egui_extras::DatePickerButton::new(&mut app.chart_end)
                        .id_salt("chart-end")
                        .show_icon(false)
                        .calendar_week(false)
                        .start_end_years(min.year()..=max.year()),
                );
                app.chart_start = app.chart_start.clamp(min, max);
                app.chart_end = app.chart_end.clamp(min, max);
                if app.chart_start > app.chart_end {
                    app.chart_end = app.chart_start;
                }
                if ui.add(solid_button("Apply", SECONDARY)).clicked() {
                    app.refresh_chart();
                }
            });
        }

        draw_legend(ui);
        draw_plot(app, ui);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(bold("Recent Tests", 17.0).color(HEADING));
            ui.label(
                RichText::new("(last 20 data points)")
                    .size(13.0)
                    .color(MUTED),
            );
        });
        draw_table(app, ui);
    });
}

fn draw_legend(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (label, color) in [
                ("Download Speed (Mbps)", DOWNLOAD),
                ("Upload Speed (Mbps)", UPLOAD),
            ] {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(40.0, 12.0), egui::Sense::hover());
                ui.painter().rect(
                    rect,
                    0.0,
                    color.gamma_multiply(0.15),
                    Stroke::new(2.0, color),
                    egui::StrokeKind::Inside,
                );
                ui.label(RichText::new(label).size(12.0).color(MUTED));
                ui.add_space(12.0);
            }
        });
    });
}

/// Cardinal-spline smoothing equivalent to Chart.js `tension: 0.4`.
pub fn smooth_curve(points: &[[f64; 2]], tension: f64, segments: usize) -> Vec<[f64; 2]> {
    if points.len() < 3 || segments == 0 {
        return points.to_vec();
    }
    let n = points.len();
    // Control points per data point (before, after).
    let mut controls = Vec::with_capacity(n);
    for i in 0..n {
        let prev = points[i.saturating_sub(1)];
        let current = points[i];
        let next = points[(i + 1).min(n - 1)];
        let d01 = ((current[0] - prev[0]).powi(2) + (current[1] - prev[1]).powi(2)).sqrt();
        let d12 = ((next[0] - current[0]).powi(2) + (next[1] - current[1]).powi(2)).sqrt();
        let total = d01 + d12;
        let (s01, s12) = if total > 0.0 {
            (d01 / total, d12 / total)
        } else {
            (0.5, 0.5)
        };
        let fa = tension * s01;
        let fb = tension * s12;
        let before = [
            current[0] - fa * (next[0] - prev[0]),
            current[1] - fa * (next[1] - prev[1]),
        ];
        let after = [
            current[0] + fb * (next[0] - prev[0]),
            current[1] + fb * (next[1] - prev[1]),
        ];
        controls.push((before, after));
    }
    let mut out = Vec::with_capacity((n - 1) * segments + 1);
    for i in 0..n - 1 {
        let p0 = points[i];
        let p1 = controls[i].1;
        let p2 = controls[i + 1].0;
        let p3 = points[i + 1];
        for step in 0..segments {
            let t = step as f64 / segments as f64;
            let mt = 1.0 - t;
            let x = mt * mt * mt * p0[0]
                + 3.0 * mt * mt * t * p1[0]
                + 3.0 * mt * t * t * p2[0]
                + t * t * t * p3[0];
            let y = mt * mt * mt * p0[1]
                + 3.0 * mt * mt * t * p1[1]
                + 3.0 * mt * t * t * p2[1]
                + t * t * t * p3[1];
            out.push([x, y]);
        }
    }
    out.push(points[n - 1]);
    out
}

fn draw_plot(app: &App, ui: &mut egui::Ui) {
    let points: Vec<(f64, f64, f64, String)> = app
        .chart_data
        .iter()
        .map(|test| {
            let local = test.local_time();
            (
                local.timestamp() as f64,
                test.download,
                test.upload,
                format::chart_label(local),
            )
        })
        .collect();
    let labels: Arc<Vec<String>> = Arc::new(points.iter().map(|p| p.3.clone()).collect());
    let download: Vec<[f64; 2]> = points.iter().map(|p| [p.0, p.1]).collect();
    let upload: Vec<[f64; 2]> = points.iter().map(|p| [p.0, p.2]).collect();
    let span = match (points.first(), points.last()) {
        (Some(first), Some(last)) => (last.0 - first.0).abs(),
        _ => 0.0,
    };
    let hover_labels = labels.clone();

    // Smooth in a normalised space so the curve shape is independent of axis scale.
    let (x_min, x_max) = (
        points.first().map(|p| p.0).unwrap_or(0.0),
        points.last().map(|p| p.0).unwrap_or(1.0),
    );
    let y_max = points.iter().map(|p| p.1.max(p.2)).fold(1.0_f64, f64::max);
    let x_scale = (x_max - x_min).max(1.0);
    let normalise = |series: &[[f64; 2]]| -> Vec<[f64; 2]> {
        series
            .iter()
            .map(|p| [(p[0] - x_min) / x_scale, p[1] / y_max])
            .collect()
    };
    let denormalise = |series: Vec<[f64; 2]>| -> Vec<[f64; 2]> {
        series
            .into_iter()
            .map(|p| [p[0] * x_scale + x_min, p[1] * y_max])
            .collect()
    };
    // Like Chart.js `capBezierPoints`, never let the smoothed curve dip below zero.
    let clamp_floor = |series: Vec<[f64; 2]>| -> Vec<[f64; 2]> {
        series.into_iter().map(|p| [p[0], p[1].max(0.0)]).collect()
    };
    let download_curve = clamp_floor(denormalise(smooth_curve(&normalise(&download), 0.4, 12)));
    let upload_curve = clamp_floor(denormalise(smooth_curve(&normalise(&upload), 0.4, 12)));
    let marker_radius = if points.len() > 60 { 2.0 } else { 3.0 };

    let mut plot = Plot::new("speed-chart")
        .height(300.0)
        .x_axis_label(RichText::new("Time").size(12.0).color(MUTED))
        .y_axis_label(RichText::new("Speed (Mbps)").size(12.0).color(MUTED))
        .include_y(0.0)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_grid(true)
        .grid_color(Color32::from_rgb(0xe6, 0xe6, 0xe6))
        .show_background(false)
        .x_grid_spacer(time_grid_spacer)
        .y_grid_spacer(value_grid_spacer)
        .y_axis_formatter(|mark, _range| format_axis_value(mark.value))
        .x_axis_formatter(move |mark, _range| format::chart_axis_label(mark.value, span))
        .label_formatter(move |pos| match pos {
            egui_plot::HoverPosition::NearDataPoint {
                plot_name,
                position,
                index,
            } if !plot_name.is_empty() => {
                let when = hover_labels.get(*index).cloned().unwrap_or_default();
                Some(format!("{plot_name}\n{:.2} Mbps\n{when}", position.y))
            }
            _ => None,
        });
    if points.len() == 1 {
        let x = points[0].0;
        plot = plot.include_x(x - 60.0).include_x(x + 60.0);
    }
    if points.is_empty() {
        plot = plot.include_x(0.0).include_x(1.0).include_y(1.0);
    }

    plot.show(ui, |plot_ui| {
        plot_ui.line(
            Line::new("", PlotPoints::from(download_curve))
                .color(DOWNLOAD)
                .width(2.5)
                .fill(0.0)
                .fill_alpha(0.08)
                .allow_hover(false),
        );
        plot_ui.line(
            Line::new("", PlotPoints::from(upload_curve))
                .color(UPLOAD)
                .width(2.5)
                .fill(0.0)
                .fill_alpha(0.08)
                .allow_hover(false),
        );
        plot_ui.points(
            Points::new("Download Speed (Mbps)", PlotPoints::from(download))
                .color(DOWNLOAD)
                .radius(marker_radius)
                .filled(true),
        );
        plot_ui.points(
            Points::new("Upload Speed (Mbps)", PlotPoints::from(upload))
                .color(UPLOAD)
                .radius(marker_radius)
                .filled(true),
        );
    });
}

/// Grid lines at round speed values so the axis reads 0, 20, 40 ... instead of one line.
fn value_grid_spacer(input: egui_plot::GridInput) -> Vec<egui_plot::GridMark> {
    const STEPS: [f64; 16] = [
        0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0,
        2000.0,
    ];
    let (min, max) = input.bounds;
    let span = (max - min).max(0.1);
    let step = STEPS
        .iter()
        .copied()
        .find(|step| span / step <= 8.0 && *step >= input.base_step_size)
        .unwrap_or(STEPS[STEPS.len() - 1]);
    let mut value = (min / step).floor() * step;
    let mut marks = Vec::new();
    while value <= max {
        if value >= min {
            marks.push(egui_plot::GridMark {
                value,
                step_size: step,
            });
        }
        value += step;
    }
    marks
}

fn format_axis_value(value: f64) -> String {
    if value.fract().abs() < 1e-6 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// Grid lines on sensible clock boundaries (minutes, hours, days) in local time.
fn time_grid_spacer(input: egui_plot::GridInput) -> Vec<egui_plot::GridMark> {
    const HOUR: f64 = 3600.0;
    const DAY: f64 = 24.0 * HOUR;
    const STEPS: [f64; 15] = [
        60.0,
        5.0 * 60.0,
        15.0 * 60.0,
        30.0 * 60.0,
        HOUR,
        2.0 * HOUR,
        3.0 * HOUR,
        6.0 * HOUR,
        12.0 * HOUR,
        DAY,
        2.0 * DAY,
        7.0 * DAY,
        14.0 * DAY,
        30.0 * DAY,
        90.0 * DAY,
    ];
    let (min, max) = input.bounds;
    let span = (max - min).max(1.0);
    let step = STEPS
        .iter()
        .copied()
        .find(|step| span / step <= 12.0 && *step >= input.base_step_size)
        .unwrap_or(STEPS[STEPS.len() - 1]);
    let offset = f64::from(Local::now().offset().local_minus_utc());
    let mut value = ((min + offset) / step).floor() * step - offset;
    let mut marks = Vec::new();
    while value <= max {
        if value >= min {
            marks.push(egui_plot::GridMark {
                value,
                step_size: step,
            });
        }
        value += step;
    }
    marks
}

const TABLE_COLUMNS: [&str; 7] = [
    "Time",
    "Download (Mbps)",
    "Upload (Mbps)",
    "Ping (ms)",
    "Server",
    "Network Interface",
    "ISP",
];

fn draw_table(app: &App, ui: &mut egui::Ui) {
    let rows: Vec<&crate::data_store::SpeedTest> = app.recent.iter().take(TABLE_ROWS).collect();
    egui::ScrollArea::horizontal()
        .id_salt("results-table")
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
            TableBuilder::new(ui)
                .striped(false)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto().at_least(170.0))
                .column(Column::auto().at_least(130.0))
                .column(Column::auto().at_least(120.0))
                .column(Column::auto().at_least(90.0))
                .column(Column::auto().at_least(140.0))
                .column(Column::auto().at_least(150.0))
                .column(Column::remainder().at_least(140.0))
                .header(44.0, |mut header| {
                    for title in TABLE_COLUMNS {
                        header.col(|ui| {
                            let rect = ui.max_rect();
                            ui.painter().rect_filled(rect, 0.0, TABLE_HEADER);
                            ui.painter().hline(
                                rect.x_range(),
                                rect.bottom(),
                                Stroke::new(1.0, BORDER),
                            );
                            ui.add_space(12.0);
                            ui.label(semibold(title, 14.0).color(HEADING));
                        });
                    }
                })
                .body(|mut body| {
                    // Hover highlight uses last frame's hovered row (the row response is
                    // only available once its cells have been added).
                    let hover_id = egui::Id::new("results-table-hovered-row");
                    let hovered_row: Option<usize> =
                        body.ui_mut().ctx().data(|d| d.get_temp(hover_id));
                    for (row_index, test) in rows.into_iter().enumerate() {
                        body.row(44.0, |mut row| {
                            let cells = [
                                format::datetime_long(test.local_time()),
                                format!("{:.2}", test.download),
                                format!("{:.2}", test.upload),
                                format!("{:.2}", test.ping),
                                test.server.clone().unwrap_or_else(|| "Unknown".to_string()),
                                test.network_interface
                                    .clone()
                                    .unwrap_or_else(|| "Unknown".to_string()),
                                test.isp.clone().unwrap_or_else(|| "Unknown".to_string()),
                            ];
                            let is_error =
                                test.error.is_some() || test.server.as_deref() == Some("Error");
                            let hovered = hovered_row == Some(row_index);
                            for (index, cell) in cells.into_iter().enumerate() {
                                row.col(|ui| {
                                    let rect = ui.max_rect();
                                    let painter = ui.painter();
                                    if hovered {
                                        painter.rect_filled(rect, 0.0, TABLE_HEADER);
                                    }
                                    painter.hline(
                                        rect.x_range(),
                                        rect.bottom(),
                                        Stroke::new(1.0, BORDER),
                                    );
                                    ui.add_space(12.0);
                                    let color = if index == 4 && is_error { DANGER } else { TEXT };
                                    ui.label(RichText::new(cell).size(14.0).color(color));
                                });
                            }
                            let response = row.response();
                            if response.hovered() {
                                response
                                    .ctx
                                    .data_mut(|d| d.insert_temp(hover_id, row_index));
                            } else if hovered {
                                response.ctx.data_mut(|d| d.remove_temp::<usize>(hover_id));
                            }
                        });
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::smooth_curve;

    #[test]
    fn smoothing_keeps_endpoints_and_passes_through_points() {
        let points = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 0.0], [3.0, 1.0]];
        let curve = smooth_curve(&points, 0.4, 10);
        assert_eq!(curve.len(), 3 * 10 + 1);
        assert_eq!(curve[0], [0.0, 0.0]);
        assert_eq!(curve[10], [1.0, 1.0]);
        assert_eq!(*curve.last().unwrap(), [3.0, 1.0]);
    }

    #[test]
    fn smoothing_short_series_is_identity() {
        let points = vec![[0.0, 0.0], [1.0, 1.0]];
        assert_eq!(smooth_curve(&points, 0.4, 10), points);
    }
}
