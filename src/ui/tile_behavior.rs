use crate::data::CameraPose;
use crate::renderer::OrbitCamera;
use crate::state::PlaybackState;
use super::pane_kind::PaneKind;
use super::panes;

pub struct PaneContext<'a> {
    pub poses: &'a [CameraPose],
    pub playback: &'a PlaybackState,
    pub camera: &'a mut OrbitCamera,
    pub scene_texture_id: egui::TextureId,
    pub open_file_req: &'a mut bool,
    pub error_msg: &'a mut Option<String>,
}

pub struct SlamBehavior<'a> {
    pub ctx: PaneContext<'a>,
}

impl<'a> egui_tiles::Behavior<PaneKind> for SlamBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneKind,
    ) -> egui_tiles::UiResponse {
        match pane {
            PaneKind::View3D => {
                // Capture the pane bounds before rendering content.
                let pane_rect = ui.clip_rect();
                panes::view3d::show(ui, self.ctx.scene_texture_id);
                // Scroll-wheel zoom — only when the pointer is inside the viewport pane.
                // raw_scroll_delta is in logical pixels; ~50 px per wheel click,
                // scaled to ~1.0 to match the original LineDelta behaviour.
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 && ui.rect_contains_pointer(pane_rect) {
                    self.ctx.camera.zoom(scroll * 0.02);
                }
            }
            PaneKind::InfoPanel => {
                panes::info_panel::show(
                    ui,
                    self.ctx.poses,
                    self.ctx.playback,
                    self.ctx.camera,
                    self.ctx.error_msg,
                );
            }
            PaneKind::MiniMap => {
                panes::mini_map::show(ui, self.ctx.poses, self.ctx.playback);
            }
            PaneKind::PositionPlot => {
                panes::position_plot::show(ui, self.ctx.poses, self.ctx.playback);
            }
            PaneKind::SpeedPlot => {
                panes::speed_plot::show(ui, self.ctx.poses, self.ctx.playback);
            }
            PaneKind::TimePlot(cfg) => {
                panes::time_plot::show(
                    ui, self.ctx.poses, self.ctx.playback, cfg,
                    None, panes::time_plot::DEFAULT_HALF_WINDOW,
                );
            }
            PaneKind::PlotPicker => {
                panes::plot_picker::show(ui, self.ctx.poses, self.ctx.playback);
            }
        }
        egui_tiles::UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &PaneKind) -> egui::WidgetText {
        pane.title().into()
    }
}
