# rustwallp — arquitetura

Como o app funciona por dentro e por que cada decisão foi tomada. Para instalar
e usar, veja o [README](../README.md).

## 1. Objetivo

- Trocar o wallpaper sem editar scripts à mão.
- Ver antes como fica: prévia com a barra real por cima e um modo de teste na
  tela de verdade.
- Aceitar vídeos e imagens, para a área de trabalho e para a tela de bloqueio.

## 2. A regra principal: nada depende do app

Remover o rustwallp não pode quebrar nada nem mudar o que está funcionando.

1. **O app não mostra o wallpaper.** Quem mostra são os scripts do dotfiles:
   `start-wallpaper.sh` (área de trabalho), `lock.sh` e `saver_wallpaper.sh`
   (tela de bloqueio). O i3 os chama no login, com ou sem o app.
2. **A escolha mora em dois links simbólicos comuns:**
   - `~/.config/wallpaper/desktop`
   - `~/.config/wallpaper/lock`
3. **Os scripts têm um padrão próprio.** Sem link, ou com link para um arquivo
   que sumiu, usam o vídeo padrão (`~/Wallpapers/windows-xp-loading-moewalls-com.mp4`),
   que é exatamente o comportamento de antes do app.
4. **O app só escreve em:** os dois links, `~/.cache/rustwallp/` e
   `~/Wallpapers/` (só no "Adicionar"). Nunca edita scripts, o config do i3 nem
   o da polybar.
5. **Nenhuma regra nova no i3.** Os diálogos se declaram como
   `_NET_WM_WINDOW_TYPE_DIALOG`, que o i3 já deixa flutuante.
6. **Testar não grava nada.** Durante o "Testar na tela", o script recebe o
   arquivo por variável de ambiente e o link não muda. Se o app travar ou a
   sessão cair no meio de um teste, a escolha salva continua intacta.
7. **Nunca deixa a tela sem wallpaper.** Se aplicar falhar, volta o anterior.
8. **Desinstalar** apaga só o executável, o ícone, o atalho do menu e o cache.

## 3. O contrato com os scripts

### `start-wallpaper.sh`

```text
arquivo = $RUSTWALLP_DESKTOP_FILE, se definido e existir   (só o "Testar na tela")
        senão o alvo do link ~/.config/wallpaper/desktop, se existir
        senão o vídeo padrão
encerra o xwinwrap anterior
vídeo  -> xwinwrap + mpv (loop, --hwdec=auto, sem som)
imagem -> feh --bg-fill (fica parada, sem processo rodando)
sai com erro se o arquivo final não existe
```

O tipo vem da extensão, sem diferenciar maiúsculas: vídeo `mp4 mkv webm mov`,
imagem `png jpg jpeg webp`. É a mesma lista do app (`library.rs`).

### `lock.sh` e `saver_wallpaper.sh`

```text
arquivo do bloqueio = $RUSTWALLP_LOCK_FILE, se definido   (só o "Testar" da aba de bloqueio)
                    senão o link ~/.config/wallpaper/lock
                    senão o link ~/.config/wallpaper/desktop   ("igual à área de trabalho")
                    senão o vídeo padrão
```

O `saver_wallpaper.sh` passa `--image-display-duration=inf` ao `mpv`, que assim
também mostra imagens paradas.

"Igual à área de trabalho" significa **não ter** o link `lock`: a tela de
bloqueio segue o desktop automaticamente, inclusive quando ele muda. Com o link
`lock`, ela é independente, mesmo que aponte para o mesmo arquivo.

## 4. Aplicar e testar

### Aplicar (área de trabalho)

1. Guarda o alvo atual do link.
2. Troca o link de forma atômica: cria `desktop.tmp` e renomeia por cima.
3. Roda o `start-wallpaper.sh` com `setsid -w`, com entrada e saída em
   `/dev/null`. Assim o `xwinwrap` fica numa sessão própria: fechar o app não
   derruba o vídeo, e ninguém fica preso esperando a saída do app.
4. Julga o resultado. Falhou se o arquivo não existe, se o script saiu com
   erro, ou (só vídeo) se nenhum `xwinwrap` está rodando 3 s depois. Imagem
   não deixa processo rodando: basta o `feh` sair sem erro.
5. Se falhou, restaura o link anterior (ou o remove, se não havia) e roda o
   script de novo.

### Aplicar (tela de bloqueio)

Só troca o link `lock`; vale no próximo bloqueio.

### Testar na tela (área de trabalho)

