"""Role-based SVG colour transform for the cybercore.brand pack.

Every literal colour that maps to a brand slot becomes
`var(--cc-brand-<slot>, <original>)`; the original stays as the fallback so an
SVG with no theme variables renders exactly like its source. Regex based (no
XML parser) so the file is never reformatted. Standard library only.
"""
from __future__ import annotations

import json
import math
import re
from dataclasses import dataclass
from pathlib import Path

BRAND_SLOTS: tuple[str, ...] = (
    "bg", "white", "panel", "line", "muted",
    "acid-green", "hot-pink", "purple", "cyan", "orange", "red",
)

_ATTRS = "fill|stroke|stop-color|flood-color|lighting-color|color"
_HEX = r"#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{3})"
_TAG = re.compile(r"<[A-Za-z][^<>]*>")
_ATTR = re.compile(r"""\s(%s)=(["'])(%s)\2""" % (_ATTRS, _HEX))
_STYLE_ATTR = re.compile(r"""\sstyle=(["'])(.*?)\1""", re.S)
_STYLE_BLOCK = re.compile(r"(<style\b[^>]*>)(.*?)(</style>)", re.S | re.I)
_RULE_BODY = re.compile(r"\{[^{}]*\}")
_VAR = r"var\(--cc-brand-[a-z-]+,\s*%s\s*\)" % _HEX
# Either an existing var() call (left alone) or a bare hex colour in a value.
_VALUE_HEX = re.compile(r"(%s)|(?<![\w#&])(?<!url\()(%s)(?![\w-])" % (_VAR, _HEX))
_UNVAR = re.compile(r"var\(--cc-brand-[a-z-]+,\s*(%s)\s*\)" % _HEX)


@dataclass(frozen=True)
class Palette:
    slots: dict[str, str]
    colors: dict[str, str | None]
    max_distance: float


def _expand(hex_color: str) -> str:
    h = hex_color.strip().lstrip("#").upper()
    if len(h) == 3:
        h = "".join(c * 2 for c in h)
    return "#" + h


def _rgb(hex_color: str) -> tuple[int, int, int]:
    h = _expand(hex_color)
    return int(h[1:3], 16), int(h[3:5], 16), int(h[5:7], 16)


def load_palette(path: Path) -> Palette:
    data = json.loads(Path(path).read_text())
    slots = {k: _expand(v) for k, v in data["slots"].items()}
    colors: dict[str, str | None] = {_expand(k): v for k, v in data.get("colors", {}).items()}
    for slot, value in slots.items():
        colors.setdefault(value, slot)
    return Palette(slots=slots, colors=colors, max_distance=float(data["max_distance"]))


def slot_for(hex_color: str, palette: Palette) -> str | None:
    key = _expand(hex_color)
    if key in palette.colors:
        return palette.colors[key]
    rgb = _rgb(key)
    best, best_d = None, None
    for slot in BRAND_SLOTS:
        if slot not in palette.slots:
            continue
        d = math.dist(rgb, _rgb(palette.slots[slot]))
        if best_d is None or d < best_d:
            best, best_d = slot, d
    if best_d is not None and best_d <= palette.max_distance:
        return best
    return None


def _var(slot: str, original: str) -> str:
    return "var(--cc-brand-%s, %s)" % (slot, original)


def _theme_values(text: str, palette: Palette) -> str:
    def sub(m: re.Match) -> str:
        if m.group(1):
            return m.group(1)
        slot = slot_for(m.group(2), palette)
        return _var(slot, m.group(2)) if slot else m.group(2)

    return _VALUE_HEX.sub(sub, text)


def themeable_svg(svg: str, palette: Palette) -> str:
    def tag(m: re.Match) -> str:
        t = m.group(0)
        moved: list[str] = []

        def take(a: re.Match) -> str:
            slot = slot_for(a.group(3), palette)
            if slot is None:
                return a.group(0)
            moved.append("%s:%s" % (a.group(1), _var(slot, a.group(3))))
            return ""

        t = _ATTR.sub(take, t)
        if not moved:
            return t
        s = _STYLE_ATTR.search(t)
        if s:
            existing = s.group(2).strip().rstrip(";").strip()
            parts = ([existing] if existing else []) + moved
            return t[: s.start()] + ' style="%s"' % ";".join(parts) + t[s.end():]
        end = len(t) - (2 if t.endswith("/>") else 1)
        return t[:end] + ' style="%s"' % ";".join(moved) + t[end:]

    out = _TAG.sub(tag, svg)

    def style_attr(m: re.Match) -> str:
        return " style=%s%s%s" % (m.group(1), _theme_values(m.group(2), palette), m.group(1))

    out = _STYLE_ATTR.sub(style_attr, out)

    def style_block(m: re.Match) -> str:
        body = _RULE_BODY.sub(lambda r: _theme_values(r.group(0), palette), m.group(2))
        return m.group(1) + body + m.group(3)

    return _STYLE_BLOCK.sub(style_block, out)


def unvar(svg: str) -> str:
    return _UNVAR.sub(lambda m: m.group(1), svg)
