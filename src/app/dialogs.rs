//! Modal dialogs: alerts, confirmations, clear-until, export range, about, debug log.

use egui::{Align, Color32, CornerRadius, Layout, RichText, Stroke, TextStyle};

use super::format;
use super::view::{
    BORDER, DANGER, HEADING, LINK, MUTED, PRIMARY, SECONDARY, bold, semibold, solid_button,
};
use super::{App, ExportDialog};
use crate::CURRENT_VERSION;

const MODAL_WIDTH: f32 = 460.0;

fn modal_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(0xfe, 0xfe, 0xfe))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::same(30))
        .shadow(egui::epaint::Shadow {
            offset: [0, 20],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(70),
        })
}

fn modal_title(ui: &mut egui::Ui, title: &str) {
    ui.label(bold(title, 20.0).color(HEADING));
    ui.add_space(10.0);
}

fn filled_button(label: &str, fill: Color32) -> egui::Button<'static> {
    solid_button(label, fill)
}

pub fn draw(app: &mut App, ctx: &egui::Context) {
    draw_debug_log(app, ctx);
    draw_about(app, ctx);
    draw_export_dialog(app, ctx);
    draw_clear_dialog(app, ctx);
    draw_confirm_clear_history(app, ctx);
    draw_alert(app, ctx);
}

fn draw_alert(app: &mut App, ctx: &egui::Context) {
    let Some(alert) = app.alerts.front().cloned() else {
        return;
    };
    let mut close = false;
    let response = egui::Modal::new(egui::Id::new("alert-modal"))
        .frame(modal_frame())
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);
            modal_title(ui, &alert.title);
            ui.label(RichText::new(&alert.message).color(MUTED));
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(url) = alert.download_url.as_deref() {
                    if ui.add(filled_button("Later", SECONDARY)).clicked() {
                        close = true;
                    }
                    if ui.add(filled_button("Download Update", PRIMARY)).clicked() {
                        app.open_external(url);
                        close = true;
                    }
                } else if ui.add(filled_button("OK", PRIMARY)).clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    close = true;
                }
            });
        });
    if close || response.should_close() {
        app.alerts.pop_front();
    }
}

fn draw_confirm_clear_history(app: &mut App, ctx: &egui::Context) {
    if !app.confirm_clear_history {
        return;
    }
    let mut decision: Option<bool> = None;
    let response = egui::Modal::new(egui::Id::new("confirm-clear-history"))
        .frame(modal_frame())
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);
            modal_title(ui, "Clear All History");
            ui.label(
                RichText::new(
                    "Are you sure you want to clear all speed test history? This action cannot be undone.",
                )
                .color(MUTED),
            );
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(filled_button("Cancel", SECONDARY)).clicked() {
                    decision = Some(false);
                }
                if ui.add(filled_button("Clear History", DANGER)).clicked() {
                    decision = Some(true);
                }
            });
        });
    if response.should_close() {
        decision = Some(false);
    }
    if let Some(confirmed) = decision {
        app.confirm_clear_history = false;
        if confirmed {
            app.clear_history();
        }
    }
}

