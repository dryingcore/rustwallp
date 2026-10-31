//! Aplicar um wallpaper = trocar o link e rodar o script do dotfiles.
//! Se não subir, volta o anterior: nunca deixa a tela sem wallpaper.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::library::{classify, MediaKind};
use crate::paths::Paths;
use crate::selection::{read_choice, set_choice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Failed(String),
}

/// Regra de falha: arquivo sumiu, script com erro ou (vídeo) nenhum xwinwrap rodando.
pub fn judge(file_exists: bool, script_ok: bool, kind: MediaKind, xwinwrap_running: bool) -> Outcome {
    if !file_exists {
        Outcome::Failed("o arquivo não existe mais".into())
    } else if !script_ok {
        Outcome::Failed("o start-wallpaper.sh terminou com erro".into())
    } else if kind == MediaKind::Video && !xwinwrap_running {
        Outcome::Failed("o vídeo não começou a tocar".into())
    } else {
        Outcome::Ok
    }
}

/// Roda o start-wallpaper.sh numa sessão própria (setsid -w) e sem a nossa
/// entrada/saída, para o xwinwrap não depender do app: fechar o app não
/// derruba o wallpaper, e ninguém fica esperando a saída do app terminar.
/// `show`: mostra este arquivo só desta vez (RUSTWALLP_DESKTOP_FILE), sem tocar no link.
fn run_start_script(p: &Paths, show: Option<&Path>) -> bool {
    let mut cmd = Command::new("setsid");
    cmd.arg("-w").arg(p.start_script());
    match show {
        Some(file) => cmd.env("RUSTWALLP_DESKTOP_FILE", file),
        None => cmd.env_remove("RUSTWALLP_DESKTOP_FILE"),
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn xwinwrap_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "xwinwrap"])
        .status()
        .is_ok_and(|s| s.success())
}

/// Roda o script e julga o resultado para `file` (o arquivo que deve estar na tela).
fn run_and_judge(p: &Paths, file: &Path, show: Option<&Path>) -> Outcome {
    let kind = classify(file).unwrap_or(MediaKind::Video);
    let script_ok = run_start_script(p, show);
    if kind == MediaKind::Video {
        thread::sleep(Duration::from_secs(3));
    }
    judge(file.is_file(), script_ok, kind, xwinwrap_running())
}

pub fn apply_desktop(p: &Paths, file: &Path) -> Outcome {
    if !file.is_file() {
        return judge(false, false, MediaKind::Image, false);
    }
    let prev = read_choice(&p.link_desktop());
    if let Err(e) = set_choice(&p.link_desktop(), file) {
        return Outcome::Failed(format!("não deu para gravar a escolha: {e}"));
    }
    match run_and_judge(p, file, None) {
        Outcome::Ok => Outcome::Ok,
        Outcome::Failed(why) => {
            restore_desktop(p, prev);
            Outcome::Failed(format!("{why}; o wallpaper anterior foi restaurado"))
        }
    }
}

/// "Testar na tela": mostra `file` sem gravar nada. Se o app travar no meio,
/// a escolha salva (o link) continua intacta.
pub fn preview_desktop(p: &Paths, file: &Path) -> Outcome {
    if !file.is_file() {
        return judge(false, false, MediaKind::Image, false);
    }
    run_and_judge(p, file, Some(file))
}

/// Volta a mostrar o que está salvo (o link, ou o vídeo padrão dos scripts).
pub fn show_saved_desktop(p: &Paths) -> Outcome {
    let shown = read_choice(&p.link_desktop()).unwrap_or_else(|| p.default_wallpaper());
    run_and_judge(p, &shown, None)
}

/// Volta o link `desktop` ao estado anterior (`None` = sem link, vídeo padrão) e roda o script.
pub fn restore_desktop(p: &Paths, prev: Option<PathBuf>) -> Outcome {
    let result = match &prev {
        Some(file) => set_choice(&p.link_desktop(), file),
        None => remove_link(&p.link_desktop()),
    };
    if let Err(e) = result {
        return Outcome::Failed(format!("não deu para restaurar a escolha: {e}"));
    }
    let shown = prev.unwrap_or_else(|| p.default_wallpaper());
    run_and_judge(p, &shown, None)
}

