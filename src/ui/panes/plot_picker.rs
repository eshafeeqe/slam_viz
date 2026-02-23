use egui::Ui;
use crate::data::CameraPose;
use crate::state::PlaybackState;
use crate::ui::pane_kind::{PlotField, TimePlotConfig};
use super::time_plot::{self, DEFAULT_HALF_WINDOW};

const ACCENT:             egui::Color32 = egui::Color32::from_rgb(0, 188, 212);
const TEXT_DIM:           egui::Color32 = egui::Color32::from_rgb(140, 140, 158);
const INLINE_PLOT_HEIGHT: f32 = 140.0;

pub fn show(ui: &mut Ui, poses: &[CameraPose], playback: &PlaybackState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        group(ui, "pos", "Position", true,
              &[PlotField::PositionX, PlotField::PositionY, PlotField::PositionZ],
              poses, playback);

        ui.add_space(4.0);

        group(ui, "dyn", "Dynamics", false,
              &[PlotField::Speed],
              poses, playback);
    });
}

/// One collapsible group.  Header row = "[▶/▼] Label ··· w: [drag]"
fn group(
    ui: &mut Ui,
    key: &str,
    label: &str,
    default_open: bool,
    fields: &[PlotField],
    poses: &[CameraPose],
    playback: &PlaybackState,
) {
    let win_id  = egui::Id::new(format!("{key}_window"));
    let open_id = egui::Id::new(format!("{key}_open"));

    let mut full_secs: f64 = ui.memory(|m| {
        m.data.get_temp(win_id).unwrap_or(DEFAULT_HALF_WINDOW * 2.0)
    });
    let mut is_open: bool = ui.memory(|m| {
        m.data.get_temp(open_id).unwrap_or(default_open)
    });

    // ── Header row ────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        // Arrow + label — clicking anywhere toggles the group
        let arrow = if is_open { "▼" } else { "▶" };
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!("{arrow} {label}"))
                        .color(ACCENT)
                        .strong(),
                )
                .frame(false),
            )
            .clicked()
        {
            is_open = !is_open;
        }

        // Window control pinned to the right side
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let resp = ui.add(
                egui::DragValue::new(&mut full_secs)
                    .suffix("s")
                    .speed(0.1)
                    .range(0.5..=120.0),
            );
            // Also accept scroll wheel over the drag widget
            if resp.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    full_secs = (full_secs + scroll as f64 * 0.2).clamp(0.5, 120.0);
                }
            }
            ui.label(egui::RichText::new("w:").size(10.0).color(TEXT_DIM));
        });
    });

    // ── Body ──────────────────────────────────────────────────────────────────
    if is_open {
        let half = full_secs / 2.0;
        ui.indent(format!("{key}_body"), |ui| {
            for field in fields {
                plot_row(ui, field.clone(), poses, playback, half);
            }
        });
    }

    // Persist state
    ui.memory_mut(|m| {
        m.data.insert_temp(win_id, full_secs);
        m.data.insert_temp(open_id, is_open);
    });
}

fn plot_row(
    ui: &mut Ui,
    field: PlotField,
    poses: &[CameraPose],
    playback: &PlaybackState,
    half_window: f64,
) {
    egui::CollapsingHeader::new(
        egui::RichText::new(field.label()).color(field.default_color()),
    )
    .id_salt(format!("plot_{}", field.label()))
    .default_open(false)
    .show(ui, |ui| {
        let cfg = TimePlotConfig::new(field);
        time_plot::show(ui, poses, playback, &cfg, Some(INLINE_PLOT_HEIGHT), half_window);
    });
}
