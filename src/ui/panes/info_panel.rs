use egui::{Color32, Stroke, Ui};
use crate::data::CameraPose;
use crate::state::PlaybackState;
use crate::renderer::OrbitCamera;

const BG_SURFACE: Color32 = Color32::from_rgb(32, 32, 42);
const BORDER:     Color32 = Color32::from_rgb(48, 48, 62);
const ACCENT:     Color32 = Color32::from_rgb(0, 188, 212);
const TEXT_DIM:   Color32 = Color32::from_rgb(140, 140, 158);

fn section_header(ui: &mut Ui, label: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(label).size(10.5).color(ACCENT).strong());
    ui.add(egui::Separator::default().spacing(2.0));
}

fn kv_row(ui: &mut Ui, key: &str, val: String) {
    ui.label(egui::RichText::new(key).size(12.0).color(TEXT_DIM));
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(egui::RichText::new(val).monospace().size(12.0));
    });
    ui.end_row();
}

pub fn show(
    ui: &mut Ui,
    poses: &[CameraPose],
    playback: &PlaybackState,
    camera: &mut OrbitCamera,
    error_msg: &mut Option<String>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(pose) = poses.get(playback.current_frame) {
            let [px, py, pz] = pose.position;
            let [qx, qy, qz, qw] = pose.orientation;

            // POSE
            section_header(ui, "POSE");
            egui::Grid::new("pose_meta")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 3.0))
                .show(ui, |ui| {
                    kv_row(ui, "Frame", format!("{}", playback.current_frame));
                    kv_row(ui, "Time", format!("{:.3}s", pose.timestamp));
                });

            // POSITION
            section_header(ui, "POSITION");
            egui::Grid::new("pose_pos")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 3.0))
                .show(ui, |ui| {
                    for (lbl, v) in [("X", px), ("Y", py), ("Z", pz)] {
                        kv_row(ui, lbl, format!("{:.4}", v));
                    }
                });

            // EULER (DEG)
            let q = glam::Quat::from_xyzw(qx, qy, qz, qw);
            let (y, x, z) = q.to_euler(glam::EulerRot::YXZ);
            section_header(ui, "EULER (DEG)");
            egui::Grid::new("pose_euler")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 3.0))
                .show(ui, |ui| {
                    kv_row(ui, "Yaw",   format!("{:.1}°", y.to_degrees()));
                    kv_row(ui, "Pitch", format!("{:.1}°", x.to_degrees()));
                    kv_row(ui, "Roll",  format!("{:.1}°", z.to_degrees()));
                });

            // QUATERNION
            section_header(ui, "QUATERNION");
            egui::Grid::new("pose_quat")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 3.0))
                .show(ui, |ui| {
                    for (lbl, v) in [("X", qx), ("Y", qy), ("Z", qz), ("W", qw)] {
                        kv_row(ui, lbl, format!("{:.4}", v));
                    }
                });
        } else {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("No pose data").color(TEXT_DIM));
        }

        // CAMERA
        let cam_pos = camera.position();
        section_header(ui, "CAMERA");
        egui::Grid::new("cam_info")
            .num_columns(2)
            .spacing(egui::vec2(12.0, 3.0))
            .show(ui, |ui| {
                kv_row(ui, "Dist", format!("{:.2}", camera.distance));
                kv_row(ui, "X", format!("{:.1}", cam_pos.x));
                kv_row(ui, "Y", format!("{:.1}", cam_pos.y));
                kv_row(ui, "Z", format!("{:.1}", cam_pos.z));
            });

        ui.add_space(8.0);
        let btn_size = egui::vec2(ui.available_width(), 26.0);
        if ui
            .add(
                egui::Button::new("Reset Camera")
                    .min_size(btn_size)
                    .fill(BG_SURFACE)
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .clicked()
        {
            camera.reset();
        }
        ui.add_space(4.0);
        let btn_size = egui::vec2(ui.available_width(), 26.0);
        if ui
            .add(
                egui::Button::new("Fit to Scene")
                    .min_size(btn_size)
                    .fill(BG_SURFACE)
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .clicked()
        {
            camera.fit_to_scene(poses);
        }

        // CONTROLS
        section_header(ui, "CONTROLS");
        egui::Grid::new("controls")
            .num_columns(2)
            .spacing(egui::vec2(8.0, 2.0))
            .show(ui, |ui| {
                for (key, desc) in [
                    ("MMB drag",  "orbit"),
                    ("Shift+MMB", "pan"),
                    ("Ctrl+MMB",  "zoom"),
                    ("Scroll",    "zoom"),
                    ("Num 7/1/3", "top/front/right"),
                    ("Num 5",     "reset view"),
                ] {
                    ui.label(
                        egui::RichText::new(key)
                            .monospace()
                            .size(11.0)
                            .color(ACCENT),
                    );
                    ui.label(
                        egui::RichText::new(desc)
                            .size(11.0)
                            .color(Color32::from_rgb(110, 110, 128)),
                    );
                    ui.end_row();
                }
            });

        // Error display
        if let Some(err) = error_msg.clone() {
            ui.add_space(8.0);
            ui.separator();
            ui.colored_label(egui::Color32::from_rgb(255, 80, 80), format!("Error: {}", err));
            if ui.button("Dismiss").clicked() {
                *error_msg = None;
            }
        }
    });
}
