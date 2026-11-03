#!/usr/bin/env python3
"""Generate the public-safe static Cybercore theme chooser."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
THEMES = ROOT / "schema" / "themes"
OUT_DATA = ROOT / "docs" / "theme-chooser-data.json"

PRIVATE_FAMILIES = {"omarchy-live"}


def display_name(value: str) -> str:
    return re.sub(r"\s+", " ", value.replace("-", " ").replace("_", " ")).title()


def main() -> None:
    entries = []
    for path in sorted(THEMES.rglob("*.json")):
        family = path.parent.name
        if family in PRIVATE_FAMILIES:
            continue
        palette = json.loads(path.read_text(encoding="utf-8"))
        entries.append(
            {
                "id": path.stem,
                "name": display_name(path.stem),
                "family": family,
                "colors": {
                    key: palette[key]
                    for key in (
                        "bg",
                        "panel",
                        "line",
                        "acid_green",
                        "cyan",
                        "purple",
                        "hot_pink",
                    )
                    if key in palette
                },
            }
        )

    OUT_DATA.write_text(
        json.dumps({"count": len(entries), "themes": entries}, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
