use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};
use crate::data::CameraPose;
use crate::state::PlaybackState;

pub fn show(ui: &mut Ui, poses: &[CameraPose], playback: &PlaybackState) {
    let frame = playback.current_frame;
    let half = 100_usize;
    let start = frame.saturating_sub(half);
    let end = (frame + half).min(poses.len().saturating_sub(1));

    if poses.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("No pose data");
        });
        return;
    }

    let window = &poses[start..=end];

    let x_pts: PlotPoints = window
        .iter()
        .enumerate()
        .map(|(i, p)| [(start + i) as f64, p.position[0] as f64])
        .collect();
    let y_pts: PlotPoints = window
        .iter()
        .enumerate()
        .map(|(i, p)| [(start + i) as f64, p.position[1] as f64])
        .collect();
    let z_pts: PlotPoints = window
        .iter()
        .enumerate()
        .map(|(i, p)| [(start + i) as f64, p.position[2] as f64])
        .collect();

    Plot::new("position_plot")
        .show_axes(true)
        .show_grid(true)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(x_pts).color(egui::Color32::from_rgb(255, 80, 80)).name("X"));
            plot_ui.line(Line::new(y_pts).color(egui::Color32::from_rgb(80, 200, 80)).name("Y"));
            plot_ui.line(Line::new(z_pts).color(egui::Color32::from_rgb(80, 140, 255)).name("Z"));
        });
}
