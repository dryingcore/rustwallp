#!/usr/bin/env bash
# Remove o rustwallp. Apaga exatamente o que o install.sh gravou, mais o cache de
# miniaturas. Nunca mexe em ~/Wallpapers nem nos scripts do wallpaper.
#
# Os links ~/.config/wallpaper/{desktop,lock} ficam, por padrão: os scripts
# continuam mostrando o último wallpaper escolhido. Apagando os links, voltam
# ao vídeo padrão (o comportamento de antes do rustwallp).
#
# Uso: ./uninstall.sh            (pergunta sobre os links)
#      ./uninstall.sh --links    (apaga os links sem perguntar)
#      ./uninstall.sh --keep     (mantém os links sem perguntar)

set -euo pipefail

rm -f "$HOME/.local/bin/rustwallp" \
	"$HOME/.local/share/icons/hicolor/scalable/apps/rustwallp.svg" \
	"$HOME/.local/share/applications/rustwallp.desktop"
rm -rf "$HOME/.cache/rustwallp"
echo "rustwallp removido (executável, ícone, atalho do menu e cache)."

links=("$HOME/.config/wallpaper/desktop" "$HOME/.config/wallpaper/lock")
answer=n
case "${1:-}" in
--links) answer=s ;;
--keep) answer=n ;;
*)
	if [[ -t 0 ]] && { [[ -L "${links[0]}" ]] || [[ -L "${links[1]}" ]]; }; then
		read -r -p "Apagar também a escolha de wallpaper (volta ao vídeo padrão)? [s/N] " answer
	fi
	;;
esac

if [[ "${answer,,}" == s* ]]; then
	rm -f "${links[@]}"
	echo "Escolha apagada: no próximo login (ou Mod+Shift+R) volta o vídeo padrão."
else
	echo "Escolha mantida: os scripts continuam mostrando o último wallpaper escolhido."
fi
