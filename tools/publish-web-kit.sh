#!/usr/bin/env bash
# Mirror the live org kit (cybercoretech.github.io kit/ + data/) into
# cybercore-web-kit's dist/. The org site is what every Pages site loads, so
# the Web Kit repository ships exactly those files plus checksums.
#
#   CYBERCORE_ORG_SITE=<org checkout> CYBERCORE_WEB_KIT_REPO=<web-kit checkout> \
#     tools/publish-web-kit.sh
#
# Output depends only on the org files (build.json records the org revision
# that last touched kit/ or data/), so re-running without changes is a no-op.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ORG_SITE="${CYBERCORE_ORG_SITE:-$ROOT/../../web/cybercore-tech.github.io}"
WEB_KIT_REPO="${CYBERCORE_WEB_KIT_REPO:-$ROOT/../../web/cybercore-web-kit}"
DIST="$WEB_KIT_REPO/dist"

test -f "$ORG_SITE/kit/chooser.js" || { echo "not an org site checkout: $ORG_SITE" >&2; exit 1; }
test -d "$WEB_KIT_REPO" || { echo "missing web-kit checkout: $WEB_KIT_REPO" >&2; exit 1; }

ORG_REV="$(git -C "$ORG_SITE" log -1 --format=%H -- kit data 2>/dev/null || printf 'unknown')"
ORG_DATE="$(git -C "$ORG_SITE" log -1 --format=%cI -- kit data 2>/dev/null || printf 'unknown')"

rm -rf "$DIST/v1" "$DIST/data"
mkdir -p "$DIST/v1" "$DIST/data"
cp -a "$ORG_SITE/kit/v1/." "$DIST/v1/"
cp "$ORG_SITE/kit/contact.js" "$ORG_SITE/kit/chooser.js" "$ORG_SITE/kit/chooser.css" "$DIST/"
# Registry snapshot, without local backup files or the folder README.
rsync -a --exclude="*.backup-*" --exclude="README.md" "$ORG_SITE/data/" "$DIST/data/"

KIT_VERSION="$(jq -r '.kit_version' "$DIST/v1/webkit.json")"
jq -n \
  --arg kit_version "$KIT_VERSION" \
  --arg source_revision "$ORG_REV" \
  --arg source_date "$ORG_DATE" \
  '{
    schema_version: 1,
    kit_version: $kit_version,
    generated_at: $source_date,
    source: {
      repository: "https://github.com/cybercoretech/cybercoretech.github.io",
      revision: $source_revision,
      upstream: "https://github.com/cybercoretech/cybercore"
    },
    distribution: {
      versioned_assets: "v1/",
      contact_component: "contact.js",
      theme_chooser: "chooser.js",
      registry_snapshot: "data/",
      checksum_file: "checksums-sha256.txt"
    }
  }' > "$DIST/build.json"

(cd "$DIST" && find contact.js chooser.js chooser.css v1 data -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum > checksums-sha256.txt)

printf 'Cybercore Web Kit mirrored into %s (org %s)\n' "$DIST" "$ORG_REV"
