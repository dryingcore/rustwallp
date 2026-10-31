## Instalar

Baixe o `.deb` abaixo e instale com o apt (ele instala também o que o app precisa):

```bash
sudo apt install ./rustwallp_*_amd64.deb
```

Depois, abra pelo menu de apps (`Mod+D`) → **Wallpapers**.

Funciona no Debian 12 e 13 com i3 + polybar. Precisa dos scripts de wallpaper
do [dotfiles](https://github.com/dryingcore/dotfiles) (a versão que lê
`~/.config/wallpaper/desktop` e `lock`).

## Remover

```bash
sudo apt remove rustwallp
rm -rf ~/.cache/rustwallp   # opcional: miniaturas
```

O wallpaper continua funcionando: quem o mostra são os scripts do dotfiles.
Detalhes no [README](https://github.com/dryingcore/rustwallp#readme).
