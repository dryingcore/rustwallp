//! "Testar na tela": vai para um workspace vazio, mostra o wallpaper e espera
//! 15 s por Manter/Voltar. Durante o teste o link não muda (o script recebe o
//! arquivo por variável): só "Manter" grava. Voltar, tempo esgotado ou app
//! fechado = volta a mostrar o que está salvo.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::apply::{self, Outcome};
use crate::i3;
use crate::paths::Paths;
use crate::selection::set_choice;

pub const TEST_DURATION: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Keep,
    Revert,
}

#[derive(Debug, Clone, Copy)]
pub struct Countdown {
    pub started: Instant,
    pub total: Duration,
}

impl Countdown {
    pub fn remaining(&self, now: Instant) -> Duration {
        self.total.saturating_sub(now.saturating_duration_since(self.started))
    }

    /// `Some(Revert)` quando o tempo acaba; antes disso, nada.
    pub fn tick(&self, now: Instant) -> Option<Decision> {
        self.remaining(now).is_zero().then_some(Decision::Revert)
    }
}

pub struct TestSession {
    p: Paths,
    file: PathBuf,
    origin: String,
    pub countdown: Countdown,
}

impl TestSession {
    pub fn start(p: &Paths, file: &Path, now: Instant) -> Result<TestSession, String> {
        let workspaces = i3::get_workspaces()?;
        let origin = workspaces
            .iter()
            .find(|w| w.focused)
            .map(|w| w.name.clone())
            .ok_or("o i3 não informou o workspace atual")?;
        i3::goto(&i3::pick_empty(&workspaces))?;
        if let Outcome::Failed(why) = apply::preview_desktop(p, file) {
            apply::show_saved_desktop(p);
            let _ = i3::goto(&origin);
            return Err(format!("{why}; o wallpaper salvo foi mostrado de novo"));
        }
        Ok(TestSession {
            p: p.clone(),
            file: file.to_path_buf(),
            origin,
            countdown: Countdown { started: now, total: TEST_DURATION },
        })
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        self.countdown.remaining(now)
    }

    pub fn tick(&self, now: Instant) -> Option<Decision> {
        self.countdown.tick(now)
    }

    /// Grava a escolha; a tela já mostra o arquivo, então não roda o script de novo.
    pub fn keep(self) -> Outcome {
        let outcome = match set_choice(&self.p.link_desktop(), &self.file) {
            Ok(()) => Outcome::Ok,
            Err(e) => Outcome::Failed(format!("não deu para gravar a escolha: {e}")),
        };
        let _ = i3::goto(&self.origin);
        outcome
    }

    pub fn revert(self) -> Outcome {
        let outcome = apply::show_saved_desktop(&self.p);
        let _ = i3::goto(&self.origin);
        outcome
    }

    /// Fechar o app no meio do teste é o mesmo que "Voltar".
    pub fn on_exit(self) -> Outcome {
        self.revert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn countdown(t0: Instant) -> Countdown {
        Countdown { started: t0, total: TEST_DURATION }
    }

    #[test]
    fn remaining_counts_down() {
        let t0 = Instant::now();
        assert_eq!(countdown(t0).remaining(t0 + Duration::from_secs(5)), Duration::from_secs(10));
    }

    #[test]
    fn expires_to_revert() {
        let t0 = Instant::now();
        assert_eq!(countdown(t0).tick(t0 + Duration::from_secs(15)), Some(Decision::Revert));
        assert_eq!(countdown(t0).remaining(t0 + Duration::from_secs(20)), Duration::ZERO);
    }

    #[test]
    fn not_expired_is_none() {
        let t0 = Instant::now();
        assert_eq!(countdown(t0).tick(t0 + Duration::from_secs(14)), None);
    }
}
