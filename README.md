# local-app-template

O molde executável dos apps desktop da suíte Local: Rust puro + egui forkado, CI nos 3 SOs,
empacotamento completo, auto-update (`craft-update`), i18n, xtask, fontes — **sem domínio de app**.
Todo app novo da suíte nasce daqui em vez de ser copiado de outro app (foi assim que o
`craft-update` divergiu ×10).

Não é um template "por tipo de app". Kits de capacidade (editor-kit do Word, media-kit do player,
vault-kit do Obsidian) nascem de app real concluído, no `local-libs`, quando houver segundo
consumidor — ver `docs/planos/plano-bibliotecas.md` (F10).

## O que já vem pronto

| Peça | Onde |
|---|---|
| Janela egui com o frame certo por desktop (probe + `craft-window`), barra de menus, menu de janela quando o app desenha o frame | `apps/localapp/src/app.rs` |
| Janela **About** com verificação de atualizações (`craft-update`), prompt de update no start e self-update por MSI/AppImage/pacman | `apps/localapp/src/app.rs`, `src/suite_entry.rs` |
| i18n por catálogo TSV (`craft-i18n`), 3 traduções de exemplo (pt-br, es, ja), seletor de idioma | `i18n/locales/`, `apps/localapp/src/i18n.rs`, `build.rs` |
| Seletor nativo de arquivo assíncrono (rfd 0.17 + pollster, sem travar o frame) | `apps/localapp/src/pickers.rs` |
| Fontes craft-fonts embutidas (CJK/árabe/hebraico/devanágari) | `assets/craft-fonts/`, `apps/localapp/src/fonts.rs`, `build.rs` |
| CI: Ubuntu + Windows (fmt, clippy `-D warnings`, build `--locked`, testes, gates) + Arch (`makepkg --printsrcinfo`) + guardião de fontes `git+` | `.github/workflows/ci.yml` |
| Release por tag `v*`: MSI WiX v5 (atalho Iniciar **e** desktop), portable zip, AppImage/deb/rpm/tar, pacote Arch com `pkgver` carimbado, checksums e assinatura minisign de todos os assets | `.github/workflows/release.yml`, `packaging/` |
| xtask: `fmt`, `assets`, `layers`, `version`, `ico` | `xtask/` |
| Forks vendored (egui 6d518fc, winit a517b38, craft-libs, rustls-rustcrypto, winprint.rs) | `libs/` |
| Regras do repo (deps/lock, commits, release, disco) | `AGENTS.md` |

## Como nascer um app (checklist)

O script `scripts/nascer-app.ps1` faz 1–3 e 5 num repo recém-clonado; os passos com 🔑 são
decisões que só você toma.

1. **Crie o repo** a partir deste: `gh repo create Anon5T4R/<repo> --public --source . --push` (ou
   use o botão "Use this template"). Clone em `C:\Dev\underworld-apps\<app>`.
2. **Renomeie**: `powershell -File scripts/nascer-app.ps1 -Id <id> -Name "<Nome>"`, onde `<id>` é o
   nome máquina (`localkeys`, minúsculo, sem espaço) e `<Nome>` o display (`Local Keys`). O script
   renomeia crates/pacote/binário/ids/env vars/pastas, **gera GUIDs novos** do MSI e reescreve os
   pontos de identificação; ele lista cada arquivo que tocou.
   🔑 Ele **não** escolhe ícone, tagline, categoria de empacotamento nem traduções.
3. 🔑 **Ícone**: substitua o placeholder cinza com "?" — desenhe o SVG, aponte o
   `packaging/icons.sh`, rode-o (`resvg`) e faça a QA visual. Ver `assets/app-icon/README.md`.
4. 🔑 **GUIDs**: o script gera um UpgradeCode e os Component GUIDs. Nunca reutilize os do template
   nem de outro app: o UpgradeCode identifica o produto entre versões (dois apps com o mesmo se
   sobrescrevem) e um Component GUID é casado com o que instala.
5. **Identidade de empacotamento**: `apps/localapp/src/suite_entry.rs` (`APP`: id, nome, tagline,
   repo), `packaging/linux/<id>.desktop` + `<id>.metainfo.xml.in`, `packaging/arch/PKGBUILD`,
   `packaging/windows/<id>.wxs` (nome, fabricante, chaves de registro), URLs em `package.ps1`,
   `nfpm.yaml`, `Cargo.toml` (`repository`). Nota: `craft-update`'s `App::bundle_id()` devolve
   `com.<id>.app` só para os ids que ele conhece (`localpaint`, `localphoto`) — os outros caem em
   `ai.storyteller.<id>`. Os 10 apps convivem com isso porque instalam por MSI/pacman (o registro
   de menu do "rodou fora do pacote" é a única rota que usa `bundle_id()`); decida se o app nascido
   mantém `com.<id>.app` no `.desktop`/MSI (padrão da suíte) ou entra no match da crate.
