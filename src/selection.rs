//! A escolha do usuário mora em dois links simbólicos, lidos pelos scripts:
//! ~/.config/wallpaper/desktop e ~/.config/wallpaper/lock.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::paths::Paths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub desktop: Option<PathBuf>,
    pub lock: Option<PathBuf>,
    /// Sem link `lock` (ou apontando para o mesmo arquivo): o lock.sh segue o desktop.
    pub lock_same_as_desktop: bool,
}

/// Alvo do link, só se `link` for um link e o alvo existir.
pub fn read_choice(link: &Path) -> Option<PathBuf> {
    if !fs::symlink_metadata(link).ok()?.file_type().is_symlink() {
        return None;
    }
    let target = fs::canonicalize(link).ok()?;
    target.is_file().then_some(target)
}

/// Aponta `link` para `target` sem nunca deixar de existir um link no meio.
pub fn set_choice(link: &Path, target: &Path) -> io::Result<()> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp = link.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let _ = fs::remove_file(&tmp);
    std::os::unix::fs::symlink(target, &tmp)?;
    fs::rename(&tmp, link)
}

pub fn load(p: &Paths) -> Selection {
    let desktop = read_choice(&p.link_desktop());
    let lock = read_choice(&p.link_lock());
    // mesma ordem do lock.sh: sem link lock, o bloqueio segue o desktop
    let lock_same_as_desktop = lock.is_none();
    Selection { desktop, lock, lock_same_as_desktop }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"x").unwrap();
        path
    }

    #[test]
    fn read_missing_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_choice(&dir.path().join("desktop")), None);
    }

    #[test]
    fn read_broken_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let target = touch(dir.path(), "gone.mp4");
        let link = dir.path().join("desktop");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        fs::remove_file(&target).unwrap();
        assert_eq!(read_choice(&link), None);
    }

    #[test]
    fn set_then_read_with_spaces() {
        let dir = tempfile::tempdir().unwrap();
        let target = touch(dir.path(), "meu clip (1).mp4");
        let link = dir.path().join("desktop");
        set_choice(&link, &target).unwrap();
        assert_eq!(read_choice(&link), Some(target));
    }

    #[test]
    fn set_replaces_existing_link() {
        let dir = tempfile::tempdir().unwrap();
        let first = touch(dir.path(), "a.mp4");
        let second = touch(dir.path(), "b.png");
        let link = dir.path().join("desktop");
        set_choice(&link, &first).unwrap();
        set_choice(&link, &second).unwrap();
        assert_eq!(read_choice(&link), Some(second));
        assert!(!dir.path().join("desktop.tmp").exists());
    }

    #[test]
    fn set_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let target = touch(dir.path(), "a.mp4");
        let link = dir.path().join("novo/wallpaper/desktop");
        set_choice(&link, &target).unwrap();
        assert_eq!(read_choice(&link), Some(target));
    }

    #[test]
    fn load_independent_lock_link_is_not_same_even_if_equal() {
        // com link lock, mudar o desktop não chega ao bloqueio (lock.sh lê o
        // lock primeiro): não é "igual", mesmo apontando para o mesmo arquivo
        let dir = tempfile::tempdir().unwrap();
        let p = Paths { home: dir.path().to_path_buf() };
        let target = touch(dir.path(), "a.mp4");
        set_choice(&p.link_desktop(), &target).unwrap();
        set_choice(&p.link_lock(), &target).unwrap();
        assert!(!load(&p).lock_same_as_desktop);
    }

    #[test]
    fn load_same_as_desktop_when_no_lock_link() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths { home: dir.path().to_path_buf() };
        let target = touch(dir.path(), "a.mp4");
        set_choice(&p.link_desktop(), &target).unwrap();

        let sel = load(&p);
        assert_eq!(sel.desktop, Some(target));
        assert_eq!(sel.lock, None);
        assert!(sel.lock_same_as_desktop);
    }
}
