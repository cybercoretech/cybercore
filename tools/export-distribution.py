#!/usr/bin/env python3
"""Export Cybercore sources to the repositories that publish them.

cybercore is the single source of truth. Every published copy is generated
from it by this script, so a change made here reaches every consumer:

    export-distribution.py org   <org-site-root>   # cybercoretech.github.io
    export-distribution.py packs <packs-repo-root> # cybercore-packs
    export-distribution.py check                   # dry run into a temp dir

`org` writes kit/v1/*, kit/chooser.*, kit/brand.js, kit/brand-vars.js,
kit/brand/** (brand.json + svg/**), data/themes/**, data/cybergrid.json,
data/mashups.json and data/kitty-themes.json. It never touches org-owned
files (kit/contact.js, data/contact.json, pages). The site's default theme
(`active` in data/cybergrid.json) is kept as the org site has it.

`packs` regenerates every official `cybercore.*` pack listed in
packs/packs.json: payload files, pack.json (paths + SHA-256) and
registry/index.json. A pack's version only changes when its payload changes.

Output depends only on the source files (no timestamps or revisions), so a
run without source changes produces no diff. Standard library only.
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / "schema"
sys.path.insert(0, str(Path(__file__).resolve().parent))
import brand_theme  # noqa: E402
PACK_SOURCES = ROOT / "packs"

# Web Kit file mapping (same as tools/build-web-kit.sh).
KIT_V1 = {
    "cybercore.js": ROOT / "web/cybercore.js",
    "tokens.css": ROOT / "css/cybertokens.css",
    "components.css": ROOT / "css/cybercomponents.css",
    "effects.css": ROOT / "web/effects.css",
    "webkit.json": SCHEMA / "webkit.json",
}
KIT_ROOT = {
    "chooser.js": ROOT / "web/chooser.js",
    "chooser.css": ROOT / "web/chooser.css",
    "brand.js": ROOT / "web/brand.js",
    "brand-vars.js": ROOT / "web/brand-vars.js",
}
DATA_FILES = {
    "mashups.json": SCHEMA / "mashups.json",
    "kitty-themes.json": SCHEMA / "kitty-themes.json",
}

KIND_BY_SUFFIX = {
    ".css": "css",
    ".js": "javascript",
    ".json": "index",
    ".svg": "svg",
    ".png": "png",
    ".ttf": "font",
    ".woff2": "font",
    ".html": "html-template",
    ".txt": "license",
    ".md": "readme",
    ".rs": "snippet",
    ".toml": "snippet",
    ".sh": "snippet",
    ".yml": "snippet",
}


class ExportError(Exception):
    pass


def dump_json(value) -> bytes:
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def load_json(path: Path):
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ExportError(f"{path.relative_to(ROOT)}: {error}") from error


def write_if_changed(path: Path, data: bytes) -> bool:
    if path.exists() and path.read_bytes() == data:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return True


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


# --------------------------------------------------------------------------
# Theme catalog
# --------------------------------------------------------------------------

def theme_files() -> dict[str, tuple[str, Path]]:
    """theme id -> (family, file). Family is the directory under schema/themes."""
    themes: dict[str, tuple[str, Path]] = {}
    for path in sorted((SCHEMA / "themes").glob("*/*.json")):
        theme_id, family = path.stem, path.parent.name
        if theme_id in themes:
            raise ExportError(f"duplicate theme id {theme_id!r}")
        load_json(path)  # must parse
        themes[theme_id] = (family, path)
    if not themes:
        raise ExportError("no themes found under schema/themes")
    return themes


def ordered_families(themes: dict[str, tuple[str, Path]]) -> dict[str, list[str]]:
    """Families in the curated order of schema/cybergrid.json.

    Listed themes keep their position, unlisted themes are appended sorted,
    removed themes are dropped, and new families are appended sorted.
    """
    by_family: dict[str, set[str]] = {}
    for theme_id, (family, _) in themes.items():
        by_family.setdefault(family, set()).add(theme_id)

    curated = load_json(SCHEMA / "cybergrid.json").get("families", {})
    result: dict[str, list[str]] = {}
    for family, listed in curated.items():
        present = by_family.get(family, set())
        ids = [t for t in listed if t in present]
        ids += sorted(present - set(ids))
        if ids:
            result[family] = ids
    for family in sorted(set(by_family) - set(result)):
        result[family] = sorted(by_family[family])
    return result


def cybergrid_document(themes, active: str | None) -> dict:
    meta = load_json(SCHEMA / "cybergrid.json")
    wallpapers = meta.get("wallpaper_options", {})
    unknown = sorted(set(wallpapers) - set(themes))
    if unknown:
        raise ExportError(f"wallpaper_options names unknown themes: {unknown[:5]}")
    active = active or meta["active"]
    if active not in themes:
        raise ExportError(f"default theme {active!r} is not in the catalog")
    return {
        "schema_version": meta["schema_version"],
        "active": active,
        "paths": meta["paths"],
        "families": ordered_families(themes),
        "wallpaper_options": wallpapers,
    }


# --------------------------------------------------------------------------
# org site
# --------------------------------------------------------------------------

def export_org(target: Path) -> list[str]:
    if not target.is_dir():
        raise ExportError(f"{target} is not a directory")
    changed: list[str] = []

    def put(rel: str, data: bytes):
        if write_if_changed(target / rel, data):
            changed.append(rel)

    for name, source in KIT_V1.items():
        put(f"kit/v1/{name}", source.read_bytes())
    for name, source in KIT_ROOT.items():
        put(f"kit/{name}", source.read_bytes())
    for name, source in DATA_FILES.items():
        load_json(source)
        put(f"data/{name}", source.read_bytes())

    themes = theme_files()
    wanted = {f"data/themes/{family}/{tid}.json" for tid, (family, _) in themes.items()}
    themes_dir = target / "data/themes"
    if themes_dir.is_dir():
        for stale in sorted(themes_dir.rglob("*.json")):
            rel = stale.relative_to(target).as_posix()
            if rel not in wanted:
                stale.unlink()
                changed.append(f"{rel} (removed)")
        for empty in sorted(themes_dir.glob("*"), reverse=True):
            if empty.is_dir() and not any(empty.iterdir()):
                empty.rmdir()
    for tid, (family, path) in themes.items():
        put(f"data/themes/{family}/{tid}.json", path.read_bytes())

    brand = {k: v for k, v in payload_brand().items() if k == "brand.json" or k.startswith("svg/")}
    brand_dir = target / "kit/brand"
    if brand_dir.is_dir():
        for stale in sorted(brand_dir.rglob("*")):
            if stale.is_file():
                rel = stale.relative_to(brand_dir).as_posix()
                if rel not in brand:
                    stale.unlink()
                    changed.append(f"kit/brand/{rel} (removed)")
        for empty in sorted(brand_dir.rglob("*"), reverse=True):
            if empty.is_dir() and not any(empty.iterdir()):
                empty.rmdir()
    for rel, data in brand.items():
        put(f"kit/brand/{rel}", data)

    grid_path = target / "data/cybergrid.json"
    active = None
    if grid_path.exists():
        active = json.loads(grid_path.read_text()).get("active")
        if active not in themes:
            active = None
    put("data/cybergrid.json", dump_json(cybergrid_document(themes, active)))
    return changed


# --------------------------------------------------------------------------
# packs
# --------------------------------------------------------------------------

def svg_title(data: bytes, fallback: str) -> str:
    match = re.search(rb"<title[^>]*>([^<]+)</title>", data)
    return match.group(1).decode().strip() if match else fallback


def payload_themes() -> dict[str, bytes]:
    themes = theme_files()
    families = ordered_families(themes)
    catalog = {
        "format": "cybercore-theme-catalog",
        "schema_version": load_json(SCHEMA / "cybergrid.json")["schema_version"],
        "theme_count": len(themes),
        "families": families,
        "themes": {tid: load_json(themes[tid][1]) for ids in families.values() for tid in ids},
    }
    return {
        "catalog.json": dump_json(catalog),
        "mashups.json": (SCHEMA / "mashups.json").read_bytes(),
        "kitty-themes.json": (SCHEMA / "kitty-themes.json").read_bytes(),
    }


def payload_effects() -> dict[str, bytes]:
    webkit = load_json(SCHEMA / "webkit.json")
    files = {
        "effects.css": (ROOT / "web/effects.css").read_bytes(),
        "recipes.json": dump_json({
            "format": "cybercore-effect-recipes",
            "kit_version": webkit["kit_version"],
            "motion_profiles": webkit["motion_profiles"],
            "effects": webkit["effects"],
        }),
    }
    for manifest in sorted((PACK_SOURCES / "cybercore-effects").glob("*/manifest.json")):
        effect_dir = manifest.parent
        for item in sorted(effect_dir.iterdir()):
            if item.is_file() and item.name != "manifest.json":
                files[f"{effect_dir.name}/{item.name}"] = item.read_bytes()
    return files


def payload_components() -> dict[str, bytes]:
    webkit = load_json(SCHEMA / "webkit.json")
    return {
        "tokens.css": (ROOT / "css/cybertokens.css").read_bytes(),
        "components.css": (ROOT / "css/cybercomponents.css").read_bytes(),
        "chooser.js": (ROOT / "web/chooser.js").read_bytes(),
        "chooser.css": (ROOT / "web/chooser.css").read_bytes(),
        "brand-vars.js": (ROOT / "web/brand-vars.js").read_bytes(),
        "components.json": dump_json({
            "format": "cybercore-component-index",
            "kit_version": webkit["kit_version"],
            "components": webkit["components"],
            "asset_variants": webkit["asset_variants"],
            "element": {
                "tag": "cybercore-theme-chooser",
                "script": "chooser.js",
                "stylesheet": "chooser.css",
            },
        }),
    }


def payload_wallpapers() -> dict[str, bytes]:
    source = PACK_SOURCES / "cybercore-images/cyberdark-wallpapers"
    manifest = load_json(source / "manifest.json")
    files: dict[str, bytes] = {}
    index = []
    for resource in manifest["resources"]:
        name = Path(resource["path"]).name
        files[name] = (source / name).read_bytes()
        index.append({
            "id": resource["id"],
            "file": name,
            "title": resource.get("title", name),
            "tags": resource.get("tags", []),
        })
    files["wallpapers.json"] = dump_json({
        "format": "cybercore-wallpaper-index",
        "wallpapers": index,
        "theme_assignments": load_json(SCHEMA / "cybergrid.json").get("wallpaper_options", {}),
    })
    return files


def payload_tree(source: Path, skip=("README.md",)) -> dict[str, bytes]:
    """Every file under a hand-written pack source directory."""
    files = {}
    for item in sorted(source.rglob("*")):
        if item.is_file() and item.name not in skip:
            files[item.relative_to(source).as_posix()] = item.read_bytes()
    if not files:
        raise ExportError(f"{source.relative_to(ROOT)} has no payload files")
    return files


def payload_snippets() -> dict[str, bytes]:
    source = PACK_SOURCES / "cybercore-snippets"
    files = payload_tree(source)
    index = json.loads(files["snippets.json"])
    listed = {s["file"] for s in index["snippets"]}
    missing = sorted(listed - set(files))
    extra = sorted(set(files) - listed - {"snippets.json"})
    if missing or extra:
        raise ExportError(f"snippets.json out of sync: missing={missing} unlisted={extra}")
    return files


def payload_fonts() -> dict[str, bytes]:
    source = PACK_SOURCES / "cybercore-fonts"
    files = payload_tree(source)
    index = json.loads(files["fonts.json"])
    for font in index["fonts"]:
        for rel in [*font["files"], font["license_file"]]:
            if rel not in files:
                raise ExportError(f"fonts.json lists missing file {rel}")
    return files


def payload_svg_assets() -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    index = []
    for path in sorted((ROOT / "assets").glob("*.svg")) + sorted((ROOT / "assets/icons").glob("*.svg")):
        rel = path.relative_to(ROOT / "assets").as_posix()
        data = path.read_bytes()
        files[rel] = data
        index.append({"file": rel, "title": svg_title(data, path.stem)})
    files["assets.json"] = dump_json({"format": "cybercore-svg-index", "assets": index})
    return files


def payload_web_templates() -> dict[str, bytes]:
    return payload_tree(PACK_SOURCES / "cybercore-web-templates")


BRAND_GROUPS = {
    "logos": "logo", "marks": "mark", "icons": "icon", "projects": "project", "tiles": "tile",
    "headers": "header", "cards": "card", "identity": "identity", "avatars": "avatar",
}

BRAND_README = """# Cybercore Brand Kit

