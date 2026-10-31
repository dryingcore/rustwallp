#!/usr/bin/env bash
# Compila e instala o rustwallp só na sua $HOME. Grava exatamente 3 arquivos,
# que o uninstall.sh apaga:
#   ~/.local/bin/rustwallp
#   ~/.local/share/icons/hicolor/scalable/apps/rustwallp.svg
#   ~/.local/share/applications/rustwallp.desktop

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"

CARGO="$(command -v cargo || echo "$HOME/.cargo/bin/cargo")"
"$CARGO" build --release --locked

install -Dm755 target/release/rustwallp "$HOME/.local/bin/rustwallp"
install -Dm644 assets/rustwallp.svg "$HOME/.local/share/icons/hicolor/scalable/apps/rustwallp.svg"
mkdir -p "$HOME/.local/share/applications"
sed -e "s|@EXEC@|$HOME/.local/bin/rustwallp|" \
	-e "s|@ICON@|$HOME/.local/share/icons/hicolor/scalable/apps/rustwallp.svg|" \
	assets/rustwallp.desktop > "$HOME/.local/share/applications/rustwallp.desktop"

echo "rustwallp instalado. Abra pelo menu (Mod+D → Wallpapers) ou rode: ~/.local/bin/rustwallp"
