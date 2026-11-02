# rustwallp

[![CI](https://github.com/dryingcore/rustwallp/actions/workflows/ci.yml/badge.svg)](https://github.com/dryingcore/rustwallp/actions/workflows/ci.yml)

App desktop em Rust para escolher o **wallpaper da área de trabalho** e o da
**tela de bloqueio** num desktop **i3 + polybar**. Aceita vídeos (que tocam em
loop no fundo) e imagens.

- Galeria com miniaturas de tudo o que está em `~/Wallpapers`.
- Prévia com a **sua polybar de verdade** desenhada por cima do wallpaper.
- **Testar na tela** antes de decidir: vai para um workspace vazio, mostra o
  wallpaper de verdade e pergunta se você quer manter. Se não responder, volta.
- Mesmo fluxo para a tela de bloqueio.

Foi feito para um ambiente específico (o meu, descrito em
[dryingcore/dotfiles](https://github.com/dryingcore/dotfiles)) e não tenta
funcionar em outros: Debian, i3, polybar, `xwinwrap` + `mpv` no fundo e
`xsecurelock` na tela de bloqueio.

---

## Índice

- [O princípio: nada depende do app](#o-princípio-nada-depende-do-app)
- [Requisitos](#requisitos)
- [Instalação](#instalação)
- [Como usar](#como-usar)
- [Desinstalação](#desinstalação)
- [Como funciona](#como-funciona)
- [Solução de problemas](#solução-de-problemas)
- [Desenvolvimento](#desenvolvimento)
- [Limitações](#limitações)

---

## O princípio: nada depende do app

Esta é a regra que guiou todas as decisões do projeto: **remover o rustwallp
não pode quebrar nada nem mudar o que está funcionando**, e você nunca fica
dependente dele.

- **Quem mostra o wallpaper são os scripts do dotfiles**
  (`~/.config/wallpaper/start-wallpaper.sh` e `~/.config/i3/lock.sh`), não o
  app. O i3 chama esses scripts no login, com ou sem o app instalado.
- **O app só escolhe o arquivo.** A escolha fica em dois links simbólicos
  comuns, que qualquer pessoa entende e troca à mão:

  | Link | Para quê |
  |---|---|
  | `~/.config/wallpaper/desktop` | wallpaper da área de trabalho |
  | `~/.config/wallpaper/lock` | tela de bloqueio (sem ele, segue o da área de trabalho) |

- **Sem os links, tudo funciona como antes do app existir:** os scripts usam
  o vídeo padrão (`~/Wallpapers/windows-xp-loading-moewalls-com.mp4`). O mesmo
  vale para um link que aponta para um arquivo que foi apagado.
- **O app nunca edita** scripts, o config do i3 nem o da polybar, e não exige
  nenhuma regra nova no i3. Ele só escreve em:
  - os dois links acima;
  - `~/.cache/rustwallp/` (miniaturas; pode ser apagado quando quiser);
  - `~/Wallpapers/`, e só quando você usa **Adicionar** (copiando o arquivo).
- **Testar não grava nada:** durante o "Testar na tela" o script recebe o
  arquivo por uma variável e o link não muda. Se o app travar ou a sessão cair
  no meio de um teste, a sua escolha salva continua intacta.
- **Nunca deixa a tela sem wallpaper:** se aplicar falhar, ele volta o anterior.
  E se você fechar o app enquanto ele aplica, ele espera terminar antes de sair.

Dá para fazer tudo o que o app faz sem ele:

```bash
# trocar o wallpaper da área de trabalho
ln -sfn ~/Wallpapers/meu-video.mp4 ~/.config/wallpaper/desktop
~/.config/wallpaper/start-wallpaper.sh

# tela de bloqueio diferente da área de trabalho
ln -sfn ~/Wallpapers/minha-foto.png ~/.config/wallpaper/lock

# voltar ao padrão
rm ~/.config/wallpaper/desktop ~/.config/wallpaper/lock
```

---

## Requisitos

- **Debian 12 ou 13**, com **i3 + polybar** (X11).
- Os **scripts de wallpaper do [dotfiles](https://github.com/dryingcore/dotfiles)**,
  na versão que lê `~/.config/wallpaper/desktop` e `lock`
  (`start-wallpaper.sh`, `lock.sh`, `saver_wallpaper.sh`) e o `xwinwrap`.
  Se você usa o dotfiles, já tem.

O resto (`ffmpeg`, `feh`, `mpv`, GTK3 e as bibliotecas de X11/GPU) o pacote
`.deb` instala sozinho.

---

## Instalação

### Pelo pacote `.deb` (recomendado)

1. Baixe o `rustwallp_<versão>_amd64.deb` da [última versão](https://github.com/dryingcore/rustwallp/releases/latest).
2. Instale com o apt, que também instala as dependências:

   ```bash
   sudo apt install ./rustwallp_*_amd64.deb
   ```

3. Abra pelo menu de apps (`Mod+D`) → **rustwallp**, ou rode `rustwallp` no terminal.

Pelo terminal, de uma vez:

```bash
cd /tmp && gh release download --repo dryingcore/rustwallp --pattern '*.deb' \
  && sudo apt install ./rustwallp_*_amd64.deb
```

Pelo dotfiles: `cd ~/dotfiles && ./install.sh rustwallp` (faz o mesmo).

O pacote instala só isto:

| Arquivo | O que é |
|---|---|
| `/usr/bin/rustwallp` | o programa |
| `/usr/share/applications/rustwallp.desktop` | o atalho do menu ("rustwallp") |
| `/usr/share/icons/hicolor/scalable/apps/rustwallp.svg` | o ícone |
| `/usr/share/doc/rustwallp/copyright` | a licença |

**Atualizar:** baixe o `.deb` novo e rode o mesmo `sudo apt install ./…deb`.

### Compilando, só na sua `$HOME` (sem `sudo`)

Precisa do Rust (`curl https://sh.rustup.rs -sSf | sh`) e de
`sudo apt install libgtk-3-dev ffmpeg feh mpv`.

```bash
git clone https://github.com/dryingcore/rustwallp.git
cd rustwallp
./install.sh
```

Compila em release (alguns minutos na primeira vez) e grava três arquivos:
`~/.local/bin/rustwallp`, o ícone em `~/.local/share/icons/hicolor/scalable/apps/`
e o atalho em `~/.local/share/applications/`. Atualizar: `git pull && ./install.sh`.

Use **um** dos dois jeitos. Se tiver os dois, o menu mostra o da sua `$HOME`.

---

## Como usar

### A tela

- **Abas no topo:** "Área de trabalho" e "Tela de bloqueio". Cada uma tem a sua
  escolha. O botão **+ Adicionar** fica à direita.
- **Prévia:** um quadro do wallpaper selecionado, em 16:9.
  - Na aba da área de trabalho, a **sua polybar de verdade** aparece por cima,
    copiada da tela a cada 2 segundos (os gráficos de CPU e RAM se mexem).
  - Na aba da tela de bloqueio, aparece o campo de senha no visual do
    `xsecurelock`.
- **Embaixo da prévia:** nome, tipo, tamanho, duração e resolução, e os botões
  **Testar na tela** e **Aplicar**.
- **Painel da direita:** o que está em uso agora (área de trabalho e
  bloqueio), a opção "Igual à área de trabalho" e os detalhes de vídeo.
- **Galeria:** tudo o que está em `~/Wallpapers`. Borda ciano = selecionado;
  ponto verde = em uso; `▶` = vídeo (com a duração, quando o arquivo informa).
- **Rodapé:** a pasta, quantos arquivos e as mensagens (inclusive erros).

### Aplicar

Selecione na galeria e clique em **✓ Aplicar**.

- **Área de trabalho:** troca na hora. Para vídeos, o app espera uns 3 segundos
  para confirmar que o vídeo começou a tocar; se não começou, volta o anterior
  e mostra o erro no rodapé.
- **Tela de bloqueio:** vale no próximo bloqueio.

### Testar na tela

Para ver o wallpaper de verdade, com a barra, antes de decidir.

**Área de trabalho:**

1. O app pergunta ao i3 quais workspaces existem e escolhe um **vazio** (o
   primeiro número de 1 a 10 que não existe; se todos existirem, usa
   `rustwallp-teste`). Suas janelas não são movidas nem escondidas.
2. Vai para esse workspace e mostra o wallpaper **sem gravar nada**: a sua
   escolha salva só muda se você clicar em Manter.
3. Aparece uma janelinha com **✓ Manter** e **↶ Voltar** e uma contagem de 15 s.
4. **Manter:** fica com o novo. **Voltar**, tempo esgotado ou fechar o app no
   meio: volta o anterior. Nos dois casos você volta ao workspace onde estava.

Se o i3 não responder, o botão fica desativado (passe o mouse para ver o motivo).

**Tela de bloqueio:** bloqueia a tela de verdade com o arquivo escolhido, sem
salvar nada. Desbloqueie com a sua senha, como sempre, e o app pergunta se você
quer manter aquele como tela de bloqueio.

### Tela de bloqueio igual ou diferente

- **"Igual à área de trabalho" marcado** (padrão): a tela de bloqueio segue o
  wallpaper da área de trabalho, inclusive quando você troca.
- **Desmarcado:** a tela de bloqueio fica com a própria escolha. Vá à aba
  "Tela de bloqueio", selecione e aplique.

### Adicionar

**+ Adicionar** abre a janela de escolher arquivo. O arquivo é **copiado** para
`~/Wallpapers` (o original fica onde estava) e já aparece selecionado.

- Formatos: vídeo `mp4 mkv webm mov`; imagem `png jpg jpeg webp` (maiúsculas
  ou minúsculas).
- Nunca sobrescreve: se já existir `foto.jpg`, a cópia vira `foto-1.jpg`.

Para apagar ou renomear wallpapers, mexa direto na pasta `~/Wallpapers`.

---

## Desinstalação

### Instalado pelo `.deb`

```bash
sudo apt remove rustwallp
rm -rf ~/.cache/rustwallp     # opcional: as miniaturas
```

O apt apaga exatamente os arquivos que o pacote instalou. Ele não mexe na sua
`$HOME`, então `~/Wallpapers` e a sua escolha de wallpaper ficam como estão.

### Instalado pelo `install.sh`

```bash
cd rustwallp
./uninstall.sh
```

Apaga os três arquivos da instalação e o cache `~/.cache/rustwallp/`.

### A escolha de wallpaper

Nos dois casos, os links `~/.config/wallpaper/desktop` e `lock` ficam, e os
scripts continuam mostrando o último wallpaper escolhido. Para voltar ao vídeo
padrão:

```bash
rm -f ~/.config/wallpaper/desktop ~/.config/wallpaper/lock
```

(o `./uninstall.sh` pergunta isso; sem perguntar: `--keep` ou `--links`).

A remoção foi testada de verdade: instalar, escolher um wallpaper, desinstalar
e conferir por hash que os scripts e as configs do i3, da polybar e do
wallpaper ficaram idênticos, e que o wallpaper sobe igual sem o app.

---

## Como funciona

### Quem faz o quê

```
rustwallp (este app)              scripts do dotfiles (sempre rodam)
────────────────────              ──────────────────────────────────
escolhe o arquivo        ──▶      ~/.config/wallpaper/desktop ─▶ start-wallpaper.sh
troca o link (atômico)            ~/.config/wallpaper/lock    ─▶ lock.sh ─▶ saver_wallpaper.sh
roda o script e confere                                          │
                                  vídeo  ─▶ xwinwrap + mpv (loop, GPU, sem som)
                                  imagem ─▶ feh --bg-fill (parada, sem processo)
```

### Os scripts

- **`start-wallpaper.sh`** lê a variável `RUSTWALLP_DESKTOP_FILE` (só usada
  pelo "Testar"), depois o link `desktop` (ou usa o vídeo padrão), encerra
  o wallpaper anterior e mostra o novo: vídeo com `xwinwrap` + `mpv`, imagem
  com `feh`. O tipo vem da extensão.
- **`lock.sh`** decide o arquivo do bloqueio nesta ordem: a variável
  `RUSTWALLP_LOCK_FILE` (usada só pelo "Testar"), o link `lock`, o link
  `desktop` e, por fim, o vídeo padrão.
- **`saver_wallpaper.sh`** toca o arquivo atrás do campo de senha do
  `xsecurelock`, em loop (imagens ficam paradas).

### Aplicar, por dentro

1. Guarda para onde o link apontava.
2. Troca o link de forma **atômica**: cria `desktop.tmp` e renomeia por cima.
   Nunca existe um instante sem link.
3. Roda o `start-wallpaper.sh` numa sessão própria (`setsid`) e sem herdar a
   entrada/saída do app. Assim o wallpaper não depende do app: fechar o app não
   derruba o vídeo.
4. Confere: o arquivo existe, o script terminou sem erro e, se for vídeo, o
   `xwinwrap` está rodando 3 s depois.
5. Se algo falhou, restaura o link anterior e roda o script de novo.

### Miniaturas e prévia

- Vídeos: o `ffmpeg` tira um quadro em `min(3 s, metade da duração)`; o
  `ffprobe` informa duração e resolução. Imagens: o próprio app reduz.
- Cada arquivo gera uma miniatura (480 px) e um quadro de prévia (1280 px), em
  `~/.cache/rustwallp/thumbs/`. O nome inclui caminho, tamanho e data de
  modificação: só é refeito quando o arquivo muda.
- Tudo isso roda numa thread separada: a janela abre na hora e as miniaturas
  aparecem conforme ficam prontas.
- A barra da prévia é a janela da polybar copiada pelo X11. O i3 coloca a
  polybar dentro de uma moldura própria, então o app procura na árvore de
  janelas pela classe `polybar`.

### Estrutura do código

| Arquivo | Responsabilidade |
|---|---|
| `src/main.rs` | abre a janela |
| `src/paths.rs` | onde fica cada coisa (`~/Wallpapers`, links, cache, scripts) |
| `src/library.rs` | lista `~/Wallpapers` e separa vídeo de imagem |
| `src/selection.rs` | lê e troca os links |
| `src/apply.rs` | aplica, confere e volta o anterior; teste da tela de bloqueio |
| `src/i3.rs` | lista workspaces e escolhe um vazio |
| `src/test_session.rs` | "Testar na tela": contagem, manter, voltar |
| `src/media.rs` | `ffprobe`, miniaturas e quadros de prévia |
| `src/thumbs.rs` | fila de miniaturas em segundo plano |
| `src/barcap.rs` | copia a imagem da polybar |
| `src/add.rs` | "Adicionar" sem sobrescrever |
| `src/ui/` | a interface (egui): abas, prévia, galeria, painel, diálogos |

O design completo está em [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md).

---

## Solução de problemas

**A prévia mostra "barra não encontrada".**
A polybar não está rodando (ou não está numa sessão X11). Abra a barra
(`Mod+Shift+R`) e espere 2 segundos.

**"Testar na tela" está desativado.**
O app não conseguiu falar com o i3 (`i3-msg -t get_workspaces`). Confira se
está numa sessão i3 e se o `i3-msg` funciona no terminal.

**Apliquei e apareceu "o vídeo não começou a tocar".**
O app já voltou o wallpaper anterior. Teste o arquivo no terminal:
`mpv ~/Wallpapers/arquivo.mp4`. Se o `mpv` não abrir, o problema é o arquivo.

**Uma miniatura mostra "sem prévia".**
O `ffmpeg` não conseguiu ler o arquivo. Passe o mouse em cima para ver o erro.

**Um vídeo aparece só com `▶`, sem duração.**
O arquivo não informa a duração (comum em vídeos cortados para web). Ele toca
normalmente.

**O botão Adicionar não abre nada.**
Falta o GTK3 em tempo de compilação: `sudo apt install libgtk-3-dev` e rode o
`./install.sh` de novo.

**Quero voltar ao wallpaper de antes do app.**
`rm ~/.config/wallpaper/desktop ~/.config/wallpaper/lock` e `Mod+Shift+R`.

---

## Desenvolvimento

### CI

A cada push e pull request, o [GitHub Actions](.github/workflows/ci.yml) roda,
no Ubuntu 22.04:

1. `cargo fmt --check` (formatação);
2. `cargo clippy --all-targets -- -D warnings`;
3. `cargo test` (a lógica: links, workspace vazio, miniaturas, aplicar/julgar,
   contagem do teste, nomes livres no "Adicionar");
4. `packaging/build-deb.sh`, guardando o `.deb` como artefato da execução.

Os testes marcados com `#[ignore]` precisam da sessão real e só rodam à mão.

### Publicar uma versão

1. Suba a versão em `Cargo.toml` (`version = "0.2.0"`) e rode `cargo build` para atualizar o `Cargo.lock`.
2. `git commit -am "versão 0.2.0" && git tag v0.2.0 && git push --follow-tags`
3. O CI roda tudo e, por ser uma tag `v*`, cria a versão no GitHub com o `.deb`
   anexado (notas em [`packaging/release-notes.md`](packaging/release-notes.md)).

Para gerar o `.deb` localmente: `packaging/build-deb.sh` (sai em `dist/`).

### Comandos

```bash
cargo run                         # roda sem instalar
cargo test                        # testes da lógica (não mexem na sua tela)
cargo test -- --ignored           # testes com a sessão real: troca o wallpaper e volta,
                                  # copia a polybar, gera miniatura do vídeo padrão
cargo clippy --all-targets -- -D warnings
```

Os scripts têm testes próprios no dotfiles, numa `$HOME` falsa:
`tests/test-wallpaper-scripts.sh`.

### Conferência manual antes de uma versão

- [ ] aplicar um vídeo e uma imagem
- [ ] testar na tela e escolher "Voltar"
- [ ] testar na tela e deixar o tempo acabar
- [ ] testar na tela e fechar o app durante a contagem (o anterior volta)
- [ ] testar com os workspaces 1 a 10 ocupados (usa `rustwallp-teste`)
- [ ] testar a tela de bloqueio e manter
- [ ] parar a polybar: a prévia mostra "barra não encontrada"
- [ ] `./uninstall.sh` e reiniciar a sessão: o wallpaper sobe igual

---

## Limitações

- Um monitor só.
- Só X11 com i3 (sem Wayland, sway ou GNOME).
- Não toca o vídeo dentro da prévia (o "Testar na tela" mostra o real).
- Não apaga nem renomeia wallpapers: use a pasta `~/Wallpapers`.

## Licença

MIT
