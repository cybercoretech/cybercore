#!/usr/bin/env python3
"""Generate the public GitHub Wiki from the canonical built-in theme catalog."""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
THEMES = ROOT / "schema" / "themes"
GALLERY_URL = "https://cybercoretech.github.io/cybercore/"
DOCS_URL = "https://cybercoretech.github.io/cybercore/docs/"
REPOSITORY_URL = "https://github.com/cybercoretech/cybercore"
ROLES = (
    "bg",
    "white",
    "acid_green",
    "hot_pink",
    "purple",
    "cyan",
    "orange",
    "red",
    "panel",
    "line",
    "muted",
)
LEGACY_PAGES = (
    "Cyberdyne‑Themes.md",
    "Cyberpunk‑Themes.md",
    "Default‑Themes.md",
    "Dystopian‑Themes.md",
    "NeoSynth‑Themes.md",
    "Palette‑Deep‑Dive.md",
    "Synthwave‑Themes.md",
    "Theme‑Gallery.md",
)


def display_name(value: str) -> str:
    return value.replace("-", " ").title()


def page_name(family: str) -> str:
    return f"{display_name(family).replace(' ', '-')}-Themes.md"


def load_catalog() -> list[dict[str, object]]:
    themes: list[dict[str, object]] = []
    for path in sorted(THEMES.glob("*/*.json")):
        palette = json.loads(path.read_text())
        missing = [role for role in ROLES if role not in palette]
        if missing:
            raise ValueError(f"{path.relative_to(ROOT)} is missing: {', '.join(missing)}")
        themes.append(
            {
                "id": path.stem,
                "family": path.parent.name,
                "palette": palette,
            }
        )
    if not themes:
        raise ValueError(f"No theme files found beneath {THEMES}")
    return themes


def hex_color(value: str) -> str:
    return f"#{value.lstrip('#').lower()}"


def render_home(total: int, families: list[str]) -> str:
    family_list = ", ".join(display_name(family) for family in families)
    return f"""# ⚡ Cybercore

Welcome to the public Cybercore gateway: shared Rust contracts, CYBERGRID
themes, design tokens, and tooling for the Cybercore Systems Framework.

## 🚀 Start here

- [Read the public documentation]({DOCS_URL})
- [Browse the live theme gallery]({GALLERY_URL})
- [Use the crate on docs.rs](https://docs.rs/cybercore)
- [Read the source repository]({REPOSITORY_URL})

## 🎨 Theme catalog

The built-in catalog contains **{total} themes** across **{len(families)}
families**: {family_list}. The gallery and palette reference below are
generated from the repository's canonical `schema/themes/` files.

- [Browse every theme](Theme-Gallery)
- [View the full palette reference](Palette-Reference)

## 🤝 Community

- [Ask questions or share ideas]({REPOSITORY_URL}/discussions)
- [Report a bug or request a feature]({REPOSITORY_URL}/issues/new/choose)
- [Read the contribution guide]({REPOSITORY_URL}/blob/main/CONTRIBUTING.md)

Please do not post credentials, personal data, private repository content, or
machine-specific operational details in public channels.

---

_Generated from the public Cybercore repository. Do not edit generated catalog
pages manually._
"""


def render_sidebar(families: list[str]) -> str:
    family_links = "\n".join(
        f"  - [{display_name(family)}](%s)" % page_name(family).removesuffix(".md")
        for family in families
    )
    return f"""## ⚡ Cybercore

- [Home](Home)
- [Public documentation]({DOCS_URL})
- [Live theme gallery]({GALLERY_URL})
- [docs.rs](https://docs.rs/cybercore)

## 🎨 Themes

- [All themes](Theme-Gallery)
- [Palette reference](Palette-Reference)
{family_links}

## 🤝 Community

- [Discussions]({REPOSITORY_URL}/discussions)
- [Issues]({REPOSITORY_URL}/issues/new/choose)
- [Contributing]({REPOSITORY_URL}/blob/main/CONTRIBUTING.md)
- [Source repository]({REPOSITORY_URL})
"""


