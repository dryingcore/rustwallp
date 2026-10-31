//! Copia a imagem da janela da polybar (X11 GetImage) para mostrar na prévia.
//! É a barra de verdade, então a prévia acompanha qualquer mudança nela.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, ImageFormat, Window};

pub struct BarImage {
    pub width: usize,
    pub height: usize,
    /// RGBA, linha por linha.
    pub rgba: Vec<u8>,
}

pub fn capture() -> Result<BarImage, String> {
    let (conn, screen) = x11rb::connect(None).map_err(|e| format!("sem X11: {e}"))?;
    let root = conn.setup().roots[screen].root;
    let win = find_polybar(&conn, root)?.ok_or("barra não encontrada")?;

    let geo = conn
        .get_geometry(win)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    let (w, h) = (geo.width, geo.height);
    let img = conn
        .get_image(ImageFormat::Z_PIXMAP, win, 0, 0, w, h, !0)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| format!("não deu para copiar a barra: {e}"))?;

    let rgba = bgrx_to_rgba(&img.data);
    if rgba.len() != w as usize * h as usize * 4 {
        return Err(format!("formato de imagem inesperado ({} bits)", img.depth));
    }
    Ok(BarImage {
        width: w as usize,
        height: h as usize,
        rgba,
    })
}

/// Procura em profundidade: o i3 põe a polybar (dock) dentro de um "i3-frame".
fn find_polybar(conn: &impl Connection, parent: Window) -> Result<Option<Window>, String> {
    let tree = conn
        .query_tree(parent)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    for &win in &tree.children {
        let class = conn
            .get_property(false, win, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 64)
            .map_err(|e| e.to_string())?
            .reply();
        if class.is_ok_and(|c| String::from_utf8_lossy(&c.value).to_lowercase().contains("polybar")) {
            return Ok(Some(win));
        }
    }
    for &win in &tree.children {
        if let Some(found) = find_polybar(conn, win)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// Z_PIXMAP de 24/32 bits vem como BGRX: troca para RGBA opaco.
pub fn bgrx_to_rgba(data: &[u8]) -> Vec<u8> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn bgrx_becomes_opaque_rgba() {
        assert_eq!(
            super::bgrx_to_rgba(&[1, 2, 3, 9, 4, 5, 6, 9, 7]),
            [3, 2, 1, 255, 6, 5, 4, 255]
        );
    }

    #[test]
    #[ignore = "precisa da sessão X com a polybar rodando"]
    fn captures_real_bar() {
        let bar = super::capture().unwrap();
        assert_eq!(bar.width, 1920);
        assert!(bar.height >= 15 && bar.height <= 40);
    }
}
