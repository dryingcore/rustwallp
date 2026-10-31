//! Prévia 16:9: quadro do wallpaper + barra real (desktop) ou prompt do xsecurelock (bloqueio).

use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind};

use super::{cover_uv, file_name, format_duration, format_size, style, App, Tab, WHITE};
use crate::library::MediaKind;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let width = ui.available_width();
    let size = egui::vec2(width, (width * 9.0 / 16.0).min(ui.available_height() - 50.0));
    let size = egui::vec2(size.y * 16.0 / 9.0, size.y);
    // a altura limita a prévia; centraliza na largura que sobra
    let (row, _) = ui.allocate_exact_size(egui::vec2(width, size.y), Sense::hover());
    let rect = egui::Rect::from_center_size(row.center(), size);
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 6.0, Color32::BLACK);
    let selected = app.selected.clone();
    if let Some(loaded) = selected.as_ref().and_then(|p| app.loaded.get(p)) {
        let uv = cover_uv(loaded.preview.size_vec2(), size);
        painter.image(loaded.preview.id(), rect, uv, WHITE);
    } else if selected.is_some() {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "carregando…",
            FontId::monospace(13.0),
            style::DIM,
        );
    }

    match app.tab {
        Tab::Desktop => draw_bar(app, &painter, rect),
        Tab::Lock => draw_lock_prompt(&painter, rect),
    }
    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, style::LINE), StrokeKind::Inside);

    ui.add_space(4.0);
    ui.horizontal(|ui| info_line(app, ui));
}

/// A barra de verdade, na escala da prévia (1920 px da tela → largura da prévia).
fn draw_bar(app: &App, painter: &egui::Painter, rect: egui::Rect) {
    match &app.bar {
        Some(bar) => {
            let scale = rect.width() / bar.size_vec2().x;
            let bar_rect = egui::Rect::from_min_size(rect.min, bar.size_vec2() * scale);
            painter.image(
                bar.id(),
                bar_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                WHITE,
            );
        }
        None => {
            let msg = app.bar_error.as_deref().unwrap_or("barra não encontrada");
            painter.text(
                rect.left_top() + egui::vec2(8.0, 6.0),
                Align2::LEFT_TOP,
                msg,
                FontId::monospace(10.0),
                style::DIM,
            );
        }
    }
}

/// Imitação do prompt do lock.sh: preto, mono, usuário, data e asteriscos.
fn draw_lock_prompt(painter: &egui::Painter, rect: egui::Rect) {
    let user = std::env::var("USER").unwrap_or_default();
    let now = std::process::Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let box_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width() * 0.32, rect.height() * 0.2));
    painter.rect_filled(box_rect, 0.0, Color32::BLACK);
    let font = FontId::monospace(rect.height() / 40.0);
    let line = rect.height() / 28.0;
    let c = box_rect.center();
    for (i, text) in [user.as_str(), now.as_str(), "Password: ********"].iter().enumerate() {
        let y = c.y + (i as f32 - 1.0) * line;
        painter.text(
            egui::pos2(c.x, y),
            Align2::CENTER_CENTER,
            *text,
            font.clone(),
            style::TEXT,
        );
    }
}

fn info_line(app: &mut App, ui: &mut egui::Ui) {
    let Some(path) = app.selected.clone() else { return };
    ui.label(egui::RichText::new(file_name(&path)).color(egui::Color32::WHITE));
    if let Some(e) = app.entries.iter().find(|e| e.path == path) {
        let mut parts = vec![
            if e.kind == MediaKind::Video { "vídeo" } else { "imagem" }.to_string(),
            format_size(e.size),
        ];
        if let Some(info) = app.loaded.get(&path).map(|l| &l.info) {
            if let Some(d) = info.duration_s {
                parts.push(format_duration(d));
            }
            parts.push(format!("{}×{}", info.width, info.height));
        }
        ui.label(egui::RichText::new(parts.join(" · ")).color(style::DIM));
    } else {
        ui.label(egui::RichText::new("arquivo não está em ~/Wallpapers").color(style::BAD));
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let exists = path.is_file();
        let apply = egui::Button::new(egui::RichText::new("✓ Aplicar").color(egui::Color32::WHITE))
            .fill(egui::Color32::from_rgb(0x1d, 0x5c, 0x5c))
            .stroke(Stroke::new(1.0, style::ACCENT));
        if ui.add_enabled(exists && !app.busy(), apply).clicked() {
            app.apply_selected();
        }
        let can_test = exists && !app.busy() && (app.tab == Tab::Lock || app.i3_ok);
        let test = ui.add_enabled(can_test, egui::Button::new("▶ Testar na tela"));
        let test = if app.tab == Tab::Desktop && !app.i3_ok {
            test.on_disabled_hover_text("o i3 não respondeu: o teste precisa dele para ir a um workspace vazio")
        } else {
            test
        };
        if test.clicked() {
            app.test_selected();
        }
    });
}