fn draw_clear_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(mut dialog) = app.clear_dialog.clone() else {
        return;
    };
    #[derive(PartialEq)]
    enum Action {
        None,
        Preview,
        Confirm,
        Cancel,
    }
    let mut action = Action::None;
    let min_year = dialog
        .earliest
        .format("%Y")
        .to_string()
        .parse::<i16>()
        .unwrap_or(2000);
    let max_year = dialog
        .latest
        .format("%Y")
        .to_string()
        .parse::<i16>()
        .unwrap_or(2100);

    let response = egui::Modal::new(egui::Id::new("clear-data-modal"))
        .frame(modal_frame())
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);
            modal_title(ui, "Clear Data Until Date");
            ui.label(
                RichText::new(
                    "Remove all speed test data recorded before the selected date and time.",
                )
                .color(MUTED),
            );
            ui.add_space(12.0);
            ui.label(semibold("Clear data before:", 14.0).color(HEADING));
            ui.horizontal(|ui| {
                let changed_before = (dialog.date, dialog.hour, dialog.minute);
                ui.add(
                    egui_extras::DatePickerButton::new(&mut dialog.date)
                        .id_salt("cutoff-date")
                        .show_icon(false)
                        .calendar_week(false)
                        .start_end_years(min_year..=max_year),
                );
                ui.add(
                    egui::DragValue::new(&mut dialog.hour)
                        .range(0..=23)
                        .speed(0.1),
                );
                ui.label(":");
                ui.add(
                    egui::DragValue::new(&mut dialog.minute)
                        .range(0..=59)
                        .speed(0.1),
                );
                if changed_before != (dialog.date, dialog.hour, dialog.minute) {
                    dialog.preview = None;
                }
            });
            ui.label(
                RichText::new(format!(
                    "Valid range: {} to {}",
                    format::datetime_long(dialog.earliest),
                    format::datetime_long(dialog.latest)
                ))
                .small()
                .italics()
                .color(MUTED),
            );

            if let Some(preview) = dialog.preview.as_ref() {
                ui.add_space(10.0);
                egui::Frame::new()
                    .fill(Color32::from_rgb(0xf8, 0xf9, 0xfa))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::same(14))
                    .stroke(Stroke::new(1.0, BORDER))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (line, danger) in &preview.lines {
                            let text = if *danger {
                                semibold(line, 14.0).color(DANGER)
                            } else {
                                RichText::new(line)
                            };
                            ui.label(text);
                        }
                    });
            }

            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(filled_button("Cancel", PRIMARY)).clicked() {
                    action = Action::Cancel;
                }
                let can_clear = dialog.preview.as_ref().is_some_and(|p| p.can_clear);
                if ui
                    .add_enabled(can_clear, filled_button("Clear Data", DANGER))
                    .clicked()
                {
                    action = Action::Confirm;
                }
                if ui.add(filled_button("Preview", SECONDARY)).clicked() {
                    action = Action::Preview;
                }
            });
        });

    if response.should_close() {
        action = Action::Cancel;
    }
    app.clear_dialog = Some(dialog);
    match action {
        Action::None => {}
        Action::Preview => app.preview_clear(),
        Action::Confirm => app.confirm_clear(),
        Action::Cancel => app.clear_dialog = None,
    }
}

fn draw_export_dialog(app: &mut App, ctx: &egui::Context) {
    let Some(mut dialog) = app.export_dialog.clone() else {
        return;
    };
    #[derive(PartialEq)]
    enum Action {
        None,
        ExportRange,
        ExportAll,
        Cancel,
    }
    let mut action = Action::None;
    let years = dialog.min.year()..=dialog.max.year();

    let response = egui::Modal::new(egui::Id::new("export-modal"))
        .frame(modal_frame())
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);
            modal_title(ui, "Export Speed Test Data");
            ui.label(
                RichText::new("Select a date range to export speed test results as CSV.")
                    .color(MUTED),
            );
            ui.add_space(12.0);

            let before = (dialog.start, dialog.end);
            ui.label(semibold("Start date:", 14.0).color(HEADING));
            ui.add(
                egui_extras::DatePickerButton::new(&mut dialog.start)
                    .id_salt("export-start")
                    .show_icon(false)
                    .calendar_week(false)
                    .start_end_years(years.clone()),
            );
            ui.label(
                RichText::new("Earliest date to include in export")
                    .small()
                    .italics()
                    .color(MUTED),
            );
            ui.add_space(8.0);
            ui.label(semibold("End date:", 14.0).color(HEADING));
            ui.add(
                egui_extras::DatePickerButton::new(&mut dialog.end)
                    .id_salt("export-end")
                    .show_icon(false)
                    .calendar_week(false)
                    .start_end_years(years),
            );
            ui.label(
                RichText::new("Latest date to include in export")
                    .small()
                    .italics()
                    .color(MUTED),
            );

            dialog.start = dialog.start.clamp(dialog.min, dialog.max);
            dialog.end = dialog.end.clamp(dialog.min, dialog.max);
            if dialog.start > dialog.end {
                std::mem::swap(&mut dialog.start, &mut dialog.end);
            }
            if before != (dialog.start, dialog.end) {
                dialog.record_count = app.count_export_records(&dialog);
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                ui.label(RichText::new("Records to export:").small().color(MUTED));
                ui.label(semibold(dialog.record_count.to_string(), 12.0));
            });

            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(filled_button("Cancel", SECONDARY)).clicked() {
                    action = Action::Cancel;
                }
                if ui
                    .add_enabled(dialog.has_data, filled_button("Export All", PRIMARY))
                    .clicked()
                {
                    action = Action::ExportAll;
                }
                if ui
                    .add_enabled(
                        dialog.record_count > 0,
                        filled_button("Export Date Range", PRIMARY),
                    )
                    .clicked()
                {
                    action = Action::ExportRange;
                }
            });
        });

    if response.should_close() {
        action = Action::Cancel;
    }
    let ExportDialog { start, end, .. } = dialog;
    app.export_dialog = Some(dialog);
    match action {
        Action::None => {}
        Action::ExportRange => app.export_range(start, end),
        Action::ExportAll => app.export_all(),
        Action::Cancel => app.export_dialog = None,
    }
}

