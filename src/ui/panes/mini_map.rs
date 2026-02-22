use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use crate::data::CameraPose;
use crate::state::PlaybackState;

pub fn show(ui: &mut Ui, poses: &[CameraPose], playback: &PlaybackState) {
    Plot::new("mini_map")
        .data_aspect(1.0)
        .show_axes(true)
        .show_grid(true)
        .show(ui, |plot_ui| {
            // Full XZ trajectory (cyan)
            if poses.len() >= 2 {
                let traj: PlotPoints = poses
                    .iter()
                    .map(|p| [p.position[0] as f64, p.position[2] as f64])
                    .collect();
                plot_ui.line(Line::new(traj).color(egui::Color32::from_rgb(0, 188, 212)).name("Trajectory"));
            }

            // Current pose dot (white)
            if let Some(pose) = poses.get(playback.current_frame) {
                let pt = PlotPoints::new(vec![[
                    pose.position[0] as f64,
                    pose.position[2] as f64,
                ]]);
                plot_ui.points(
                    Points::new(pt)
                        .color(egui::Color32::WHITE)
                        .radius(5.0)
                        .name("Current"),
                );
            }
        });
}
