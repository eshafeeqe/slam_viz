use egui::{Ui, TextureId, Vec2};

pub struct ViewportPanel;

impl ViewportPanel {
    pub fn show(ui: &mut Ui, texture_id: TextureId, size: Vec2) {
        ui.image(egui::load::SizedTexture::new(texture_id, size));
    }
}