6. **Entrada no catálogo da suíte** (o passo que o template não pode fazer por você):
   1. No repo `Anon5T4R/local-libs` (`C:\Dev\underworld\local-libs`), adicione a entrada do app em
      `crates/update/src/suite.rs` (`pub const <APP>: App = .`) **e** à lista `SUITE` (ajuste o
      tamanho), com id, nome, tagline e repo reais. Rode os testes da crate.
   2. `powershell -File scripts/sync.ps1 -Mode push` — copia a crate para o `libs/craft-libs` de
      todos os apps e confere por hash (este repo incluído). Commite a fonte (`HASHES.update`
      junto) e depois cada app.
   3. Aqui: troque o `App` declarado em `apps/localapp/src/suite_entry.rs` por
      `craft_update::suite::<APP>` e apague o aviso de template do arquivo.
7. **Assinatura minisign** (antes da primeira release): no repo `Anon5T4R/local-libs`,
   `powershell -File scripts/minisign-nascimento.ps1 -Repo Anon5T4R/<repo>` — seta
   `MINISIGN_KEY`/`MINISIGN_PASSPHRASE`, confere o backup e valida `packaging/minisign.pub`. Rode de
   novo **após o primeiro push** (ele lê a pubkey do repo).
8. **Dependências**: registre toda dep nova no inventário `Anon5T4R/local-deps`
   (`DEPENDENCIES.md`, regra no `WORKFLOW.md` do repo) e commite o `Cargo.lock`.
9. **Primeira release**: `cargo xtask version set 0.1.0` (se ainda não estiver), `git tag v0.1.0`,
   `git push origin v0.1.0`. O Actions builda e publica.

## Estrutura

```
apps/localapp/     app: main.rs (args, ícone, janela), app.rs (menus/About/update), i18n.rs,
                   pickers.rs, fonts.rs, suite_entry.rs, build.rs (ícone, i18n, fontes)
assets/            app-icon/ (placeholder), craft-fonts/ (vendored), ATTRIBUTION.md
i18n/locales/      catálogos de tradução (context ⇥ fonte EN ⇥ tradução)
libs/              forks vendored: craft-libs, egui, winit, rustls-rustcrypto, winprint.rs
packaging/         env.sh, fonts.sh, icons.sh, minisign.pub, linux/, arch/, windows/
scripts/           nascer-app.ps1 (renomeação paramétrica do molde)
xtask/             fmt, assets, layers, version, ico
```

Onde escrever o app: `apps/localapp/src/app.rs` (`LocalApp::page`) é a página; o resto é o frame
que a suíte compartilha. Camadas internas: crie `crates/<id>-<nome>` e registre em
`xtask/src/layers.rs` (`TABLE`) — o gate `cargo xtask layers` cobra o registro e as direções.

## Build e verificação

```
cargo build --locked                  # debug (CRAFT_FONTS_DIR=assets/craft-fonts para as fontes CJK)
cargo test --locked                   # testes
cargo xtask fmt                       # fmt --check só dos membros (libs/ fica como veio)
cargo xtask assets                    # todo asset tem linha em assets/ATTRIBUTION.md
cargo xtask layers                    # camadas de crate
packaging/windows/package.ps1         # MSI + portable (Windows; precisa de WiX v5)
packaging/linux/package.sh            # AppImage/deb/rpm/tar (Linux; precisa de nfpm/appimagetool)
```

O CI roda exatamente `cargo xtask fmt`, clippy, `cargo build --locked`, os dois gates, os testes
(Linux) e o `makepkg --printsrcinfo` (Arch). Não deixe `target/` para trás: `cargo clean` após uso
local — builds pesados são do CI.

## O que o template NÃO traz (de propósito)

- **Kits/capacidades** (editor, mídia, cofre): nascem no `local-libs` quando houver app real + segundo
  consumidor.
- **`deny.toml` + job cargo-deny**: 6 dos 10 apps têm; o local-photo (base deste molde) não. O gate
  de licença/advisory entra quando o app tiver dependências próprias para auditar.
- **Empacotamento macOS/FreeBSD/web**: fora do padrão atual de release da suíte; o app que precisar
  copia de um app que tenha.
- **CLI/MCP e servidor de controle**: o local-photo tem os dois; são domínio, não esqueleto.
- **Persistência de preferências (janela, idioma)**: o app decide o formato e onde gravar; aqui o
  idioma é o do sistema a cada start.
- **Log em arquivo, single-instance, tray, atalhos globais**: cada app adiciona quando precisa (e
  registra a dep no `local-deps`).

## Manutenção do molde

O template evolui junto com o padrão craft (`docs/planos/padrao-craft.md` no dev-notes): quando um
app paga uma lição que vale para todos (CI, packaging, updater), a mudança vem para cá primeiro, e
os apps a absorvem no próximo toque. `craft-update` é exceção: a fonte é o `local-libs` — a cópia
em `libs/craft-libs/crates/update` aqui é espelho e **nunca** se edita neste repo.

## Licença

MIT OR Apache-2.0, como o resto da suíte (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` com as licenças
de terceiros).
