import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import build_brand_avatars  # noqa: E402

B = ROOT / "packs" / "cybercore-brand"


class BrandSourcesTest(unittest.TestCase):
    def test_layout_and_counts(self):
        self.assertEqual(len(list((B / "projects").glob("*.svg"))), 51)
        self.assertEqual(len(list((B / "tiles").glob("*.svg"))), 12)
        for size in (16, 32, 48, 64, 96, 128, 192, 256, 384, 512):
            self.assertTrue((B / f"icons/symbol-{size}.svg").exists())
            self.assertTrue((B / f"icons/symbol-{size}.png").exists())
        for variant in ("panel", "round", "mono"):
            self.assertTrue((B / f"avatars/{variant}.svg").exists())
            for size in (64, 128, 256, 512):
                self.assertTrue((B / f"avatars/{variant}-{size}.png").exists())

    def test_projects_json_matches_files(self):
        data = json.loads((B / "projects.json").read_text())
        self.assertEqual(data["format"], "cybercore-brand-projects")
        repos = [p["repo"] for p in data["projects"]]
        self.assertEqual(repos, sorted(repos))
        for p in data["projects"]:
            self.assertTrue((B / p["icon"]).exists())
            if p["tile"]:
                self.assertTrue((B / p["tile"]).exists())
            if p["upstream"]:
                self.assertTrue(p["upstream"].startswith("https://"))

    def test_no_private_or_third_party_content(self):
        for path in B.rglob("*"):
            if path.is_file() and path.suffix in {".svg", ".json", ".md"}:
                text = path.read_text().lower()
                for token in ("darkstardevx", "/home/raven", "darkbox", "blackbox", "dsdx.code@", "19244308+", "omarchy-logo", "hyprland"):
                    self.assertNotIn(token, text, f"{path}: {token}")

    def test_avatars_are_deterministic(self):
        before = {p.name: p.read_bytes() for p in (B / "avatars").glob("*.svg")}
        self.assertEqual(sorted(before), ["mono.svg", "panel.svg", "round.svg"])
        build_brand_avatars.main()
        self.assertEqual(before, {p.name: p.read_bytes() for p in (B / "avatars").glob("*.svg")})


if __name__ == "__main__":
    unittest.main()
