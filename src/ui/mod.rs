//! A janela: estado, trabalhos em segundo plano e montagem dos painéis.

mod dialog;
mod gallery;
mod preview;
mod side;
mod style;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::add;
use crate::apply::{self, Outcome};
use crate::barcap;
use crate::i3;
use crate::library::{self, Entry};
use crate::media::Info;
use crate::paths::Paths;
use crate::selection::{self, Selection};
use crate::test_session::{Decision, TestSession};
use crate::thumbs::Thumbs;

const BAR_REFRESH: Duration = Duration::from_secs(2);
/// Quanto o app espera, ao fechar, por um aplicar/testar em andamento (~4 s normalmente).
const EXIT_WAIT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Desktop,
    Lock,
}

/// Imagens carregadas de um wallpaper.
pub struct Loaded {
    pub thumb: TextureHandle,
    pub preview: TextureHandle,
    pub info: Info,
}

/// Algo demorado rodando numa thread (aplicar espera o vídeo subir, ~3 s).
enum Job {
    Apply(Receiver<Outcome>),
    StartTest(Receiver<Result<TestSession, String>>),
    Revert(Receiver<Outcome>),
    LockTest(Receiver<Result<(), String>>, PathBuf),
}

pub struct App {
    p: Paths,
    tab: Tab,
    entries: Vec<Entry>,
    loaded: HashMap<PathBuf, Loaded>,
    failed: HashMap<PathBuf, String>,
    thumbs: Thumbs,
    sel: Selection,
    selected: Option<PathBuf>,
    bar: Option<TextureHandle>,
    bar_error: Option<String>,
    bar_at: Option<Instant>,
    i3_ok: bool,
    job: Option<Job>,
    session: Option<TestSession>,
    ask_keep_lock: Option<PathBuf>,
    status: String,
    ctx: egui::Context,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> App {
        style::apply(&cc.egui_ctx);
        let p = Paths::from_env();
        let thumbs = Thumbs::new(p.cache_dir().join("thumbs"));
        let mut app = App {
            tab: Tab::Desktop,
            entries: Vec::new(),
            loaded: HashMap::new(),
            failed: HashMap::new(),
            thumbs,
            sel: selection::load(&p),
            selected: None,
            bar: None,
            bar_error: None,
            bar_at: None,
            i3_ok: i3::get_workspaces().is_ok(),
            job: None,
            session: None,
            ask_keep_lock: None,
            status: String::new(),
            ctx: cc.egui_ctx.clone(),
            p,
        };
        app.rescan();
        app.selected = Some(app.in_use(Tab::Desktop));
        app
    }

    /// Relê ~/Wallpapers e pede as miniaturas que faltam.
    fn rescan(&mut self) {
        match library::scan(&self.p.wallpapers_dir()) {
            Ok(entries) => {
                for e in &entries {
                    if !self.loaded.contains_key(&e.path) {
                        self.thumbs.request(e.clone());
                    }
                }
                self.status = format!("~/Wallpapers · {} arquivos", entries.len());
                self.entries = entries;
            }
            Err(e) => self.status = format!("não deu para ler ~/Wallpapers: {e}"),
        }
    }

    /// O arquivo que está valendo agora (o link, ou o vídeo padrão dos scripts).
    pub fn in_use(&self, tab: Tab) -> PathBuf {
        let desktop = self.sel.desktop.clone().unwrap_or_else(|| self.p.default_wallpaper());
        match tab {
            Tab::Desktop => desktop,
            Tab::Lock => self.sel.lock.clone().unwrap_or(desktop),
        }
    }

    pub fn busy(&self) -> bool {
        self.job.is_some() || self.session.is_some() || self.ask_keep_lock.is_some()
    }

