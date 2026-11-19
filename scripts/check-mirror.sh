#!/usr/bin/env bash
# Confere as crates espelhadas do local-libs (libs/craft-libs/crates/<crate>) contra a fonte única
# (github.com/Anon5T4R/local-libs, público). Compara o hash da árvore git do diretório, que é
# byte-exato (o local-libs não converte EOL): sem hash próprio, sem manifesto para manter.
#   - igual à fonte atual (main)            -> ok
#   - igual a uma versão antiga da fonte    -> aviso (espelho desatualizado: rode o sync.ps1)
#   - igual a nenhuma versão da fonte       -> erro  (alguém editou a cópia no app)
# Crates do craft-libs que não vêm do local-libs (craft-ui, craft-i18n...) não entram.
# Uso: scripts/check-mirror.sh   (na raiz do app, dentro de um checkout git)
set -euo pipefail
src_url=${LOCAL_LIBS_URL:-https://github.com/Anon5T4R/local-libs.git}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git clone -q --bare --filter=blob:none "$src_url" "$work/src.git"
# A árvore como está no disco (no CI = HEAD; no gate local inclui o que ainda não foi commitado),
# montada num índice temporário: o índice real do repositório não é tocado.
cp "$(git rev-parse --git-path index)" "$work/index" 2>/dev/null || true
GIT_INDEX_FILE="$work/index" git add -A libs/craft-libs/crates
tree=$(GIT_INDEX_FILE="$work/index" git write-tree)
fail=0
for crate in $(git --git-dir="$work/src.git" ls-tree --name-only main crates/ | sed 's#^crates/##'); do
  app_tree=$(git rev-parse -q --verify "$tree:libs/craft-libs/crates/$crate" 2>/dev/null) || continue
  head_tree=$(git --git-dir="$work/src.git" rev-parse "main:crates/$crate")
  if [ "$app_tree" = "$head_tree" ]; then
    echo "ok: $crate"
    continue
  fi
  found=""
  for c in $(git --git-dir="$work/src.git" log --format=%H main -- "crates/$crate"); do
    t=$(git --git-dir="$work/src.git" rev-parse -q --verify "$c:crates/$crate" 2>/dev/null) || continue
    if [ "$t" = "$app_tree" ]; then found=$c; break; fi
  done
  if [ -n "$found" ]; then
    echo "::warning::$crate: espelho desatualizado (igual à fonte em ${found:0:7}); rode sync.ps1 -Crate $crate -Mode push"
  else
    echo "::error::$crate: a cópia em libs/craft-libs/crates/$crate não é nenhuma versão do local-libs (editada no app?)"
    fail=1
  fi
done
exit $fail
