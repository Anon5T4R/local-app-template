#!/usr/bin/env bash
# Regenerate every app icon file from assets/app-icon/localapp.svg.
#
# Needs: resvg (brew install resvg / cargo install resvg). The outputs are committed, so building
# and packaging never need these tools.
#
#   packaging/icons.sh
#
# The icon is a PLACEHOLDER (grey tile, question mark): a born app replaces the SVG and the
# generated files, see assets/app-icon/README.md.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/localapp.svg"
ID="com.localapp.app"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v resvg >/dev/null || { echo "error: resvg not found (cargo install resvg)" >&2; exit 1; }

render() {
  local src="$1" size="$2" out="$3"
  resvg -w "$size" -h "$size" "$src" "$out" </dev/null
}

render "$SVG" 1024 "$DIR/localapp-1024.png"

# Linux hicolor theme (also the runtime window icon on Windows and Linux: 256x256).
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$SVG" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$SVG" "$DIR/hicolor/scalable/apps/$ID.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  render "$SVG" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/localapp.ico" "${ICO_PNGS[@]}")

echo "icons written to $DIR"
