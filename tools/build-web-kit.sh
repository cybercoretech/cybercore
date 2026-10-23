#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${1:-$ROOT/../../web/cybercore-tech.github.io/kit/v1}"

mkdir -p "$DEST"
cp "$ROOT/web/cybercore.js" "$DEST/cybercore.js"
cp "$ROOT/css/cybertokens.css" "$DEST/tokens.css"
cp "$ROOT/css/cybercomponents.css" "$DEST/components.css"
cp "$ROOT/web/effects.css" "$DEST/effects.css"
cp "$ROOT/schema/webkit.json" "$DEST/webkit.json"

printf 'Cybercore Web Kit generated at %s\n' "$DEST"
