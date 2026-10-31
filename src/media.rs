//! Duração/resolução (ffprobe) e miniaturas (ffmpeg para vídeo, crate image para imagem).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use crate::library::{Entry, MediaKind};

pub const THUMB_WIDTH: u32 = 480;
/// Quadro maior para a prévia, que ocupa ~900 px na tela.
pub const PREVIEW_WIDTH: u32 = 1280;

#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub duration_s: Option<f64>,
    pub width: u32,
    pub height: u32,
}

/// Lê a saída de
/// `ffprobe -v error -select_streams v:0 -show_entries stream=width,height:format=duration -of json`.
pub fn parse_ffprobe(json: &str) -> Option<Info> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let stream = v["streams"].get(0)?;
    Some(Info {
        duration_s: v["format"]["duration"].as_str().and_then(|d| d.parse().ok()),
        width: stream["width"].as_u64()? as u32,
        height: stream["height"].as_u64()? as u32,
    })
}

pub fn probe(path: &Path, kind: MediaKind) -> Option<Info> {
    match kind {
        MediaKind::Image => {
            let (width, height) = image::image_dimensions(path).ok()?;
            Some(Info { duration_s: None, width, height })
        }
        MediaKind::Video => {
            let out = Command::new("ffprobe")
                .args(["-v", "error", "-select_streams", "v:0"])
                .args(["-show_entries", "stream=width,height:format=duration", "-of", "json"])
                .arg(path)
                .output()
                .ok()?;
            parse_ffprobe(&String::from_utf8_lossy(&out.stdout))
        }
    }
}

fn cache_key(e: &Entry) -> u64 {
    let mut h = DefaultHasher::new();
    e.path.hash(&mut h);
    e.size.hash(&mut h);
    e.mtime.duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().hash(&mut h);
    h.finish()
}

/// Caminho da miniatura no cache; muda quando o arquivo muda (tamanho ou mtime).
pub fn thumb_path(cache: &Path, e: &Entry) -> PathBuf {
    cache.join(format!("{:016x}.jpg", cache_key(e)))
}

/// Caminho do quadro grande da prévia (mesma chave da miniatura).
pub fn preview_path(cache: &Path, e: &Entry) -> PathBuf {
    cache.join(format!("{:016x}-preview.jpg", cache_key(e)))
}

/// Grava um quadro de `width` px de largura (vídeo: em min(3 s, duração/2)).
pub fn make_thumb(e: &Entry, info: &Info, out: &Path, width: u32) -> Result<(), String> {
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    match e.kind {
        MediaKind::Image => {
            let img = image::open(&e.path).map_err(|err| err.to_string())?;
            let height = (width as u64 * img.height() as u64 / img.width().max(1) as u64) as u32;
            img.resize_exact(width, height.max(1), image::imageops::FilterType::Triangle)
                .to_rgb8()
                .save(out)
                .map_err(|err| err.to_string())
        }
        MediaKind::Video => {
            let at = info.duration_s.map_or(0.0, |d| (d / 2.0).min(3.0));
            let ok = Command::new("ffmpeg")
                .args(["-v", "error", "-y", "-ss", &format!("{at:.2}"), "-i"])
                .arg(&e.path)
                .args(["-frames:v", "1", "-vf", &format!("scale={width}:-2")])
                .arg(out)
                .status()
                .map_err(|err| format!("ffmpeg não rodou: {err}"))?
                .success();
            if ok { Ok(()) } else { Err(format!("ffmpeg falhou em {}", e.path.display())) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn entry(path: &str, mtime_s: u64) -> Entry {
        Entry {
            path: PathBuf::from(path),
            kind: MediaKind::Video,
            size: 10,
            mtime: SystemTime::UNIX_EPOCH + Duration::from_secs(mtime_s),
        }
    }

    #[test]
    fn parse_ffprobe_video() {
        let json = r#"{"streams":[{"width":1920,"height":1080}],"format":{"duration":"16.789333"}}"#;
        let info = parse_ffprobe(json).unwrap();
        assert_eq!((info.width, info.height), (1920, 1080));
        assert!((info.duration_s.unwrap() - 16.789333).abs() < 1e-6);
    }

    #[test]
    fn parse_ffprobe_no_stream_is_none() {
        assert_eq!(parse_ffprobe(r#"{"streams":[],"format":{}}"#), None);
    }

    #[test]
    fn thumb_key_changes_with_mtime() {
        let cache = Path::new("/c");
        assert_ne!(thumb_path(cache, &entry("/w/a.mp4", 1)), thumb_path(cache, &entry("/w/a.mp4", 2)));
        assert_eq!(thumb_path(cache, &entry("/w/a.mp4", 1)), thumb_path(cache, &entry("/w/a.mp4", 1)));
        assert_ne!(thumb_path(cache, &entry("/w/a.mp4", 1)), preview_path(cache, &entry("/w/a.mp4", 1)));
    }

    #[test]
    fn image_thumb_is_480_wide() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("big.png");
        image::RgbImage::new(1000, 500).save(&src).unwrap();
        let meta = std::fs::metadata(&src).unwrap();
        let e = Entry { path: src.clone(), kind: MediaKind::Image, size: meta.len(), mtime: meta.modified().unwrap() };
        let info = probe(&src, MediaKind::Image).unwrap();
        let out = dir.path().join("t.jpg");

        make_thumb(&e, &info, &out, THUMB_WIDTH).unwrap();
        assert_eq!(image::image_dimensions(&out).unwrap(), (480, 240));

        let big = dir.path().join("p.jpg");
        make_thumb(&e, &info, &big, PREVIEW_WIDTH).unwrap();
        assert_eq!(image::image_dimensions(&big).unwrap(), (1280, 640));
    }

    #[test]
    #[ignore = "usa ffmpeg e o vídeo real de ~/Wallpapers"]
    fn real_video_thumb() {
        let src = crate::paths::Paths::from_env().default_wallpaper();
        let meta = std::fs::metadata(&src).unwrap();
        let e = Entry { path: src.clone(), kind: MediaKind::Video, size: meta.len(), mtime: meta.modified().unwrap() };
        let info = probe(&src, MediaKind::Video).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("t.jpg");

        make_thumb(&e, &info, &out, THUMB_WIDTH).unwrap();
        assert_eq!(image::image_dimensions(&out).unwrap().0, 480);
    }
}
