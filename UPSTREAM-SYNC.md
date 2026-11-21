# UPSTREAM-SYNC — Local App (template)

O que neste repo vem de fora, em que revisão, e o que fazer quando o upstream mudar.

## De onde saiu este template

A árvore nasceu do **local-photo v0.1.2** (`main` em `5d28483`), com o domínio removido: saíram os
crates de foto/RAW/HEIF, o CLI/MCP, o app wasm, os assets de câmera, o empacotamento macOS/FreeBSD/web
e os docs de foto; ficaram o esqueleto (app egui mínimo, workspace, CI, packaging Windows/Linux/Arch,
xtask, minisign, i18n, fontes). O ícone é um placeholder cinza com "?".

O template **não tem upstream próprio**: ele evolui com o padrão craft (`docs/planos/padrao-craft.md`
no dev-notes). As origens abaixo são as dos forks que ele carrega.

## Forks vendored (`libs/`)

| Árvore | Origem | Revisão | Sync |
|---|---|---|---|
| `libs/egui` | storytold/egui (fork do egui) | `6d518fc799681571963842f95b55a6b68431b15a` | skill `sync-upstream`; patches da suíte (bidi, scripting, viewport hooks) |
| `libs/winit` | storytold/winit (fork do winit) | `a517b3846ee8457d84ee4975110bfda6797c44dc` | idem |
| `libs/craft-libs` | storytold/craft-libs | base `1e6f1278` | idem, **exceto** `crates/update` (abaixo) |
| `libs/craft-libs/crates/update` | **`Anon5T4R/local-libs`** | `HASHES.update` da fonte | **fora do sync-upstream**: fork permanente consciente. Mudança vai na fonte e replica com `scripts/sync.ps1 -Mode push`; a cópia aqui é espelho e nunca se edita in place |
| `libs/winprint.rs` | alampy/winprint.rs (BSD-3) | `417ac3c` (+ `winprint-gdi` no rev antigo) | usado por `craft-libs/crates/print`; upstream vivo, patches genéricos vão por PR |
| `libs/rustls-rustcrypto` | fork da suíte | — | o `craft-update` pina este fork (o upstream alpha puxa rustls-webpki 0.102 só por `alg_id`) |
| `assets/craft-fonts` | espelho do **`Anon5T4R/local-libs`** (origem histórica: storytold/craft-fonts) | árvore conferida contra a fonte pelo `check-mirror` (sem rev própria) | build input (`CRAFT_FONTS_DIR`); mudança nas fontes vai no `local-libs` (fonte única) e replica com `sync.ps1 -Asset craft-fonts` |

Regra do repo: nada é lido de `C:\Dev\storytold` em build, teste ou release. As menções a
`storytold/*` em código, docs e atribuições são referência histórica e obrigação de licença.

## Catálogo da suíte

`craft_update::suite` (na crate espelhada) é a fonte única dos `App` de cada app. Enquanto este repo
é o template, ele declara o próprio `App` em `apps/localapp/src/suite_entry.rs`; um app nascido
adiciona a entrada no `local-libs` e passa a apontar para o catálogo (README.md, passo 6).
