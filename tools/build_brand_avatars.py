#!/usr/bin/env python3
"""Generate the Cybercore avatar SVGs from packs/cybercore-brand/logos/symbol.svg.

Each avatar is a 512x512 SVG nesting the symbol at 72% size, centred. Output is
byte-identical on every run. Standard library only.
"""
from __future__ import annotations

import re
from pathlib import Path

BRAND = Path(__file__).resolve().parents[1] / "packs" / "cybercore-brand"
SIZE = 512
SCALE = 0.72
HEX = re.compile(r"#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b")
# The symbol's own hexagon backing; dropped from the mono variant, where every
# colour becomes one tone and the backing would swallow the glyph.
BACKING = re.compile(r'\s*<path\b[^>]*(?:fill="#08111B"|stroke="#173043")[^>]*/>', re.I)
MONO_INK = "#F5F3FF"
DARK = "#08111B"


def _split_symbol(svg: str) -> tuple[str, str]:
    """Return (viewBox, inner markup) of the symbol, outer <svg> stripped."""
    open_tag = re.search(r"<svg\b[^>]*>", svg)
    view_box = re.search(r'viewBox="([^"]+)"', open_tag.group(0)) if open_tag else None
    if not open_tag or not view_box:
        raise ValueError("symbol.svg needs an <svg> element with a viewBox")
    inner = svg[open_tag.end():svg.rindex("</svg>")]
    return view_box.group(1), inner.strip("\n")


def _avatar(background: str, view_box: str, inner: str) -> str:
    size = SIZE * SCALE
    offset = (SIZE - size) / 2
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 {SIZE} {SIZE}">\n'
        f"  {background}\n"
        f'  <svg x="{offset:g}" y="{offset:g}" width="{size:g}" height="{size:g}" viewBox="{view_box}">\n'
        f"{inner}\n"
        "  </svg>\n"
        "</svg>\n"
    )


def build_all(symbol_svg: str) -> dict[str, str]:
    view_box, inner = _split_symbol(symbol_svg)
    mono_inner = HEX.sub(MONO_INK, BACKING.sub("", inner))
    return {
        "panel": _avatar(f'<rect width="{SIZE}" height="{SIZE}" rx="64" fill="#1B1236"/>', view_box, inner),
        "round": _avatar(
            f'<circle cx="{SIZE // 2}" cy="{SIZE // 2}" r="{SIZE // 2 - 6}" fill="{DARK}" stroke="#22E6FF" stroke-width="12"/>',
            view_box, inner),
        "mono": _avatar(f'<rect width="{SIZE}" height="{SIZE}" fill="{DARK}"/>', view_box, mono_inner),
    }


def main() -> int:
    symbol = (BRAND / "logos" / "symbol.svg").read_text()
    out = BRAND / "avatars"
    out.mkdir(parents=True, exist_ok=True)
    for name, svg in build_all(symbol).items():
        (out / f"{name}.svg").write_text(svg, newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
