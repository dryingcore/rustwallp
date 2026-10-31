//! Onde fica cada coisa, a partir de $HOME.

use std::path::PathBuf;

/// Vídeo usado pelos scripts quando não há link (mesmo valor de start-wallpaper.sh).
pub const DEFAULT_WALLPAPER: &str = "windows-xp-loading-moewalls-com.mp4";

#[derive(Debug, Clone)]
pub struct Paths {
    pub home: PathBuf,
}

impl Paths {
    pub fn from_env() -> Paths {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
        Paths { home }
    }

    pub fn wallpapers_dir(&self) -> PathBuf {
        self.home.join("Wallpapers")
    }

    pub fn link_desktop(&self) -> PathBuf {
        self.home.join(".config/wallpaper/desktop")
    }

    pub fn link_lock(&self) -> PathBuf {
        self.home.join(".config/wallpaper/lock")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.home.join(".cache/rustwallp")
    }

    pub fn start_script(&self) -> PathBuf {
        self.home.join(".config/wallpaper/start-wallpaper.sh")
    }

    pub fn lock_script(&self) -> PathBuf {
        self.home.join(".config/i3/lock.sh")
    }

    pub fn default_wallpaper(&self) -> PathBuf {
        self.wallpapers_dir().join(DEFAULT_WALLPAPER)
    }
}
