//! Painel da direita: o que está em uso, "igual à área de trabalho" e detalhes de vídeo.

use eframe::egui::{self, Sense};

use super::{cover_uv, file_name, style, App, Tab, WHITE};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    style::section(ui, "Em uso agora");
    for (tab, label) in [(Tab::Desktop, "área de trabalho"), (Tab::Lock, "tela de bloqueio")] {
        let path = app.in_use(tab);
        ui.horizontal(|ui| {
            let size = egui::vec2(96.0, 54.0);
            let (rect, response) = ui.allocate_exact_size(size, Sense::click());
            match app.loaded.get(&path) {
                Some(l) => {
                    ui.painter_at(rect)
                        .image(l.thumb.id(), rect, cover_uv(l.thumb.size_vec2(), size), WHITE);
                }
                None => {
                    ui.painter_at(rect)
                        .rect_filled(rect, 4.0, egui::Color32::from_rgb(0x1a, 0x1a, 0x1a));
                }
            }
            if response.clicked() {
                app.tab = tab;
                app.selected = Some(path.clone());
            }
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(file_name(&path)).color(egui::Color32::WHITE));
                ui.label(egui::RichText::new(label).small().color(style::DIM));
            });
        });
    }

    ui.add_space(8.0);
    style::section(ui, "Tela de bloqueio");
    let mut same = app.sel.lock_same_as_desktop;
    if ui
        .add_enabled(!app.busy(), egui::Checkbox::new(&mut same, "Igual à área de trabalho"))
        .changed()
    {
        app.set_lock_same(same);
    }

    ui.add_space(8.0);
    style::section(ui, "Vídeo");
    for (k, v) in [("Decodificação", "GPU"), ("Som", "desligado"), ("Loop", "infinito")] {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(k).color(style::DIM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(v));
        });
    }
}
