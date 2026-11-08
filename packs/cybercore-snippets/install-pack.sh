#!/usr/bin/env bash
# Install official Cybercore packs into the current project.
# Files land in .cybercore/packs/, exact versions in .cybercore/packs.lock.json.
set -euo pipefail
cargo install --git https://github.com/cybercoretech/cybercore-packs --package cybercore-pack
cybercore-pack search cybercore
cybercore-pack install cybercore.themes
cybercore-pack install cybercore.fonts
cybercore-pack verify
