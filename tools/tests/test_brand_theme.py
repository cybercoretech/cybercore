"""Tests for the role-based SVG colour transform (tools/brand_theme.py)."""
import re
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

from brand_theme import load_palette, slot_for, themeable_svg, unvar  # noqa: E402

BRAND = ROOT / "packs" / "cybercore-brand"
P = load_palette(BRAND / "palette.json")

_ATTRS = "fill|stroke|stop-color|flood-color|lighting-color|color"
_TAG = re.compile(r"<[A-Za-z][^<>]*>")
_ATTR = re.compile(r"""\s(%s)=(["'])(#[0-9a-fA-F]{3}|#[0-9a-fA-F]{6})\2""" % _ATTRS)
_STYLE = re.compile(r"""\sstyle=(["'])(.*?)\1""", re.S)


def normalize(svg):
    """Move themeable presentation-attribute colours into `style`, as the
    transform does, but without var(); makes comparisons placement-blind."""

    def tag(m):
        t = m.group(0)
        moved = []

        def take(a):
            if slot_for(a.group(3), P) is None:
                return a.group(0)
            moved.append("%s:%s" % (a.group(1), a.group(3)))
            return ""

        t = _ATTR.sub(take, t)
        if not moved:
            return t
        s = _STYLE.search(t)
        if s:
            existing = s.group(2).strip().rstrip(";").strip()
            parts = ([existing] if existing else []) + moved
            return t[: s.start()] + ' style="%s"' % ";".join(parts) + t[s.end():]
        end = len(t) - (2 if t.endswith("/>") else 1)
        return t[:end] + ' style="%s"' % ";".join(moved) + t[end:]

    return _TAG.sub(tag, svg)


class BrandThemeTest(unittest.TestCase):
    def test_explicit_and_nearest_slots(self):
        self.assertEqual(slot_for("#22e6ff", P), "cyan")
        self.assertEqual(slot_for("#12E5DE", P), "cyan")
        self.assertEqual(slot_for("#2BE0FF", P), "cyan")
        self.assertIsNone(slot_for("#C68642", P))

    def test_rewrites_presentation_attributes_into_style(self):
        out = themeable_svg('<svg><rect fill="#22E6FF" stroke="#FF2D9E"/></svg>', P)
        self.assertNotIn('fill="', out)
        self.assertIn("fill:var(--cc-brand-cyan, #22E6FF)", out)
        self.assertIn("stroke:var(--cc-brand-hot-pink, #FF2D9E)", out)

    def test_rewrites_style_blocks_and_short_hex(self):
        src = '<svg><style>.a{fill:#fff}</style><stop stop-color="#b44dff"/><path style="fill:#48ff52;opacity:.5"/></svg>'
        out = themeable_svg(src, P)
        self.assertIn("fill:var(--cc-brand-white, #fff)", out)
        self.assertIn("stop-color:var(--cc-brand-purple, #b44dff)", out)
        self.assertIn("fill:var(--cc-brand-acid-green, #48ff52);opacity:.5", out)

    @unittest.skipUnless(
        BRAND.joinpath("logos/symbol.svg").exists(),
        "brand logos arrive in Task 2",
    )
    def test_fallbacks_restore_source_exactly(self):
        for name in ("logos/symbol.svg", "logos/horizontal.svg"):
            src = (BRAND / name).read_text()
            self.assertEqual(normalize(unvar(themeable_svg(src, P))), normalize(src))

    def test_idempotent_and_leaves_unmapped_colours(self):
        once = themeable_svg('<svg><rect fill="#C68642"/><rect fill="#22E6FF"/></svg>', P)
        self.assertEqual(themeable_svg(once, P), once)
        self.assertIn('fill="#C68642"', once)


if __name__ == "__main__":
    unittest.main()