    /// Roda `work` numa thread e acorda a interface quando termina (a janela
    /// pode estar escondida em outro workspace e não ter nada mais para acordá-la).
    fn spawn<T: Send + 'static>(&self, work: impl FnOnce() -> T + Send + 'static) -> Receiver<T> {
        let (tx, rx) = channel();
        let ctx = self.ctx.clone();
        thread::spawn(move || {
            let _ = tx.send(work());
            ctx.request_repaint();
        });
        rx
    }

    fn receive_thumbs(&mut self, ctx: &egui::Context) {
        for (path, result) in self.thumbs.poll() {
            match result.and_then(|(thumb, preview, info)| {
                Ok(Loaded {
                    thumb: load_texture(ctx, &thumb)?,
                    preview: load_texture(ctx, &preview)?,
                    info,
                })
            }) {
                Ok(loaded) => {
                    self.loaded.insert(path, loaded);
                }
                Err(why) => {
                    self.failed.insert(path, why);
                }
            }
        }
    }

    fn refresh_bar(&mut self, ctx: &egui::Context) {
        if self.bar_at.is_some_and(|t| t.elapsed() < BAR_REFRESH) {
            return;
        }
        self.bar_at = Some(Instant::now());
        match barcap::capture() {
            Ok(bar) => {
                let image = ColorImage::from_rgba_unmultiplied([bar.width, bar.height], &bar.rgba);
                match &mut self.bar {
                    Some(tex) => tex.set(image, TextureOptions::LINEAR),
                    None => self.bar = Some(ctx.load_texture("polybar", image, TextureOptions::LINEAR)),
                }
                self.bar_error = None;
            }
            Err(why) => {
                self.bar = None;
                self.bar_error = Some(why);
            }
        }
    }

    // --- ações dos botões ---------------------------------------------------

    fn apply_selected(&mut self) {
        let Some(file) = self.selected.clone() else { return };
        match self.tab {
            Tab::Desktop => {
                let p = self.p.clone();
                self.job = Some(Job::Apply(self.spawn(move || apply::apply_desktop(&p, &file))));
                self.status = "aplicando…".into();
            }
            Tab::Lock => {
                self.status = match apply::set_lock(&self.p, Some(&file)) {
                    Ok(()) => format!("tela de bloqueio: {}", file_name(&file)),
                    Err(e) => format!("não deu para gravar a escolha: {e}"),
                };
                self.sel = selection::load(&self.p);
            }
        }
    }

    fn test_selected(&mut self) {
        let Some(file) = self.selected.clone() else { return };
        let p = self.p.clone();
        match self.tab {
            Tab::Desktop => {
                self.job = Some(Job::StartTest(
                    self.spawn(move || TestSession::start(&p, &file, Instant::now())),
                ));
                self.status = "indo para um workspace vazio…".into();
            }
            Tab::Lock => {
                let shown = file.clone();
                self.job = Some(Job::LockTest(self.spawn(move || apply::test_lock(&p, &shown)), file));
                self.status = "tela bloqueada: desbloqueie com a sua senha".into();
            }
        }
    }

    /// Seletor de arquivo; a cópia vai para ~/Wallpapers e já fica selecionada.
    fn add_file(&mut self) {
        let exts = add::picker_extensions();
        let Some(src) = rfd::FileDialog::new()
            .add_filter("Wallpapers", &exts)
            .add_filter("Todos os arquivos", &["*"])
            .pick_file()
        else {
            return;
        };
        match add::import(&self.p.wallpapers_dir(), &src) {
            Ok(dest) => {
                self.status = format!("adicionado: {}", file_name(&dest));
                self.rescan();
                self.selected = Some(dest);
            }
            Err(e) => self.status = format!("não deu para adicionar: {e}"),
        }
    }

    fn set_lock_same(&mut self, same: bool) {
        let target = if same { None } else { Some(self.in_use(Tab::Lock)) };
        if let Err(e) = apply::set_lock(&self.p, target.as_deref()) {
            self.status = format!("não deu para gravar a escolha: {e}");
        }
        self.sel = selection::load(&self.p);
    }

    fn finish_test(&mut self, decision: Decision) {
        let Some(session) = self.session.take() else { return };
        match decision {
            Decision::Keep => {
                self.status = match session.keep() {
                    Outcome::Ok => "wallpaper mantido".into(),
                    Outcome::Failed(why) => format!("erro: {why}"),
                };
                self.sel = selection::load(&self.p);
            }
            Decision::Revert => {
                self.job = Some(Job::Revert(self.spawn(move || session.revert())));
                self.status = "voltando ao wallpaper anterior…".into();
            }
        }
    }

    fn poll_job(&mut self) {
        let Some(job) = self.job.take() else { return };
        let outcome_text = |o: Outcome, ok: &str| match o {
            Outcome::Ok => ok.to_string(),
            Outcome::Failed(why) => format!("erro: {why}"),
        };
        match job {
            Job::Apply(rx) => match rx.try_recv() {
                Ok(o) => {
                    self.status = outcome_text(o, "wallpaper aplicado");
                    self.sel = selection::load(&self.p);
                }
                Err(_) => self.job = Some(Job::Apply(rx)),
            },
            Job::Revert(rx) => match rx.try_recv() {
                Ok(o) => {
                    self.status = outcome_text(o, "wallpaper anterior restaurado");
                    self.sel = selection::load(&self.p);
                }
                Err(_) => self.job = Some(Job::Revert(rx)),
            },
            Job::StartTest(rx) => match rx.try_recv() {
                Ok(Ok(session)) => {
                    self.session = Some(session);
                    self.status = "testando: escolha Manter ou Voltar".into();
                }
                Ok(Err(why)) => self.status = format!("erro: {why}"),
                Err(_) => self.job = Some(Job::StartTest(rx)),
            },
            Job::LockTest(rx, file) => match rx.try_recv() {
                Ok(Ok(())) => {
                    self.ask_keep_lock = Some(file);
                    self.status.clear();
                }
                Ok(Err(why)) => self.status = format!("erro: {why}"),
                Err(_) => self.job = Some(Job::LockTest(rx, file)),
            },
        }
    }
}

