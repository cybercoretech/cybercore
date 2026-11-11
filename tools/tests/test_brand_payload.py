"""Tests for the cybercore.brand payload and its pack registration."""
import importlib.util
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tools" / "tests"))

from brand_theme import unvar  # noqa: E402
from test_brand_theme import normalize  # noqa: E402

_spec = importlib.util.spec_from_file_location("export_distribution", ROOT / "tools" / "export-distribution.py")
ED = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ED)


class BrandPayloadTest(unittest.TestCase):
    def test_payload_shape(self):
        files = ED.payload_brand()
        index = json.loads(files["brand.json"])
        self.assertEqual(index["format"], "cybercore-brand-index")
        self.assertEqual(index["slots"][:3], ["bg", "white", "panel"])
        ids = [a["id"] for a in index["assets"]]
        self.assertEqual(ids, sorted(ids))
        self.assertIn("logo.symbol", ids)
        self.assertEqual(sum(a["group"] == "project" for a in index["assets"]), 51)
        for a in index["assets"]:
            for rel in [a["svg"], a["fixed"], *a["png"]]:
                if rel:
                    self.assertIn(rel, files)

    def test_asset_naming_and_png_attachment(self):
        index = json.loads(ED.payload_brand()["brand.json"])
        by_id = {a["id"]: a for a in index["assets"]}
        for size in (16, 512):
            a = by_id[f"icon.symbol-{size}"]
            self.assertEqual(a["png"], [f"png/icon/symbol-{size}.png"])
            self.assertEqual(a["svg"], f"svg/icon/symbol-{size}.svg")
        self.assertEqual([p.split("-")[-1] for p in by_id["avatar.panel"]["png"]],
                         ["64.png", "128.png", "256.png", "512.png"])
        mark = by_id["identity.mark"]
        self.assertIsNone(mark["svg"])
        self.assertIsNone(mark["fixed"])
        self.assertEqual(mark["png"], ["png/identity/mark.png"])

    def test_project_assets_use_repo_titles(self):
        index = json.loads(ED.payload_brand()["brand.json"])
        projects = [a for a in index["assets"] if a["group"] == "project"]
        self.assertTrue(all(a["project"] and a["title"] == a["project"] for a in projects))
        self.assertTrue(all(a["project"] is None for a in index["assets"] if a["group"] != "project"))

    def test_projects_json_titles_equal_repo(self):
        data = json.loads((ROOT / "packs/cybercore-brand/projects.json").read_text())
        for p in data["projects"]:
            self.assertEqual(p["title"], p["repo"])

    def test_every_themed_svg_unvars_to_fixed(self):
        files = ED.payload_brand()
        for rel, data in files.items():
            if rel.startswith("svg/"):
                fixed = files["fixed/" + rel[4:]].decode()
                self.assertEqual(normalize(unvar(data.decode())), normalize(fixed), rel)

    def test_payload_is_deterministic(self):
        self.assertEqual(ED.payload_brand(), ED.payload_brand())

    def test_pack_registered(self):
        packs = {p["id"]: p for p in json.loads((ROOT / "packs/packs.json").read_text())["packs"]}
        b = packs["cybercore.brand"]
        self.assertEqual((b["kind"], b["version"], b["resource_prefix"], b["license"], b["title"]),
                         ("brand-pack", "1.0.0", "brand", "MIT", "Cybercore Brand Kit"))
        self.assertEqual(b["compatibility"], {"web_kit": ">=1.0.0", "control_plane": ">=0.1.0"})
        self.assertIn("Superseded by cybercore.brand; kept for existing consumers.",
                      packs["cybercore.svg-assets"]["description"])


if __name__ == "__main__":
    unittest.main()
