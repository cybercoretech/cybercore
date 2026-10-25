#!/usr/bin/env python3
"""Generate public-safe updater task pages from the public updater manifest."""

from __future__ import annotations

import json
from pathlib import Path

PAGES = [
    ("overview", "Overview"),
    ("install", "Installation"),
    ("configuration", "Configuration"),
    ("cli-tui", "CLI and TUI"),
    ("control-panel", "Control Panel"),
    ("themes-assets", "Themes and assets"),
    ("github-repository", "GitHub and repository status"),
    ("recovery", "Recovery and services"),
    ("validation", "Validation and troubleshooting"),
    ("release-status", "Release status"),
]


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    manifest = json.loads((root / "tools/cybercore-consumers.json").read_text())
    output = root / "docs/public-book/src/updater"
    tasks = manifest.get("consumers", []) + manifest.get("syncs", [])
    for task in tasks:
        name = task["name"]
        slug = name.lower().replace("_", "-")
        folder = output / slug
        folder.mkdir(parents=True, exist_ok=True)
        links = "\n".join(f"- [{label}]({slug_name}.md)" for slug_name, label in PAGES)
        (folder / "index.md").write_text(
            f"# {name}\n\nPublic operator reference for the `{name}` task.\n\n{links}\n"
        )
        for slug_name, label in PAGES:
            (folder / f"{slug_name}.md").write_text(
                f"# {name}: {label}\n\n"
                f"This public-safe page documents the `{name}` updater task. "
                "It contains no machine-specific paths, private repository notes, "
                "credentials, or personal data.\n\n"
                "Use the shared [updater operations](../../operations.md), "
                "[TUI](../../tui.md), [Control Panel](../../control-panel.md), "
                "and [recovery](../../recovery-validation.md) references for "
                "the applicable procedure.\n"
            )


if __name__ == "__main__":
    main()
