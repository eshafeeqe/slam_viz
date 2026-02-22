use egui::Ui;
use crate::ui::viewport::ViewportPanel;

pub fn show(ui: &mut Ui, texture_id: egui::TextureId) {
    let size = ui.available_size();
    ViewportPanel::show(ui, texture_id, size);
}
