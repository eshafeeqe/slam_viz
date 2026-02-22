use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};
use crate::data::CameraPose;
use crate::state::PlaybackState;

pub fn show(ui: &mut Ui, poses: &[CameraPose], _playback: &PlaybackState) {
    if poses.len() < 2 {
        ui.centered_and_justified(|ui| {
            ui.label("Not enough pose data");
        });
        return;
    }

    let speeds: Vec<[f64; 2]> = poses
        .windows(2)
        .enumerate()
        .map(|(i, w)| {
            let dt = (w[1].timestamp - w[0].timestamp) as f32;
            let dx = w[1].position[0] - w[0].position[0];
            let dy = w[1].position[1] - w[0].position[1];
            let dz = w[1].position[2] - w[0].position[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let speed = if dt > 1e-6 { dist / dt } else { 0.0 };
            [i as f64, speed as f64]
        })
        .collect();

    Plot::new("speed_plot")
        .show_axes(true)
        .show_grid(true)
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(PlotPoints::new(speeds))
                    .color(egui::Color32::from_rgb(255, 140, 0))
                    .name("Speed"),
            );
        });
}