def render_gallery(themes: list[dict[str, object]]) -> str:
    rows = [
        "| Theme | Family | Background | Accent | Gallery |",
        "|---|---|---|---|---|",
    ]
    for theme in themes:
        palette = theme["palette"]
        assert isinstance(palette, dict)
        rows.append(
            "| `{id}` | [{family}]({page}) | `{bg}` | `{accent}` | "
            "[Open gallery]({gallery}) |".format(
                id=theme["id"],
                family=display_name(str(theme["family"])),
                page=page_name(str(theme["family"])).removesuffix(".md"),
                bg=hex_color(str(palette["bg"])),
                accent=hex_color(str(palette["acid_green"])),
                gallery=GALLERY_URL,
            )
        )
    return """# 🎨 Cybercore Theme Gallery

This generated catalog lists every public built-in Cybercore theme. Open the
live gallery to browse the rendered system; theme selection is intentionally
kept in the gallery rather than encoded into undocumented URL parameters.

""" + "\n".join(rows) + "\n"


def render_palette_reference(themes: list[dict[str, object]]) -> str:
    header = "| Theme | Family | " + " | ".join(f"`{role}`" for role in ROLES) + " |"
    divider = "|" + "---|" * (len(ROLES) + 2)
    rows = [header, divider]
    for theme in themes:
        palette = theme["palette"]
        assert isinstance(palette, dict)
        colors = " | ".join(f"`{hex_color(str(palette[role]))}`" for role in ROLES)
        rows.append(
            f"| `{theme['id']}` | {display_name(str(theme['family']))} | {colors} |"
        )
    return """# 🔬 Cybercore Palette Reference

All values are generated from the public built-in theme files. Color values
are six-digit hexadecimal CYBERGRID roles.

""" + "\n".join(rows) + "\n"


def render_family(family: str, themes: list[dict[str, object]]) -> str:
    rows = [
        "| Theme | Background | Acid green | Hot pink | Cyan | Gallery |",
        "|---|---|---|---|---|---|",
    ]
    for theme in themes:
        palette = theme["palette"]
        assert isinstance(palette, dict)
        rows.append(
            "| `{id}` | `{bg}` | `{acid}` | `{pink}` | `{cyan}` | "
            "[Open gallery]({gallery}) |".format(
                id=theme["id"],
                bg=hex_color(str(palette["bg"])),
                acid=hex_color(str(palette["acid_green"])),
                pink=hex_color(str(palette["hot_pink"])),
                cyan=hex_color(str(palette["cyan"])),
                gallery=GALLERY_URL,
            )
        )
    return f"""# 🎨 {display_name(family)} Themes

This page is generated from `schema/themes/{family}/`.

""" + "\n".join(rows) + "\n"


def render_pages() -> dict[str, str]:
    catalog = load_catalog()
    by_family: dict[str, list[dict[str, object]]] = defaultdict(list)
    for theme in catalog:
        by_family[str(theme["family"])].append(theme)
    families = sorted(by_family)
    pages = {
        "Home.md": render_home(len(catalog), families),
        "_Sidebar.md": render_sidebar(families),
        "Theme-Gallery.md": render_gallery(catalog),
        "Palette-Reference.md": render_palette_reference(catalog),
    }
    for family in families:
        pages[page_name(family)] = render_family(family, by_family[family])
    return pages


def write_pages(output: Path, pages: dict[str, str]) -> None:
    output.mkdir(parents=True, exist_ok=True)
    for legacy in LEGACY_PAGES:
        (output / legacy).unlink(missing_ok=True)
    for name, content in pages.items():
        (output / name).write_text(content)


def check_pages(output: Path, pages: dict[str, str]) -> int:
    stale = [name for name in LEGACY_PAGES if (output / name).exists()]
    mismatched = [
        name
        for name, content in pages.items()
        if not (output / name).is_file() or (output / name).read_text() != content
    ]
    if stale or mismatched:
        if stale:
            print(f"Legacy generated pages remain: {', '.join(stale)}")
        if mismatched:
            print(f"Generated pages differ: {', '.join(mismatched)}")
        return 1
    print(f"GitHub Wiki is current: {len(pages)} generated pages.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path, help="Wiki checkout or output directory")
    parser.add_argument("--check", action="store_true", help="Verify output without writing files")
    args = parser.parse_args()

    pages = render_pages()
    if args.check:
        return check_pages(args.output, pages)
    write_pages(args.output, pages)
    print(f"Generated {len(pages)} GitHub Wiki pages in {args.output}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
