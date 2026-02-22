use egui::{Color32, Stroke, Ui};
use crate::state::PlaybackState;
use crate::data::CameraPose;

const ACCENT: Color32 = Color32::from_rgb(0, 188, 212);
const TEXT_DIM: Color32 = Color32::from_rgb(140, 140, 158);

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, playback: &mut PlaybackState, poses: &[CameraPose]) {
        ui.horizontal(|ui| {
            ui.add_space(12.0); 
            // ── Play / Pause button ──────────────────────────────────────────
            let icon = if playback.is_playing { "⏸" } else { "▶" };
            let play_btn = egui::Button::new(
                egui::RichText::new(icon).size(16.0).color(Color32::WHITE),
            )
            .min_size(egui::vec2(36.0, 28.0))
            .fill(Color32::from_rgb(0, 100, 116))
            .stroke(Stroke::new(1.0, ACCENT))
            .rounding(egui::Rounding::same(6.0));
            if ui.add(play_btn).clicked() {
                playback.toggle_play();
            }

            ui.add_space(8.0);

            // ── Speed selector — segmented buttons ──────────────────────────
            for &speed in &[0.25_f32, 0.5, 1.0, 2.0] {
                let selected = (playback.playback_speed - speed).abs() < 0.01;
                let (fill, text_color, stroke) = if selected {
                    (
                        Color32::from_rgb(0, 100, 116),
                        ACCENT,
                        Stroke::new(1.0, ACCENT),
                    )
                } else {
                    (
                        Color32::TRANSPARENT,
                        TEXT_DIM,
                        Stroke::new(1.0, Color32::from_rgb(48, 48, 62)),
                    )
                };
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{speed}×")).size(12.0).color(text_color),
                )
                .min_size(egui::vec2(36.0, 22.0))
                .fill(fill)
                .stroke(stroke)
                .rounding(egui::Rounding::same(4.0));
                if ui.add(btn).clicked() {
                    playback.playback_speed = speed;
                }
            }
        });

        ui.add_space(4.0);

        // ── Timeline slider — full-width, trailing fill ──────────────────────
        let mut frame = playback.current_frame;
        let max = playback.total_frames.saturating_sub(1);
        let width = ui.available_width();
        ui.style_mut().spacing.slider_width = width;
        let slider = egui::Slider::new(&mut frame, 0..=max)
            .show_value(false)
            .trailing_fill(true);
        if ui.add(slider).changed() {
            playback.seek(frame);
        }

        // ── Info row ─────────────────────────────────────────────────────────
        let ts = poses
            .get(playback.current_frame)
            .map(|p| p.timestamp)
            .unwrap_or(0.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0); 
            ui.label(
                egui::RichText::new(format!("{} / {}", playback.current_frame, max))
                    .monospace()
                    .size(11.0)
                    .color(TEXT_DIM),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0); 
                ui.label(
                    egui::RichText::new(format!("{:.3}s", ts))
                        .monospace()
                        .size(11.0)
                        .color(ACCENT),
                );
            });
        });
    }
}