fn remove_link(link: &Path) -> io::Result<()> {
    match fs::remove_file(link) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// `None` = igual à área de trabalho: sem link `lock`, o lock.sh segue o `desktop`.
pub fn set_lock(p: &Paths, file: Option<&Path>) -> io::Result<()> {
    match file {
        Some(file) => set_choice(&p.link_lock(), file),
        None => remove_link(&p.link_lock()),
    }
}

/// Bloqueia a tela com `file` sem salvar nada; volta quando o usuário desbloqueia.
pub fn test_lock(p: &Paths, file: &Path) -> Result<(), String> {
    let status = Command::new(p.lock_script())
        .env("RUSTWALLP_LOCK_FILE", file)
        .status()
        .map_err(|e| format!("lock.sh não rodou: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("a tela de bloqueio terminou com erro".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_fails() {
        assert!(matches!(
            judge(false, true, MediaKind::Image, false),
            Outcome::Failed(_)
        ));
    }

    #[test]
    fn script_error_fails() {
        assert!(matches!(
            judge(true, false, MediaKind::Image, false),
            Outcome::Failed(_)
        ));
    }

    #[test]
    fn video_without_xwinwrap_fails() {
        assert!(matches!(judge(true, true, MediaKind::Video, false), Outcome::Failed(_)));
    }

    #[test]
    fn image_ok_without_xwinwrap() {
        assert_eq!(judge(true, true, MediaKind::Image, false), Outcome::Ok);
    }

    #[test]
    fn video_ok_with_xwinwrap() {
        assert_eq!(judge(true, true, MediaKind::Video, true), Outcome::Ok);
    }

    #[test]
    fn set_lock_none_removes_link() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths {
            home: dir.path().to_path_buf(),
        };
        let file = dir.path().join("a.png");
        fs::write(&file, b"x").unwrap();
        set_lock(&p, Some(&file)).unwrap();
        assert!(read_choice(&p.link_lock()).is_some());
        set_lock(&p, None).unwrap();
        assert!(fs::symlink_metadata(p.link_lock()).is_err());
        set_lock(&p, None).unwrap(); // sem link: não é erro
    }

    #[test]
    fn script_children_do_not_inherit_our_stdio() {
        // o xwinwrap fica rodando depois do script; se herdar nossa saída,
        // prende quem lê dela (e amarra o wallpaper ao app)
        let dir = tempfile::tempdir().unwrap();
        let p = Paths {
            home: dir.path().to_path_buf(),
        };
        let script = p.start_script();
        fs::create_dir_all(script.parent().unwrap()).unwrap();
        let pidfile = dir.path().join("pid");
        fs::write(
            &script,
            format!("#!/bin/sh\nsleep 30 &\necho $! > '{}'\n", pidfile.display()),
        )
        .unwrap();
        fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();

        assert!(run_start_script(&p, None));
        let pid = fs::read_to_string(&pidfile).unwrap().trim().to_string();
        let stdout = fs::read_link(format!("/proc/{pid}/fd/1")).unwrap();
        Command::new("kill").arg(&pid).status().unwrap();
        assert_eq!(stdout, PathBuf::from("/dev/null"));
    }

    /// $HOME falsa com um start-wallpaper.sh que só anota a variável recebida.
    fn fake_home() -> (tempfile::TempDir, Paths, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths {
            home: dir.path().to_path_buf(),
        };
        let script = p.start_script();
        fs::create_dir_all(script.parent().unwrap()).unwrap();
        let log = dir.path().join("env.log");
        fs::write(
            &script,
            format!("#!/bin/sh\necho \"[$RUSTWALLP_DESKTOP_FILE]\" > '{}'\n", log.display()),
        )
        .unwrap();
        fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        (dir, p, log)
    }

    #[test]
    fn preview_shows_file_via_env_without_touching_link() {
        let (dir, p, log) = fake_home();
        let saved = dir.path().join("salvo.png");
        let test = dir.path().join("teste.png");
        fs::write(&saved, b"x").unwrap();
        fs::write(&test, b"x").unwrap();
        set_choice(&p.link_desktop(), &saved).unwrap();

        assert_eq!(preview_desktop(&p, &test), Outcome::Ok);
        assert_eq!(
            fs::read_to_string(&log).unwrap().trim(),
            format!("[{}]", test.display())
        );
        assert_eq!(read_choice(&p.link_desktop()), Some(fs::canonicalize(&saved).unwrap()));
    }

    #[test]
    fn show_saved_runs_without_override() {
        let (dir, p, log) = fake_home();
        let saved = dir.path().join("salvo.png");
        fs::write(&saved, b"x").unwrap();
        set_choice(&p.link_desktop(), &saved).unwrap();

        assert_eq!(show_saved_desktop(&p), Outcome::Ok);
        assert_eq!(fs::read_to_string(&log).unwrap().trim(), "[]");
    }

    #[test]
    #[ignore = "troca o wallpaper de verdade (e volta)"]
    fn apply_real_roundtrip() {
        let p = Paths::from_env();
        let prev = read_choice(&p.link_desktop());
        let other = p.wallpapers_dir().join("ascii-rest-earthrise.mp4");

        assert_eq!(apply_desktop(&p, &other), Outcome::Ok);
        let running = Command::new("pgrep").args(["-af", "xwinwrap"]).output().unwrap();
        assert!(String::from_utf8_lossy(&running.stdout).contains("ascii-rest-earthrise.mp4"));

        assert_eq!(restore_desktop(&p, prev), Outcome::Ok);
    }
}
