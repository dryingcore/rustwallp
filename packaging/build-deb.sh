#!/usr/bin/env bash
# Gera dist/rustwallp_<versão>_<arquitetura>.deb, usando só o dpkg-deb.
#
#   packaging/build-deb.sh            compila em release e empacota
#   packaging/build-deb.sh --no-build empacota o target/release/rustwallp que já existe
#
# O pacote instala:
#   /usr/bin/rustwallp
#   /usr/share/applications/rustwallp.desktop   (aparece no menu como "Wallpapers")
#   /usr/share/icons/hicolor/scalable/apps/rustwallp.svg
#   /usr/share/doc/rustwallp/copyright
# e `apt remove rustwallp` apaga exatamente isso.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

NAME=rustwallp
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
ARCH="$(dpkg --print-architecture)"
OUT="dist/${NAME}_${VERSION}_${ARCH}.deb"

if [[ "${1:-}" != "--no-build" ]]; then
	CARGO="$(command -v cargo || echo "$HOME/.cargo/bin/cargo")"
	"$CARGO" build --release --locked
fi

ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT
chmod 755 "$ROOT"

install -Dm755 "target/release/$NAME" "$ROOT/usr/bin/$NAME"
install -Dm644 "assets/$NAME.svg" "$ROOT/usr/share/icons/hicolor/scalable/apps/$NAME.svg"
mkdir -p "$ROOT/usr/share/applications"
sed -e "s|@EXEC@|$NAME|" -e "s|@ICON@|$NAME|" "assets/$NAME.desktop" \
	> "$ROOT/usr/share/applications/$NAME.desktop"
chmod 644 "$ROOT/usr/share/applications/$NAME.desktop"
install -Dm644 LICENSE "$ROOT/usr/share/doc/$NAME/copyright"

mkdir -p "$ROOT/DEBIAN"
# Bibliotecas: as que o binário liga direto e as que o winit/wgpu/rfd abrem em
# tempo de execução (X11, teclado, EGL/Vulkan, GTK3). No Debian 13 o GTK3 se
# chama libgtk-3-0t64. Programas: ffmpeg (miniaturas), feh (imagem no fundo),
# mpv (vídeo no fundo).
cat > "$ROOT/DEBIAN/control" <<EOF
Package: $NAME
Version: $VERSION
Architecture: $ARCH
Maintainer: dryingcore <137729079+dryingcore@users.noreply.github.com>
Installed-Size: $(du -sk "$ROOT/usr" | cut -f1)
Section: x11
Priority: optional
Homepage: https://github.com/dryingcore/rustwallp
Depends: libc6, libgcc-s1, libdbus-1-3, libfontconfig1, libx11-6, libx11-xcb1, libxcb1, libxcursor1, libxi6, libxkbcommon0, libxkbcommon-x11-0, libegl1 | libvulkan1, libgtk-3-0t64 | libgtk-3-0, ffmpeg, feh, mpv
Recommends: i3-wm, polybar, xsecurelock
Description: escolhe o wallpaper da área de trabalho e da tela de bloqueio (i3 + polybar)
 App desktop com galeria de ~/Wallpapers (vídeos e imagens), prévia com a
 polybar real por cima e "Testar na tela" num workspace vazio.
 .
 O app só troca os links ~/.config/wallpaper/desktop e lock; quem mostra o
 wallpaper são os scripts do dotfiles. Remover o pacote não muda nada.
EOF

mkdir -p dist
dpkg-deb --root-owner-group --build "$ROOT" "$OUT" >/dev/null
echo "$OUT"
