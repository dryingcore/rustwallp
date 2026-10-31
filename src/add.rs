//! "Adicionar": copia um arquivo para ~/Wallpapers sem nunca sobrescrever.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::library::{classify, IMAGE_EXTS, VIDEO_EXTS};

/// `nome.ext` se estiver livre; senão `nome-1.ext`, `nome-2.ext`…
pub fn free_name(dir: &Path, file_name: &OsStr) -> PathBuf {
    let first = dir.join(file_name);
    if !first.exists() {
        return first;
    }
    let name = Path::new(file_name);
    let stem = name.file_stem().unwrap_or(file_name).to_string_lossy();
    let ext = name.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    (1..)
        .map(|n| dir.join(format!("{stem}-{n}{ext}")))
        .find(|candidate| !candidate.exists())
        .expect("sempre há um nome livre")
}

pub fn import(dir: &Path, src: &Path) -> io::Result<PathBuf> {
    if classify(src).is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "não é vídeo nem imagem aceito"));
    }
    let file_name = src.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "sem nome de arquivo"))?;
    fs::create_dir_all(dir)?;
    let dest = free_name(dir, file_name);
    fs::copy(src, &dest)?;
    Ok(dest)
}

/// Extensões para o filtro do seletor de arquivo. O filtro do GTK3 diferencia
/// maiúsculas, então vão as duas formas (FOTO.JPG, clip.MP4).
pub fn picker_extensions() -> Vec<String> {
    VIDEO_EXTS
        .iter()
        .chain(IMAGE_EXTS)
        .flat_map(|e| [e.to_string(), e.to_uppercase()])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_extensions_include_uppercase() {
        let exts = picker_extensions();
        for e in ["mp4", "MP4", "jpg", "JPG", "webp", "WEBP", "mov", "MOV"] {
            assert!(exts.iter().any(|x| x == e), "falta {e}");
        }
    }

    #[test]
    fn free_name_unused() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(free_name(dir.path(), OsStr::new("a.mp4")), dir.path().join("a.mp4"));
    }

    #[test]
    fn free_name_taken_twice() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.mp4"), b"x").unwrap();
        fs::write(dir.path().join("a-1.mp4"), b"x").unwrap();
        assert_eq!(free_name(dir.path(), OsStr::new("a.mp4")), dir.path().join("a-2.mp4"));
    }

    #[test]
    fn import_rejects_txt() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("notes.txt");
        fs::write(&src, b"x").unwrap();
        let err = import(&dir.path().join("w"), &src).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn import_copies_keeping_original() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("foto.JPG");
        fs::write(&src, b"img").unwrap();
        let walls = dir.path().join("Wallpapers");
        fs::create_dir(&walls).unwrap();
        fs::write(walls.join("foto.JPG"), b"old").unwrap();

        let copied = import(&walls, &src).unwrap();
        assert_eq!(copied, walls.join("foto-1.JPG"));
        assert_eq!(fs::read(&copied).unwrap(), b"img");
        assert_eq!(fs::read(walls.join("foto.JPG")).unwrap(), b"old");
        assert!(src.exists());
    }
}
