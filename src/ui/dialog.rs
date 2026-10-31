//! Janelas pequenas: Manter/Voltar do teste e a pergunta depois do teste de bloqueio.

use std::path::Path;
use std::time::Instant;

use eframe::egui::{self, ViewportBuilder, ViewportId, X11WindowType};

use super::{file_name, style};
use crate::test_session::{Decision, TestSession};

/// Janela de diálogo: o i3 já deixa flutuante, sem regra nova no config.
fn dialog_builder(title: &str) -> ViewportBuilder {
    ViewportBuilder::default()
        .with_title(title)
        .with_inner_size([340.0, 120.0])
        .with_resizable(false)
        .with_always_on_top()
        .with_window_type(X11WindowType::Dialog)
}

/// Manter/Voltar com contagem; `Some` quando houve decisão (ou o tempo acabou).
pub fn test_dialog(ctx: &egui::Context, session: &TestSession) -> Option<Decision> {
    let now = Instant::now();
    if let Some(decision) = session.tick(now) {
        return Some(decision);
    }
    let left = session.remaining(now).as_secs() + 1;
    ctx.request_repaint_after(std::time::Duration::from_millis(200));
    ctx.show_viewport_immediate(
        ViewportId::from_hash_of("rustwallp-teste"),
        dialog_builder("rustwallp — teste"),
        |ui, _| {
            let mut decision = None;
            egui::CentralPanel::default()
                .frame(style::central_frame())
                .show(ui, |ui| {
                    ui.label("Gostou deste wallpaper?");
                    ui.label(egui::RichText::new(format!("Voltando ao anterior em {left} s")).color(style::DIM));
                    ui.horizontal(|ui| {
                        if ui.button("✓ Manter").clicked() {
                            decision = Some(Decision::Keep);
                        }
                        if ui.button("↶ Voltar").clicked() {
                            decision = Some(Decision::Revert);
                        }
                    });
                });
            if ui.ctx().input(|i| i.viewport().close_requested()) {
                decision = Some(Decision::Revert);
            }
            decision
        },
    )
}

/// Depois do teste da tela de bloqueio: `Some(true)` = manter.
pub fn keep_lock_dialog(ctx: &egui::Context, file: &Path) -> Option<bool> {
    ctx.show_viewport_immediate(
        ViewportId::from_hash_of("rustwallp-lock"),
        dialog_builder("rustwallp — tela de bloqueio"),
        |ui, _| {
            let mut answer = None;
            egui::CentralPanel::default()
                .frame(style::central_frame())
                .show(ui, |ui| {
                    ui.label("Manter este como tela de bloqueio?");
                    ui.label(egui::RichText::new(file_name(file)).color(style::DIM));
                    ui.horizontal(|ui| {
                        if ui.button("✓ Manter").clicked() {
                            answer = Some(true);
                        }
                        if ui.button("Não").clicked() {
                            answer = Some(false);
                        }
                    });
                });
            if ui.ctx().input(|i| i.viewport().close_requested()) {
                answer = Some(false);
            }
            answer
        },
    )
}
