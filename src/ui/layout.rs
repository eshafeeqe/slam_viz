use egui::{Context, TextureId, Vec2};
use crate::data::CameraPose;
use crate::state::PlaybackState;
use crate::renderer::OrbitCamera;
use super::{timeline::TimelinePanel, viewport::ViewportPanel};

pub fn show_ui(
    ctx: &Context,
    texture_id: TextureId,
    poses: &[CameraPose],
    playback: &mut PlaybackState,
    camera: &mut OrbitCamera,
    open_file_requested: &mut bool,
    error_msg: &mut Option<String>,
) {
    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open…").clicked() {
                    *open_file_requested = true;
                    ui.close_menu();
                }
            });
            ui.separator();
            ui.label(format!("{} poses loaded", poses.len()));
        });
    });

    // Bottom timeline panel
    egui::TopBottomPanel::bottom("timeline").min_height(70.0).show(ctx, |ui| {
        ui.add_space(4.0);
        TimelinePanel::show(ui, playback, poses);
    });

    // Right info panel
    egui::SidePanel::right("info_panel").min_width(220.0).show(ctx, |ui| {
        ui.heading("Info");
        ui.separator();

        if let Some(pose) = poses.get(playback.current_frame) {
            let [px, py, pz] = pose.position;
            let [qx, qy, qz, qw] = pose.orientation;

            ui.label(format!("Frame: {}", playback.current_frame));
            ui.label(format!("Time: {:.3}s", pose.timestamp));
            ui.separator();
            ui.label("Position:");
            ui.label(format!("  X: {:.3}", px));
            ui.label(format!("  Y: {:.3}", py));
            ui.label(format!("  Z: {:.3}", pz));
            ui.separator();
            ui.label("Orientation (XYZW):");
            ui.label(format!("  {:.3}, {:.3}, {:.3}, {:.3}", qx, qy, qz, qw));

            // Convert quaternion to Euler for readability
            let q = glam::Quat::from_xyzw(qx, qy, qz, qw);
            let (y, x, z) = q.to_euler(glam::EulerRot::YXZ);
            ui.separator();
            ui.label("Euler (deg):");
            ui.label(format!("  Yaw:   {:.1}°", y.to_degrees()));
            ui.label(format!("  Pitch: {:.1}°", x.to_degrees()));
            ui.label(format!("  Roll:  {:.1}°", z.to_degrees()));
        } else {
            ui.label("No pose data");
        }

        ui.separator();
        ui.label("Camera:");
        let cam_pos = camera.position();
        ui.label(format!("  Dist: {:.2}", camera.distance));
        ui.label(format!("  Pos: ({:.1}, {:.1}, {:.1})", cam_pos.x, cam_pos.y, cam_pos.z));

        ui.add_space(8.0);
        if ui.button("Reset Camera").clicked() {
            camera.reset();
        }
        if ui.button("Fit to Scene").clicked() {
            camera.fit_to_scene(poses);
        }

        ui.add_space(12.0);
        ui.separator();
        ui.weak("Controls (Blender):");
        ui.weak("  MMB drag        — orbit");
        ui.weak("  Shift+MMB drag  — pan");
        ui.weak("  Ctrl+MMB drag   — zoom");
        ui.weak("  Scroll          — zoom");
        ui.weak("  Num7/1/3        — top/front/right");
        ui.weak("  Num5            — reset view");

        // Error display
        if let Some(err) = error_msg.clone() {
            ui.separator();
            ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            if ui.button("Dismiss").clicked() {
                *error_msg = None;
            }
        }
    });

    // Central 3D viewport
    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_size();
        ViewportPanel::show(ui, texture_id, Vec2::new(available.x, available.y));
    });
}
