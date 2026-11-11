"""Tests for the cybercore.brand payload and its pack registration."""
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
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

    def test_export_org_writes_kit_brand(self):
        with tempfile.TemporaryDirectory() as tmp:
            changed = ED.export_org(Path(tmp))
            for rel in ("kit/brand.js", "kit/brand-vars.js", "kit/brand/brand.json", "kit/brand/svg/logo/symbol.svg"):
                self.assertIn(rel, changed)
            stale = Path(tmp, "kit/brand/svg/logo/old.svg")
            stale.write_text("<svg/>")
            self.assertIn("kit/brand/svg/logo/old.svg (removed)", ED.export_org(Path(tmp)))
            self.assertEqual(ED.export_org(Path(tmp)), [])  # deterministic

    # --- I1: per-asset id and font isolation -------------------------------

    def _svg_files(self, prefix):
        return {rel: data.decode() for rel, data in ED.payload_brand().items()
                if rel.startswith(prefix) and rel.endswith(".svg")}

    def test_ids_never_repeat_across_exported_svgs(self):
        for prefix in ("svg/", "fixed/"):
            seen = {}
            for rel, text in self._svg_files(prefix).items():
                for m in re.finditer(r"""<[A-Za-z][^<>]*?\sid\s*=\s*(["'])([^"']+)\1""", text):
                    self.assertNotIn(m.group(2), seen, f"{rel} repeats id {m.group(2)} from {seen.get(m.group(2))}")
                    seen[m.group(2)] = rel

    def test_every_local_reference_resolves_in_the_same_file(self):
        for rel, text in self._svg_files("svg/").items():
            ids = set(re.findall(r"""\sid\s*=\s*["']([^"']+)["']""", text))
            refs = re.findall(r"""url\(\s*["']?#([^)"'\s]+)""", text)
            refs += re.findall(r"""href\s*=\s*["']#([^"']+)["']""", text)
            for attr in re.findall(r"""aria-(?:labelledby|describedby)\s*=\s*["']([^"']*)["']""", text):
                refs += attr.split()
            for ref in refs:
                self.assertIn(ref, ids, f"{rel}: #{ref} does not resolve")
            group, name = rel[4:-4].split("/")
            for i in ids:
                self.assertTrue(i.startswith(f"{group}-{name}-"), f"{rel}: id {i} is not prefixed")

    def test_embedded_font_faces_use_asset_unique_families(self):
        families = {}
        for rel, text in {**self._svg_files("svg/"), **self._svg_files("fixed/")}.items():
            for face in re.findall(r"@font-face\s*\{[^}]*\}", text):
                fam = re.search(r"""font-family\s*:\s*(["']?)([^;"'}]+)\1""", face).group(2).strip()
                self.assertTrue(fam.startswith("cc-"), f"{rel}: @font-face {fam!r} is not prefixed")
                group, name = rel.split("/", 1)[1][:-4].split("/")
                self.assertTrue(fam.startswith(f"cc-{group}-{name}-"), f"{rel}: {fam!r}")
                families.setdefault(fam, set()).add(rel.split("/", 1)[1])
        self.assertTrue(families, "no embedded fonts found")
        for fam, rels in families.items():
            self.assertEqual(len(rels), 1, f"{fam} declared by {rels}")

    def test_isolated_fonts_are_used_where_they_were_declared(self):
        files = self._svg_files("svg/")
        core = files["svg/header/core.svg"]
        neon = files.get("svg/site/header-neon.svg", "")
        checks = [
            ("core text rule", "text{font-family:'cc-header-core-jetbrains-mono',Consolas" in core),
            ("core keeps no bare family", "'JetBrains Mono'" not in core),
            ("neon JBM", 'font:500 17px cc-site-header-neon-jbm,"JetBrains Mono",monospace' in neon),
            ("neon CP", "font:700 132px cc-site-header-neon-cp," in neon),
        ]
        self.assertEqual([name for name, ok in checks if not ok], [])

    def test_sources_keep_their_original_ids(self):
        src = (ROOT / "packs/cybercore-brand/logos/symbol.svg").read_text()
        self.assertIn('id="edge"', src)
        self.assertIn('aria-labelledby="title desc"', src)

    # --- I2: published README ---------------------------------------------

    def test_readme_keeps_usage_rules_and_real_variable_names(self):
        readme = ED.payload_brand()["README.md"].decode()
        source = (ROOT / "packs/cybercore-brand/README.md").read_text()
        first_heading = next(line for line in source.splitlines() if line.startswith("#"))
        self.assertTrue(readme.startswith(source))
        self.assertIn(first_heading, readme)
        self.assertIn("--cc-brand-", readme)
        self.assertNotIn("--cc-<slot>", readme)
        self.assertIn("svg/<group>/<name>.svg", readme)

    # --- I3: web-kit mirror ships kit/brand ---------------------------------

    @unittest.skipUnless(shutil.which("jq") and shutil.which("rsync"), "needs jq and rsync")
    def test_publish_web_kit_copies_and_checksums_brand(self):
        with tempfile.TemporaryDirectory() as tmp:
            org, web_kit = Path(tmp, "org"), Path(tmp, "web-kit")
            org.mkdir()
            ED.export_org(org)
            (org / "kit/contact.js").write_text("// contact\n")
            web_kit.mkdir()
            env = {**os.environ, "CYBERCORE_ORG_SITE": str(org), "CYBERCORE_WEB_KIT_REPO": str(web_kit)}
            subprocess.run(["bash", str(ROOT / "tools/publish-web-kit.sh")], env=env, check=True,
                           capture_output=True)
            dist = web_kit / "dist"
            self.assertTrue((dist / "brand/brand.json").is_file())
            self.assertTrue((dist / "brand/svg/logo/symbol.svg").is_file())
            sums = (dist / "checksums-sha256.txt").read_text()
            self.assertIn("  brand/brand.json\n", sums)
            self.assertIn("  brand/svg/logo/symbol.svg\n", sums)
            # A brand file removed from the org kit disappears from the mirror.
            (org / "kit/brand/svg/logo/symbol.svg").unlink()
            subprocess.run(["bash", str(ROOT / "tools/publish-web-kit.sh")], env=env, check=True,
                           capture_output=True)
            self.assertFalse((dist / "brand/svg/logo/symbol.svg").exists())

    # --- I4: svg-assets content lives in brand group `site` ------------------

    def test_every_svg_asset_has_a_site_brand_asset(self):
        index = json.loads(ED.payload_brand()["brand.json"])
        by_id = {a["id"]: a for a in index["assets"]}
        svg_assets = [rel for rel in ED.payload_svg_assets() if rel.endswith(".svg")]
        self.assertEqual(len(svg_assets), 21)
        for rel in svg_assets:
            stem = Path(rel).stem
            asset = by_id.get(f"site.{stem}")
            self.assertIsNotNone(asset, rel)
            self.assertEqual(asset["svg"], f"svg/site/{stem}.svg")
            self.assertEqual(asset["fixed"], f"fixed/site/{stem}.svg")
        self.assertEqual(by_id["site.cybercore-icon-32"]["png"], ["png/site/cybercore-icon-32.png"])
        self.assertIn("site.cybercore-wordmark", by_id)

    # --- M4: kit index only lists what the kit ships ------------------------

    def test_kit_brand_json_lists_only_svg_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            ED.export_org(Path(tmp))
            kit = json.loads(Path(tmp, "kit/brand/brand.json").read_text())
            for a in kit["assets"]:
                self.assertNotIn("fixed", a)
                self.assertNotIn("png", a)
                if a["svg"]:
                    self.assertTrue(Path(tmp, "kit/brand", a["svg"]).is_file(), a["svg"])
            self.assertIsNone({a["id"]: a for a in kit["assets"]}["identity.mark"]["svg"])
            pack = json.loads(ED.payload_brand()["brand.json"])
            self.assertIn("fixed", pack["assets"][0])

    def test_chooser_ships_brand_vars_everywhere(self):
        if "from './brand-vars.js'" not in (ROOT / "web/chooser.js").read_text():
            self.skipTest("chooser.js does not import brand-vars.js")
        self.assertIn("brand-vars.js", ED.payload_components())
        self.assertIn("brand-vars.js", ED.KIT_ROOT)
        self.assertIn("kit/brand-vars.js", (ROOT / "tools/publish-web-kit.sh").read_text())


if __name__ == "__main__":
    unittest.main()
