# ✨ Cybercore Fonts

Source for the `cybercore.fonts` pack. `tools/export-distribution.py packs`
copies everything here (except this README) into cybercore-packs.

| Family | Role | License |
| --- | --- | --- |
| Oxanium (variable 200–800) | display | OFL 1.1 |
| Orbitron (variable 400–900) | headline | OFL 1.1 |
| Audiowide | logo | OFL 1.1 |
| Share Tech Mono | mono | OFL 1.1 |

`fonts.css` declares the `@font-face` rules and `--cc-font-*` variables.
Font files come unmodified from [google/fonts](https://github.com/google/fonts);
keep each family's `OFL.txt` next to its files. To add a family, add its
folder and list it in `fonts.json`; the export fails if a listed file is
missing.
