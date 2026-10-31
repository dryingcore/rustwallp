//! Tema escuro e mono, com as cores da polybar.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Stroke, TextStyle};

pub const BG: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const PANEL: Color32 = Color32::from_rgb(0x16, 0x16, 0x16);
pub const LINE: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
pub const TEXT: Color32 = Color32::from_rgb(0xdd, 0xdd, 0xdd);
pub const DIM: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x8a);
pub const ACCENT: Color32 = Color32::from_rgb(0x2f, 0xc6, 0xc6);
pub const GOOD: Color32 = Color32::from_rgb(0x00, 0xcc, 0x00);
pub const BAD: Color32 = Color32::from_rgb(0xff, 0x44, 0x44);

pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Small, FontId::new(11.0, FontFamily::Monospace)),
            (TextStyle::Body, FontId::new(13.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(13.0, FontFamily::Monospace)),
            (TextStyle::Heading, FontId::new(15.0, FontFamily::Monospace)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        ]
        .into();
        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = PANEL;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = Color32::from_rgb(0x1d, 0x5c, 0x5c);
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    });
}

pub fn bar_frame() -> Frame {
    Frame::new()
        .fill(BG)
        .inner_margin(Margin::symmetric(16, 10))
        .stroke(Stroke::new(1.0, LINE))
}

pub fn central_frame() -> Frame {
    Frame::new().fill(BG).inner_margin(Margin::same(16))
}

pub fn side_frame() -> Frame {
    Frame::new()
        .fill(PANEL)
        .inner_margin(Margin::same(14))
        .outer_margin(Margin {
            left: 0,
            right: 16,
            top: 16,
            bottom: 16,
        })
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(CornerRadius::same(6))
}

pub fn gallery_frame() -> Frame {
    Frame::new()
        .fill(BG)
        .inner_margin(Margin::symmetric(16, 14))
        .stroke(Stroke::new(1.0, LINE))
}

pub fn status_frame() -> Frame {
    Frame::new().fill(BG).inner_margin(Margin::symmetric(16, 6))
}

/// Título de seção do painel lateral ("EM USO AGORA").
pub fn section(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text.to_uppercase()).color(DIM).small());
}