impl eframe::App for App {
    /// Roda mesmo com a janela escondida (o teste leva o usuário para outro
    /// workspace, e o i3 marca a janela como escondida). Sem isto o eframe só
    /// chamaria `ui` de novo quando o usuário voltasse, e o diálogo de
    /// Manter/Voltar nunca apareceria.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_job();
        if self.busy() {
            ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.receive_thumbs(&ctx);
        self.refresh_bar(&ctx);

        if let Some(session) = &self.session {
            if let Some(decision) = dialog::test_dialog(&ctx, session) {
                self.finish_test(decision);
            }
        }
        if let Some(file) = self.ask_keep_lock.clone() {
            if let Some(keep) = dialog::keep_lock_dialog(&ctx, &file) {
                self.ask_keep_lock = None;
                if keep {
                    // grava direto no link do bloqueio: não depende da aba atual
                    self.status = match apply::set_lock(&self.p, Some(&file)) {
                        Ok(()) => format!("tela de bloqueio: {}", file_name(&file)),
                        Err(e) => format!("não deu para gravar a escolha: {e}"),
                    };
                    self.sel = selection::load(&self.p);
                }
            }
        }

        egui::Panel::top("tabs")
            .frame(style::bar_frame())
            .show(ui, |ui| self.tabs_ui(ui));
        egui::Panel::bottom("status")
            .frame(style::status_frame())
            .show(ui, |ui| {
                ui.label(egui::RichText::new(&self.status).color(style::DIM).small());
            });
        egui::Panel::bottom("gallery")
            .frame(style::gallery_frame())
            .show(ui, |ui| gallery::show(self, ui));
        egui::Panel::right("side")
            .exact_size(300.0)
            .frame(style::side_frame())
            .show(ui, |ui| side::show(self, ui));
        egui::CentralPanel::default()
            .frame(style::central_frame())
            .show(ui, |ui| preview::show(self, ui));

        if self.busy() || self.loaded.len() + self.failed.len() < self.entries.len() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else {
            ctx.request_repaint_after(BAR_REFRESH);
        }
    }

    fn on_exit(&mut self) {
        // fechar o app no meio do teste = "Voltar"
        if let Some(session) = self.session.take() {
            session.on_exit();
        }
        // um trabalho em andamento termina antes de o app sair: aplicar que
        // falha precisa restaurar o anterior, e um teste que estava começando
        // precisa ser desfeito
        match self.job.take() {
            Some(Job::Apply(rx)) | Some(Job::Revert(rx)) => {
                let _ = rx.recv_timeout(EXIT_WAIT);
            }
            Some(Job::StartTest(rx)) => {
                if let Ok(Ok(session)) = rx.recv_timeout(EXIT_WAIT) {
                    session.on_exit();
                }
            }
            // a tela está bloqueada: não há como fechar o app no meio deste
            Some(Job::LockTest(..)) | None => {}
        }
    }
}

impl App {
    fn tabs_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (tab, label) in [(Tab::Desktop, "Área de trabalho"), (Tab::Lock, "Tela de bloqueio")] {
                if ui.selectable_label(self.tab == tab, label).clicked() && self.tab != tab {
                    self.tab = tab;
                    self.selected = Some(self.in_use(tab));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(!self.busy(), egui::Button::new("+ Adicionar")).clicked() {
                    self.add_file();
                }
            });
        });
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn format_size(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0).replace('.', ",")
}

pub fn format_duration(seconds: f64) -> String {
    let s = seconds.round() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

fn load_texture(ctx: &egui::Context, path: &Path) -> Result<TextureHandle, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let image = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Ok(ctx.load_texture(path.to_string_lossy(), image, TextureOptions::LINEAR))
}

/// Recorte "cover": preenche `target` sem distorcer, cortando as sobras.
pub fn cover_uv(image_size: egui::Vec2, target: egui::Vec2) -> egui::Rect {
    let (ia, ta) = (image_size.x / image_size.y, target.x / target.y);
    let (w, h) = if ia > ta { (ta / ia, 1.0) } else { (1.0, ia / ta) };
    egui::Rect::from_center_size(egui::pos2(0.5, 0.5), egui::vec2(w, h))
}

pub const WHITE: Color32 = Color32::WHITE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(format_size(2_684_195), "2,7 MB");
        assert_eq!(format_duration(16.789), "0:17");
        assert_eq!(format_duration(75.0), "1:15");
    }

    #[test]
    fn cover_crops_wider_image() {
        let uv = cover_uv(egui::vec2(200.0, 100.0), egui::vec2(100.0, 100.0));
        assert!((uv.width() - 0.5).abs() < 1e-6 && (uv.height() - 1.0).abs() < 1e-6);
    }
}
