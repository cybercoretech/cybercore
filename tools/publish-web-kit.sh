#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_KIT_REPO="${CYBERCORE_WEB_KIT_REPO:-$ROOT/../../web/cybercore-web-kit}"
REGISTRY_ROOT="${CYBERCORE_REGISTRY_ROOT:-$ROOT/../../web/cybercore-tech.github.io/data}"
DIST="$WEB_KIT_REPO/dist"
VERSION_DIR="$DIST/v1"
SOURCE_REV="$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || printf 'unknown')"
SOURCE_DATE="$(git -C "$ROOT" log -1 --format=%cI 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)"

test -d "$WEB_KIT_REPO"
test -d "$REGISTRY_ROOT"

rm -rf "$VERSION_DIR" "$DIST/data"
mkdir -p "$VERSION_DIR" "$DIST/data"

"$ROOT/tools/build-web-kit.sh" "$VERSION_DIR"
cp "$ROOT/web/cybercore.js" "$VERSION_DIR/cybercore.js"
cp "$ROOT/../../web/cybercore-tech.github.io/kit/contact.js" "$DIST/contact.js"
cp -a "$REGISTRY_ROOT"/. "$DIST/data/"

KIT_VERSION="$(jq -r '.kit_version' "$VERSION_DIR/webkit.json")"
jq -n \
  --arg kit_version "$KIT_VERSION" \
  --arg source_revision "$SOURCE_REV" \
  --arg source_date "$SOURCE_DATE" \
  --arg registry_root "data/" \
  '{
    schema_version: 1,
    kit_version: $kit_version,
    generated_at: $source_date,
    source: {
      repository: "https://github.com/cybercoretech/cybercore",
      revision: $source_revision
    },
    distribution: {
      versioned_assets: "v1/",
      contact_component: "contact.js",
      registry_snapshot: $registry_root,
      checksum_file: "checksums-sha256.txt"
    }
  }' > "$DIST/build.json"

(cd "$DIST" && find contact.js v1 data -type f -print0 | sort -z | xargs -0 sha256sum > checksums-sha256.txt)

printf 'Cybercore Web Kit published at %s (source %s)\n' "$WEB_KIT_REPO" "$SOURCE_REV"