fn draw_about(app: &mut App, ctx: &egui::Context) {
    if !app.about_open {
        return;
    }
    let mut close = false;
    let mut check = false;
    let response = egui::Modal::new(egui::Id::new("about-modal"))
        .frame(modal_frame())
        .show(ctx, |ui| {
            ui.set_width(MODAL_WIDTH);
            modal_title(ui, &format!("Internet Speed Monitor v{CURRENT_VERSION}"));
            ui.label(
                RichText::new(
                    "A simple tool to monitor your internet connection speed at regular intervals.",
                )
                .color(MUTED),
            );
            ui.add_space(8.0);
            ui.label(semibold("Features:", 14.0));
            for feature in [
                "Automated speed testing at set intervals",
                "Real-time speed monitoring display",
                "Historical data visualization with charts",
                "CSV export functionality",
                "Debug logging for troubleshooting",
            ] {
                ui.label(format!("• {feature}"));
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("Licensed under Unlicense")
                    .small()
                    .color(MUTED),
            );
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.hyperlink_to(
                    RichText::new("GitHub").small().color(LINK),
                    crate::REPOSITORY_URL,
                );
                ui.label(RichText::new("·").small().color(MUTED));
                ui.hyperlink_to(
                    RichText::new("Documentation").small().color(LINK),
                    crate::DOCUMENTATION_URL,
                );
            });
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(filled_button("OK", PRIMARY)).clicked() {
                    close = true;
                }
                let label = if app.update_check_busy {
                    "Checking..."
                } else {
                    "Check for Updates"
                };
                if ui
                    .add_enabled(!app.update_check_busy, filled_button(label, SECONDARY))
                    .clicked()
                {
                    check = true;
                }
            });
        });
    if check {
        app.check_for_updates(true);
        close = true;
    }
    if close || response.should_close() {
        app.about_open = false;
    }
}

fn draw_debug_log(app: &mut App, ctx: &egui::Context) {
    let Some(content) = app.debug_log.as_ref() else {
        return;
    };
    let mut open = true;
    let mut clear = false;
    egui::Window::new("Debug Log")
        .open(&mut open)
        .default_size([800.0, 600.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(app.paths.debug_log.display().to_string())
                        .small()
                        .color(MUTED),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Clear log").clicked() {
                        clear = true;
                    }
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(content.clone());
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let text = if content.is_empty() {
                        "(log is empty)"
                    } else {
                        content.as_str()
                    };
                    ui.add(
                        egui::TextEdit::multiline(&mut text.to_owned().as_str())
                            .font(TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
        });
    if clear {
        app.clear_debug_log();
    }
    if !open {
        app.debug_log = None;
    }
}
