#!/usr/bin/env bash
# Maintenance script, never run in CI. Renders packs/cybercore-brand/avatars/*.svg
# to avatars/<variant>-<size>.png at 64/128/256/512 px. Needs chromium and magick.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
avatars="$root/packs/cybercore-brand/avatars"
profile="${BRAND_CHROMIUM_PROFILE:-$root/.superpowers/chromium-profile}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$profile"

for svg in "$avatars"/*.svg; do
  variant="$(basename "$svg" .svg)"
  # Render once at the master size, then downscale for the smaller sizes.
  chromium --headless --disable-gpu --no-sandbox --hide-scrollbars \
    --user-data-dir="$profile" --default-background-color=00000000 \
    --window-size=512,512 --screenshot="$work/$variant.png" "file://$svg" >/dev/null 2>&1
  for size in 64 128 256 512; do
    magick "$work/$variant.png" -filter Lanczos -resize "${size}x${size}" \
      -strip -define png:exclude-chunk=all "$avatars/$variant-$size.png"
  done
done
echo "Rendered avatar PNGs into $avatars"
