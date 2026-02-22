use egui::{Color32, Context, Stroke, TextureId};
use crate::data::CameraPose;
use crate::state::PlaybackState;
use crate::renderer::OrbitCamera;
use super::timeline::TimelinePanel;
use super::pane_kind::PaneKind;
use super::tile_behavior::{PaneContext, SlamBehavior};

// ── Color palette constants ───────────────────────────────────────────────────
const BG_DEEP:  Color32 = Color32::from_rgb(18, 18, 24);
const BORDER:   Color32 = Color32::from_rgb(48, 48, 62);

pub fn show_ui(
    ctx: &Context,
    texture_id: TextureId,
    poses: &[CameraPose],
    playback: &mut PlaybackState,
    camera: &mut OrbitCamera,
    open_file_requested: &mut bool,
    error_msg: &mut Option<String>,
    tile_tree: &mut egui_tiles::Tree<PaneKind>,
) {
    // ── Top menu bar ──────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(BG_DEEP)
                .stroke(Stroke::new(1.0, BORDER)),
        )
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open…").clicked() {
                        *open_file_requested = true;
                        ui.close_menu();
                    }
                });
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("{} poses", poses.len()))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(0, 188, 212)),
                );
            });
        });

    // ── Bottom timeline panel ─────────────────────────────────────────────────
    egui::TopBottomPanel::bottom("timeline")
        .min_height(76.0)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(Color32::from_rgb(20, 20, 28))
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(egui::Margin {
                    left: 0.0,
                    right: 0.0,
                    top: 6.0,
                    bottom: 6.0,
                }),
        )
        .show(ctx, |ui| {
            TimelinePanel::show(ui, playback, poses);
        });

    // ── Central tiled panel ───────────────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(BG_DEEP))
        .show(ctx, |ui| {
            let mut behavior = SlamBehavior {
                ctx: PaneContext {
                    poses,
                    playback,
                    camera,
                    scene_texture_id: texture_id,
                    open_file_req: open_file_requested,
                    error_msg,
                },
            };
            tile_tree.ui(&mut behavior, ui);
        });
}
