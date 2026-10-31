//! Miniaturas de ~/Wallpapers: borda ciano no selecionado, ponto verde no que está em uso.

use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind};

use super::{cover_uv, file_name, format_duration, format_size, style, App, WHITE};
use crate::library::MediaKind;

const THUMB: egui::Vec2 = egui::vec2(176.0, 99.0);

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let in_use = app.in_use(app.tab);
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            for e in app.entries.clone() {
                ui.vertical(|ui| {
                    ui.set_width(THUMB.x);
                    let (rect, response) = ui.allocate_exact_size(THUMB, Sense::click());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 5.0, Color32::from_rgb(0x1a, 0x1a, 0x1a));

                    if let Some(loaded) = app.loaded.get(&e.path) {
                        painter.image(loaded.thumb.id(), rect, cover_uv(loaded.thumb.size_vec2(), THUMB), WHITE);
                        if e.kind == MediaKind::Video {
                            // alguns vídeos não gravam a duração: aí fica só o ▶
                            let label = loaded.info.duration_s.map_or("▶".to_string(), |d| format!("▶ {}", format_duration(d)));
                            let tag_w = if loaded.info.duration_s.is_some() { 52.0 } else { 20.0 };
                            let tag = egui::Rect::from_min_size(rect.left_bottom() + egui::vec2(6.0, -22.0), egui::vec2(tag_w, 16.0));
                            painter.rect_filled(tag, 3.0, Color32::from_black_alpha(180));
                            painter.text(tag.center(), Align2::CENTER_CENTER, label, FontId::monospace(10.0), WHITE);
                        }
                    } else if let Some(why) = app.failed.get(&e.path) {
                        painter.text(rect.center(), Align2::CENTER_CENTER, "sem prévia", FontId::monospace(10.0), style::BAD);
                        response.clone().on_hover_text(why);
                    }

                    if e.path == in_use {
                        painter.circle_filled(rect.right_top() + egui::vec2(-11.0, 11.0), 5.0, style::GOOD);
                        painter.circle_stroke(rect.right_top() + egui::vec2(-11.0, 11.0), 5.0, Stroke::new(2.0, Color32::BLACK));
                    }
                    let border = if app.selected.as_ref() == Some(&e.path) { style::ACCENT } else { style::LINE };
                    painter.rect_stroke(rect, 5.0, Stroke::new(2.0, border), StrokeKind::Inside);

                    if response.clicked() {
                        app.selected = Some(e.path.clone());
                    }
                    ui.label(egui::RichText::new(file_name(&e.path)).small());
                    let kind = if e.kind == MediaKind::Video { "vídeo" } else { "imagem" };
                    ui.label(egui::RichText::new(format!("{kind} · {}", format_size(e.size))).small().color(style::DIM));
                });
            }
        });
    });
}