1. Pergunta ao i3 os workspaces (`i3-msg -t get_workspaces`). No i3 um
   workspace só existe com janelas ou em foco, então o primeiro número de 1 a
   10 fora da lista está vazio. Se todos existirem, usa `rustwallp-teste`
   (ou `rustwallp-teste-2`…).
2. Vai para esse workspace e roda o script com `RUSTWALLP_DESKTOP_FILE`.
3. Mostra o diálogo **Manter / Voltar** com 15 s.
4. **Manter:** grava o link (a tela já mostra o arquivo). **Voltar**, tempo
   esgotado ou app fechado: roda o script sem a variável, mostrando o que está
   salvo. Nos dois casos volta ao workspace de origem.

Detalhe do eframe: indo para outro workspace, o i3 marca a janela principal
como escondida, e o eframe deixa de chamar `App::ui`, só `App::logic`. Por isso
o app implementa `logic`, que acompanha os trabalhos em andamento e, enquanto
houver um, pede `RequestPaintWhileHidden` para o diálogo continuar sendo
desenhado. As threads de trabalho também acordam a interface ao terminar.

### Testar na tela (tela de bloqueio)

Roda `RUSTWALLP_LOCK_FILE=<arquivo> ~/.config/i3/lock.sh` numa thread e espera
o usuário desbloquear. Nada é gravado. Depois pergunta se quer manter; se sim,
grava o link `lock`.

### Fechar o app no meio

- Teste em andamento: equivale a "Voltar".
- Aplicar em andamento: o app espera terminar (até 20 s) antes de sair, para
  que uma falha ainda consiga restaurar o anterior.

## 5. Miniaturas, prévia e a barra

- **Vídeo:** `ffmpeg -ss <min(3 s, duração/2)> -frames:v 1`. Duração e
  resolução via `ffprobe`. Alguns vídeos não gravam a duração: aí a galeria
  mostra só `▶`.
- **Imagem:** carregada e reduzida pelo próprio app (crate `image`).
- **Dois tamanhos:** miniatura de 480 px (galeria) e quadro de 1280 px (prévia).
- **Cache:** `~/.cache/rustwallp/thumbs/<hash(caminho, tamanho, mtime)>.jpg` e
  `…-preview.jpg`. Arquivo modificado gera hash novo.
- **Thread de fundo:** a janela abre na hora e as miniaturas aparecem conforme
  ficam prontas.
- **Barra:** a imagem da janela da polybar é copiada via X11 (`GetImage`, crate
  `x11rb`) a cada 2 s e desenhada no topo da prévia, na escala certa. O i3 põe
  a polybar dentro de uma moldura própria (`i3-frame`), então a busca desce na
  árvore de janelas procurando `WM_CLASS` `polybar`. Sem polybar: só o
  wallpaper e o aviso "barra não encontrada".
- **Prévia da tela de bloqueio:** não há janela para copiar, então o app
  desenha uma imitação do prompt do `xsecurelock` (usuário, data, asteriscos).

## 6. Módulos

| Módulo | Responsabilidade | Depende de |
|---|---|---|
| `paths.rs` | caminhos a partir de `$HOME` | — |
| `library.rs` | listar `~/Wallpapers`, vídeo ou imagem | — |
| `selection.rs` | ler/trocar os links (troca atômica) | `paths` |
| `apply.rs` | aplicar, julgar, restaurar; prévia por variável; teste do bloqueio | `selection`, `library` |
| `i3.rs` | `get_workspaces`, workspace vazio, trocar de workspace | `i3-msg` |
| `test_session.rs` | contagem de 15 s, manter, voltar | `apply`, `i3` |
| `media.rs` | `ffprobe`, miniaturas e quadros de prévia | `ffmpeg`, `image` |
| `thumbs.rs` | fila de miniaturas em thread | `media` |
| `barcap.rs` | copiar a janela da polybar | `x11rb` |
| `add.rs` | copiar para `~/Wallpapers` sem sobrescrever | `library` |
| `ui/` | interface egui: abas, prévia, galeria, painel, diálogos | todos |

A lógica que decide algo (classificação, links, workspace vazio, julgamento,
contagem, nomes livres, cache) não depende de X11 e é coberta por `cargo test`.
O que depende da sessão real tem testes marcados com `#[ignore]`.

## 7. Fora do escopo, de propósito

- Apagar ou renomear wallpapers (é só mexer na pasta).
- Tocar o vídeo dentro da prévia (o "Testar na tela" mostra o real).
- Configurar a polybar ou o i3.
- Outros ambientes (Wayland, sway, GNOME) e mais de um monitor.
