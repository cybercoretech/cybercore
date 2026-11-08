# 🚚 Distribution: edit once, publish everywhere

cybercore is the **only** place to edit the shared Cybercore look. Every
published copy is generated from this repository:

```text
cybercore (this repo)
 ├─ schema/themes/**, schema/cybergrid.json,      ──►  cybercoretech.github.io
 │  schema/mashups.json, schema/kitty-themes.json       data/themes/**, data/*.json
 ├─ web/cybercore.js, web/effects.css, css/*.css, ──►  kit/v1/*
 │  schema/webkit.json
 ├─ web/chooser.js, web/chooser.css               ──►  kit/chooser.*
 │                                                        │  (every Pages site loads
 │                                                        │   these live, ~10 min cache)
 │                                                        ▼
 │                                                  cybercore-web-kit dist/ (mirror)
 └─ packs/** (+ schema, web, css, assets)          ──►  cybercore-packs (8 cybercore.* packs)
```

## 🔁 How a change travels

1. Open a PR here. `Distribution export` CI checks that every export builds,
   is deterministic, passes the org registry rules and that the Rust snippets
   compile.
2. Merge it.
3. Within 6 hours (or right away: Actions → *Sync from cybercore* → Run
   workflow) the org site and cybercore-packs each get a sync PR.
4. Merge the org site PR: all Pages sites show the change within ~10 minutes.
   cybercore-web-kit then gets its own sync PR (*Sync from org kit*).
5. Native apps get theme changes through a cybercore crate release
   (`docs/releasing.md`).

Each sync workflow lives in the publishing repository, pulls cybercore over
HTTPS (it is public) and uses that repository's own `GITHUB_TOKEN`, so no
cross-repository secret is needed. A sync PR is refreshed in place while it
is open and is only pushed when its content changes.

> One-time setting per publishing repository: **Settings → Actions → General
> → Allow GitHub Actions to create and approve pull requests**. Without it
> the sync still pushes its branch and the run summary links to open the PR.

## 🧰 Tools

| Command | What it does |
| --- | --- |
| `tools/export-distribution.py org <org-site>` | Writes kit/v1, kit/chooser.*, data/themes, data/cybergrid.json, data/mashups.json, data/kitty-themes.json. Removes palettes deleted here. Keeps the site's default theme (`active`) and org-owned files (contact, pages). |
| `tools/export-distribution.py packs <packs-repo>` | Rebuilds the 8 official packs listed in `packs/packs.json`, their `pack.json` checksums and `registry/index.json`. Changed packs get the next patch version. |
| `tools/export-distribution.py check` | Dry run of both into a temp dir; fails on errors or non-deterministic output. |
| `tools/publish-web-kit.sh` | Mirrors the org site's kit/ and data/ into cybercore-web-kit `dist/` with checksums. |
| `tools/open-sync-pr.sh` | Shared by the sync workflows: commit to a sync branch, push if changed, open or refresh one PR. |

## ✍️ What to edit where

| To change | Edit in cybercore |
| --- | --- |
| A theme palette, add/remove a theme | `schema/themes/<family>/<id>.json` |
| Family order in the chooser | `families` in `schema/cybergrid.json` (new themes are appended automatically) |
| Signature mashups | `schema/mashups.json` |
| The theme chooser | `web/chooser.js`, `web/chooser.css` |
| Runtime, tokens, components, effects | `web/cybercore.js`, `css/`, `web/effects.css`, `schema/webkit.json` |
| A pack's contents or version floor | `packs/<family>/`, `packs/packs.json` |
| Logos and icons | `assets/` |

Do not edit the generated copies in the org site, cybercore-web-kit or
cybercore-packs: the next sync PR would revert the change.
