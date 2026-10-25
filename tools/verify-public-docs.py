#!/usr/bin/env python3
"""Reject private Wiki or machine-specific content from public documentation."""

from __future__ import annotations

import sys
from pathlib import Path

FORBIDDEN_MARKERS = (
    "darkstardevx",
    "/home/",
    "blackbox",
    "private wiki",
    ".local/state",
    ".ssh/",
)


def main() -> int:
    roots = [Path("docs/public-book/src")]
    if len(sys.argv) > 1:
        roots.append(Path(sys.argv[1]))
    violations: list[str] = []
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            text = path.read_text(errors="replace").lower()
            for marker in FORBIDDEN_MARKERS:
                if marker in text:
                    violations.append(f"{path}: {marker}")
    if violations:
        print("Public documentation safety check failed:")
        print("\n".join(violations))
        return 1
    print("Public documentation safety check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
