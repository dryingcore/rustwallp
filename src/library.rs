//! Os wallpapers de ~/Wallpapers: o que é vídeo, o que é imagem.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Video,
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: MediaKind,
    pub size: u64,
    pub mtime: SystemTime,
}

/// Extensões aceitas; as mesmas que start-wallpaper.sh trata como imagem.
pub const VIDEO_EXTS: &[&str] = &["mp4", "mkv", "webm", "mov"];
pub const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp"];

pub fn classify(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if VIDEO_EXTS.contains(&ext.as_str()) {
        Some(MediaKind::Video)
    } else if IMAGE_EXTS.contains(&ext.as_str()) {
        Some(MediaKind::Image)
    } else {
        None
    }
}

/// Arquivos de vídeo e imagem de `dir`, ordenados por nome; ignora subpastas.
pub fn scan(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in fs::read_dir(dir)? {
        let path = item?.path();
        let Some(kind) = classify(&path) else { continue };
        let meta = fs::metadata(&path)?;
        if !meta.is_file() {
            continue;
        }
        entries.push(Entry {
            path,
            kind,
            size: meta.len(),
            mtime: meta.modified()?,
        });
    }
    entries.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_video_any_case() {
        assert_eq!(classify(Path::new("a.MP4")), Some(MediaKind::Video));
        assert_eq!(classify(Path::new("b.webm")), Some(MediaKind::Video));
    }

    #[test]
    fn classify_image_any_case() {
        assert_eq!(classify(Path::new("FOTO.JPG")), Some(MediaKind::Image));
        assert_eq!(classify(Path::new("x.webp")), Some(MediaKind::Image));
    }

    #[test]
    fn classify_rejects_other() {
        assert_eq!(classify(Path::new("notes.txt")), None);
        assert_eq!(classify(Path::new("semext")), None);
    }

    #[test]
    fn scan_sorted_and_filtered() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["b.mp4", "a b (1).png", "c.txt"] {
            fs::write(dir.path().join(name), b"x").unwrap();
        }
        fs::create_dir(dir.path().join("d.mp4")).unwrap();

        let names: Vec<_> = scan(dir.path())
            .unwrap()
            .into_iter()
            .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a b (1).png", "b.mp4"]);
    }
}
