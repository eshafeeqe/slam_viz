use egui::Ui;
use crate::state::PlaybackState;
use crate::data::CameraPose;

pub struct TimelinePanel;

impl TimelinePanel {
    pub fn show(ui: &mut Ui, playback: &mut PlaybackState, poses: &[CameraPose]) {
        ui.horizontal(|ui| {
            // Play/Pause button
            let btn_label = if playback.is_playing { "⏸ Pause" } else { "▶ Play" };
            if ui.button(btn_label).clicked() {
                playback.toggle_play();
            }

            // Speed selector
            ui.label("Speed:");
            for &speed in &[0.25_f32, 0.5, 1.0, 2.0] {
                let label = format!("{speed}×");
                let selected = (playback.playback_speed - speed).abs() < 0.01;
                if ui.selectable_label(selected, &label).clicked() {
                    playback.playback_speed = speed;
                }
            }
        });

        ui.add_space(4.0);

        // Timeline slider
        let mut frame = playback.current_frame;
        let max = playback.total_frames.saturating_sub(1);
        let slider = egui::Slider::new(&mut frame, 0..=max)
            .show_value(false)
            .text("");
        if ui.add(slider).changed() {
            playback.seek(frame);
        }

        // Info row
        ui.horizontal(|ui| {
            let ts = if !poses.is_empty() {
                poses.get(playback.current_frame).map(|p| p.timestamp).unwrap_or(0.0)
            } else {
                0.0
            };
            ui.label(format!("Frame: {} / {}", playback.current_frame, playback.total_frames.saturating_sub(1)));
            ui.separator();
            ui.label(format!("Time: {:.3}s", ts));
        });
    }
}
