//! Miniaturas geradas numa thread de fundo, para a janela abrir na hora.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crate::library::Entry;
use crate::media::{self, Info};

/// Resultado por arquivo: (caminho do wallpaper, (miniatura, quadro da prévia, informações) ou erro).
pub type ThumbResult = (PathBuf, Result<(PathBuf, PathBuf, Info), String>);

pub struct Thumbs {
    requests: Sender<Entry>,
    results: Receiver<ThumbResult>,
}

impl Thumbs {
    pub fn new(cache: PathBuf) -> Thumbs {
        let (requests, inbox) = channel::<Entry>();
        let (outbox, results) = channel();
        thread::spawn(move || {
            for e in inbox {
                let result = media::probe(&e.path, e.kind)
                    .ok_or_else(|| format!("não deu para ler {}", e.path.display()))
                    .and_then(|info| {
                        let thumb = media::thumb_path(&cache, &e);
                        if !thumb.exists() {
                            media::make_thumb(&e, &info, &thumb, media::THUMB_WIDTH)?;
                        }
                        let preview = media::preview_path(&cache, &e);
                        if !preview.exists() {
                            media::make_thumb(&e, &info, &preview, media::PREVIEW_WIDTH)?;
                        }
                        Ok((thumb, preview, info))
                    });
                if outbox.send((e.path, result)).is_err() {
                    break;
                }
            }
        });
        Thumbs { requests, results }
    }

    pub fn request(&self, e: Entry) {
        let _ = self.requests.send(e);
    }

    /// Tudo o que ficou pronto desde a última chamada; não bloqueia.
    pub fn poll(&self) -> Vec<ThumbResult> {
        self.results.try_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::MediaKind;
    use std::time::{Duration, Instant};

    #[test]
    fn worker_produces_thumb_for_image() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.png");
        image::RgbImage::new(960, 540).save(&src).unwrap();
        let meta = std::fs::metadata(&src).unwrap();
        let e = Entry {
            path: src.clone(),
            kind: MediaKind::Image,
            size: meta.len(),
            mtime: meta.modified().unwrap(),
        };

        let thumbs = Thumbs::new(dir.path().join("cache"));
        thumbs.request(e);

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut got = Vec::new();
        while got.is_empty() && Instant::now() < deadline {
            got = thumbs.poll();
            std::thread::sleep(Duration::from_millis(20));
        }
        let (path, result) = got.pop().expect("nenhum resultado em 5 s");
        let (thumb, preview, info) = result.unwrap();
        assert_eq!(path, src);
        assert!(thumb.exists());
        assert_eq!(image::image_dimensions(&preview).unwrap().0, 1280);
        assert_eq!((info.width, info.height), (960, 540));
    }
}
