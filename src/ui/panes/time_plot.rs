use egui::Ui;
use egui_plot::{Line, Plot, PlotBounds, PlotPoints, VLine};
use crate::data::CameraPose;
use crate::state::PlaybackState;
use crate::ui::pane_kind::TimePlotConfig;

pub const DEFAULT_HALF_WINDOW: f64 = 2.0; // default ±2 s = 4 s window

/// `height`: fixed pixel height for inline use; `None` fills the available pane height.
/// `half_window`: half the X-axis span in seconds.
pub fn show(
    ui: &mut Ui,
    poses: &[CameraPose],
    playback: &PlaybackState,
    cfg: &TimePlotConfig,
    height: Option<f32>,
    half_window: f64,
) {
    if poses.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("No pose data");
        });
        return;
    }

    let traj_start = poses.first().map(|p| p.timestamp).unwrap_or(0.0);
    let current_ts = poses
        .get(playback.current_frame)
        .map(|p| p.timestamp)
        .unwrap_or(traj_start);

    // The view starts anchored to the trajectory start.
    // Once the marker reaches the midpoint it starts scrolling so the marker
    // stays pinned at the centre.
    let x_left = if current_ts - traj_start < half_window {
        traj_start
    } else {
        current_ts - half_window
    };
    let x_right = x_left + half_window * 2.0;

    // Build all plot points (full trajectory so the line is always present)
    let pts: PlotPoints = poses
        .iter()
        .enumerate()
        .map(|(i, pose)| {
            let prev = if i > 0 { Some(&poses[i - 1]) } else { None };
            [pose.timestamp, cfg.field.value_at(pose, prev)]
        })
        .collect();

    // Y range fitted to data visible in the current window
    let (y_min_raw, y_max_raw) = poses
        .iter()
        .enumerate()
        .filter(|(_, p)| p.timestamp >= x_left && p.timestamp <= x_right)
        .map(|(i, p)| {
            let prev = if i > 0 { Some(&poses[i - 1]) } else { None };
            cfg.field.value_at(p, prev)
        })
        .fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(mn, mx), v| (mn.min(v), mx.max(v)),
        );

    let y_pad = ((y_max_raw - y_min_raw) * 0.15).max(0.1);
    let y_lo = if y_min_raw.is_finite() { y_min_raw - y_pad } else { -1.0 };
    let y_hi = if y_max_raw.is_finite() { y_max_raw + y_pad } else { 1.0 };

    let mut plot = Plot::new(format!("time_plot_{}", cfg.field.label()))
        .show_axes(true)
        .show_grid(true)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .label_formatter(|name, val| format!("{name}: {:.4}", val.y));
    if let Some(h) = height {
        plot = plot.height(h);
    }
    plot.show(ui, |plot_ui| {
        plot_ui.set_plot_bounds(PlotBounds::from_min_max(
            [x_left, y_lo],
            [x_right, y_hi],
        ));
        plot_ui.line(
            Line::new(pts)
                .color(cfg.color)
                .name(cfg.field.label()),
        );
        plot_ui.vline(
            VLine::new(current_ts)
                .color(egui::Color32::from_rgba_premultiplied(255, 255, 255, 120))
                .width(1.5),
        );
    });
}
