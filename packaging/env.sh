# shellcheck shell=bash
# Shared setup for the packaging scripts. Source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports:
#   ROOT                  workspace root
#   VERSION               [workspace.package] version from Cargo.toml (override: LOCALAPP_VERSION)
#   DIST                  output directory for release artifacts (default: $ROOT/dist/release)
#   LOCALAPP_BUILD_DATE   UTC build date, YYYY-MM-DD
#   CARGO_TARGET_DIR      cargo's target dir (default: $ROOT/target)

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

# The version lives in exactly one place: `[workspace.package] version` in the root Cargo.toml.
# (`cargo xtask version` prints the same thing; awk avoids compiling xtask here.)
workspace_version() {
  awk '
    /^\[/ { in_pkg = ($0 == "[workspace.package]") ; next }
    in_pkg && $1 == "version" { gsub(/[" ]/, "", $3); print $3; exit }
  ' "$ROOT/Cargo.toml"
  return $?
}

VERSION="${LOCALAPP_VERSION:-$(workspace_version)}"
if [[ -z "$VERSION" ]]; then
  echo "error: could not read [workspace.package] version from $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export DIST

export LOCALAPP_BUILD_DATE="${LOCALAPP_BUILD_DATE:-$(date -u +%Y-%m-%d)}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# Emit a GitHub Actions warning (plain stderr outside Actions).
warn() {
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
  return 0
}

# Copy the docs that exist into a package directory, plus the embedded fonts' licence texts.
copy_docs() {
  local dest="$1" f
  for f in README.md LICENSE LICENSE-MIT LICENSE-APACHE COPYRIGHT NOTICE; do
    if [[ -f "$ROOT/$f" ]]; then cp "$ROOT/$f" "$dest/"; fi
  done
  copy_font_licences "$dest"
  return $?
}

# The craft-fonts OFL texts of the faces the build embedded (the app embeds no other font).
copy_font_licences() {
  local dest="$1" lic family
  [[ -n "${CRAFT_FONTS_DIR:-}" ]] || return 0
  for lic in "$CRAFT_FONTS_DIR"/fonts/*/OFL.txt; do
    [[ -f "$lic" ]] || continue
    family="$(basename "$(dirname "$lic")")"
    cp "$lic" "$dest/OFL-$family.txt"
  done
}

# Portable SHA-256 of a file (prints just the hash).
sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null; then sha256sum "$file" | cut -d' ' -f1; else shasum -a 256 "$file" | cut -d' ' -f1; fi
  return $?
}