Logos, marks, icons, project icons, tiles, headers, cards, identity and avatars.

- `svg/<group>/<name>.svg` recolours by role: every brand colour is a CSS variable
  (`var(--cc-<slot>, <original>)`), so the file follows a theme and falls back to the source colours.
- `fixed/<group>/<name>.svg` is the unmodified source.
- `png/<group>/<name>.png` are raster exports.
- `brand.json` lists every asset with its group, title, files and project.
"""


def payload_brand() -> dict[str, bytes]:
    source = PACK_SOURCES / "cybercore-brand"
    palette = brand_theme.load_palette(source / "palette.json")
    projects = load_json(source / "projects.json")["projects"]
    by_icon = {Path(p["icon"]).as_posix(): p for p in projects}
    files: dict[str, bytes] = {"README.md": BRAND_README.encode()}
    assets = []

    for folder, group in BRAND_GROUPS.items():
        directory = source / folder
        svgs: dict[str, dict] = {}
        pngs: list[tuple[str, str]] = []
        for path in sorted(directory.glob("*")):
            if path.suffix == ".svg":
                data = path.read_bytes()
                name = path.stem
                files[f"fixed/{group}/{name}.svg"] = data
                files[f"svg/{group}/{name}.svg"] = brand_theme.themeable_svg(
                    data.decode(), palette).encode()
                title = svg_title(data, name)
                project = None
                if group == "project":
                    entry = by_icon.get(f"{folder}/{path.name}")
                    if entry is None:
                        raise ExportError(f"projects.json has no entry for {folder}/{path.name}")
                    title, project = entry["title"], entry["repo"]
                svgs[name] = {
                    "id": f"{group}.{name}", "group": group, "name": name, "title": title,
                    "svg": f"svg/{group}/{name}.svg", "fixed": f"fixed/{group}/{name}.svg",
                    "png": [], "project": project,
                }
            elif path.suffix == ".png":
                files[f"png/{group}/{path.stem}.png"] = path.read_bytes()
                pngs.append((path.stem, f"png/{group}/{path.stem}.png"))
        standalone: dict[str, dict] = {}
        for stem, rel in pngs:
            base = re.sub(r"-\d+$", "", stem)
            target = svgs.get(stem) or svgs.get(base)
            if target is None:
                target = standalone.setdefault(stem, {
                    "id": f"{group}.{stem}", "group": group, "name": stem, "title": stem,
                    "svg": None, "fixed": None, "png": [], "project": None,
                })
            target["png"].append(rel)
        for asset in [*svgs.values(), *standalone.values()]:
            asset["png"].sort(key=lambda r: (int(m.group(1)) if (m := re.search(r"-(\d+)\.png$", r)) else 0, r))
            assets.append(asset)

    missing = {p["repo"] for p in projects} - {a["project"] for a in assets if a["project"]}
    if missing:
        raise ExportError(f"projects.json lists repos without an icon: {sorted(missing)}")
    assets.sort(key=lambda a: a["id"])
    files["brand.json"] = dump_json({
        "format": "cybercore-brand-index", "version": 1,
        "slots": list(brand_theme.BRAND_SLOTS), "assets": assets,
    })
    return files


PAYLOADS = {
    "themes": payload_themes,
    "effects": payload_effects,
    "components": payload_components,
    "cyberdark-wallpapers": payload_wallpapers,
    "snippets": payload_snippets,
    "fonts": payload_fonts,
    "svg-assets": payload_svg_assets,
    "web-templates": payload_web_templates,
    "brand": payload_brand,
}


def version_tuple(version: str) -> tuple[int, int, int]:
    major, minor, patch = (int(part) for part in version.split("."))
    return major, minor, patch


def next_version(floor: str, previous: dict | None, resources: list[dict]) -> str:
    if previous is None:
        return floor
    old = [(r["path"], r["sha256"]) for r in previous.get("resources", [])]
    new = [(r["path"], r["sha256"]) for r in resources]
    current = previous["version"]
    if old == new and version_tuple(current) >= version_tuple(floor):
        return current
    if version_tuple(floor) > version_tuple(current):
        return floor
    major, minor, patch = version_tuple(current)
    return f"{major}.{minor}.{patch + 1}"


def resource_id(prefix: str, rel: str) -> str:
    # Keep the extension: chooser.js and chooser.css are separate resources.
    return f"{prefix}.{rel.replace('/', '.')}"


def export_packs(target: Path) -> list[str]:
    config = load_json(PACK_SOURCES / "packs.json")
    registry_path = target / "registry/index.json"
    registry = json.loads(registry_path.read_text()) if registry_path.exists() else {
        "schema_version": 1, "packs": []}
    changed: list[str] = []
    entries = []

    for pack in config["packs"]:
        short = pack["id"].split(".", 1)[1]
        rel_dir = f"packs/cybercore/{short}"
        pack_dir = target / rel_dir
        files = PAYLOADS[short]()

        resources = []
        for rel, data in sorted(files.items()):
            resources.append({
                "id": resource_id(pack["resource_prefix"], rel),
                "kind": KIND_BY_SUFFIX.get(Path(rel).suffix, "file"),
                "path": f"{rel_dir}/{rel}",
                "sha256": sha256(data),
            })
        if len({r["id"] for r in resources}) != len(resources):
            raise ExportError(f"{pack['id']}: duplicate resource ids")

        manifest_path = pack_dir / "pack.json"
        previous = json.loads(manifest_path.read_text()) if manifest_path.exists() else None
        version = next_version(pack["version"], previous, resources)

        # Payload: remove files that are no longer produced, then write.
        if pack_dir.is_dir():
            for old in sorted(pack_dir.rglob("*"), reverse=True):
                rel = old.relative_to(pack_dir).as_posix()
                if old.is_file() and rel != "pack.json" and rel not in files:
                    old.unlink()
                    changed.append(f"{rel_dir}/{rel} (removed)")
                elif old.is_dir() and not any(old.iterdir()):
                    old.rmdir()
        for rel, data in files.items():
            if write_if_changed(pack_dir / rel, data):
                changed.append(f"{rel_dir}/{rel}")

        manifest = {
            "id": pack["id"],
            "version": version,
            "kind": pack["kind"],
            "channel": pack.get("channel", "stable"),
            "title": pack["title"],
            "description": pack["description"],
            "license": pack["license"],
            "compatibility": pack["compatibility"],
            "dependencies": [],
            "resources": resources,
            "provenance": {
                "source": "https://github.com/cybercoretech/cybercore",
                "author": "Cybercore Tech",
                "reviewed": True,
            },
        }
        if write_if_changed(manifest_path, dump_json(manifest)):
            changed.append(f"{rel_dir}/pack.json")
        entries.append({
            "id": pack["id"],
            "version": version,
            "kind": pack["kind"],
            "channel": manifest["channel"],
            "manifest": f"{rel_dir}/pack.json",
            "archive": None,
        })

    official = {e["id"] for e in entries}
    others = [p for p in registry.get("packs", []) if p["id"] not in official]
    registry = {"schema_version": registry.get("schema_version", 1), "packs": entries + others}
    if write_if_changed(registry_path, dump_json(registry)):
        changed.append("registry/index.json")
    return changed


# --------------------------------------------------------------------------

def check() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        org, packs = Path(tmp, "org"), Path(tmp, "packs")
        org.mkdir()
        packs.mkdir()
        org_files = export_org(org)
        pack_files = export_packs(packs)
        # Second run must be a no-op (deterministic output).
        if export_org(org) or export_packs(packs):
            raise ExportError("export is not deterministic: second run changed files")
        registry = json.loads((packs / "registry/index.json").read_text())
        for entry in registry["packs"]:
            manifest = json.loads((packs / entry["manifest"]).read_text())
            for resource in manifest["resources"]:
                data = (packs / resource["path"]).read_bytes()
                if sha256(data) != resource["sha256"]:
                    raise ExportError(f"checksum mismatch: {resource['path']}")
        print(f"org site: {len(org_files)} files; packs: {len(pack_files)} files "
              f"in {len(registry['packs'])} packs; deterministic: yes")
        shutil.rmtree(tmp, ignore_errors=True)


def main(argv: list[str]) -> int:
    try:
        if argv[1:2] == ["check"]:
            check()
        elif len(argv) == 3 and argv[1] in {"org", "packs"}:
            target = Path(argv[2]).resolve()
            changed = (export_org if argv[1] == "org" else export_packs)(target)
            for rel in changed:
                print(rel)
            print(f"{len(changed)} file(s) changed in {target}", file=sys.stderr)
        else:
            print(__doc__, file=sys.stderr)
            return 2
    except ExportError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
