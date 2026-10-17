# CYBERGRID theme engine

`cybercore::theme` provides the runtime contract shared by Cybercore apps.
The existing embedded palettes remain the built-in source of truth; the
engine adds portable custom themes and persists a user's selected theme and
appearance without requiring a rebuild.

## Theme document format

Version 1 preserves all eleven established CYBERGRID color roles. The
`palette` is the default/dark appearance. `variants.light` is optional; when
absent, the base palette is used for light appearance as a compatibility
fallback. `design` controls fonts, density, corners, and motion.

```json
{
  "format_version": 1,
  "metadata": {
    "id": "midnight-terminal",
    "name": "Midnight Terminal",
    "family": "custom",
    "description": "A low-glare terminal palette",
    "author": "Local user"
  },
  "palette": {
    "bg": "0e091d", "white": "ffffff", "acid_green": "c8e967",
    "hot_pink": "fd3e6a", "purple": "9147a8", "cyan": "14b9b5",
    "orange": "ff7f41", "red": "f93d3b", "panel": "171329",
    "line": "332a4a", "muted": "a39bb8"
  },
  "variants": {
    "light": {
      "bg": "f5f3fa", "white": "171329", "acid_green": "527a00",
      "hot_pink": "bf2454", "purple": "704080", "cyan": "087c79",
      "orange": "a84d1c", "red": "b32626", "panel": "ffffff",
      "line": "d5d0df", "muted": "625b70"
    }
  },
  "design": {
    "font_ui": "Inter", "font_mono": "JetBrains Mono",
    "font_display": "Oxanium", "density": "comfortable",
    "corners": "sharp", "motion": "full"
  }
}
```

IDs are lowercase letters, digits, and hyphens (1–64 characters). Palette
colors are six-digit hex values with or without `#`. Font family strings are
restricted to letters, digits, spaces, underscores, and hyphens. Density is
`compact`, `comfortable`, or `spacious`; corners is `sharp`, `soft`, or
`rounded`; motion is `full`, `reduced`, or `none`. Unsupported document
versions and invalid fields are rejected.

Old flat JSON palette files (without `format_version`, `metadata`, or
`palette`) remain readable through `ThemeDocument::from_json`; callers pass
the ID to assign to that imported palette. Theme Studio imports either shape
and exports versioned documents.

## Rust API

```rust,no_run
use cybercore::theme::{Appearance, ThemeCatalog, ThemeDocument};

let mut catalog = ThemeCatalog::load()?;
let active = catalog.get(catalog.active_id()).expect("active theme");
let css = active.document.to_css(catalog.active_appearance());

let document = ThemeDocument::from_json(&std::fs::read_to_string("theme.json")?, "my-theme")?;
document.validate()?;
let contrast = document.contrast_report(Appearance::Dark);
catalog.save(document)?;
catalog.select("my-theme")?;
catalog.set_appearance(Appearance::Light)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ThemeDocument::to_css` emits the established CYBERGRID variables and shared
design variables. `contrast_report` reports WCAG contrast ratios for body
text, accents, borders, and focus indicators. `quality_report` runs those
checks for dark and light modes and records whether light mode has its own
palette or uses the compatibility fallback. Studio shows warnings while
editing; release automation can require a strict pass:

```sh
cybercore-theme check my-theme.json --strict
cybercore-theme validate-pack my-pack.cyberpack.json
```

Strict mode fails when any checked contrast threshold is missed or the theme
has no explicit light variant. The report covers shared semantic roles; apps
remain responsible for checking their own component states.
Install the checker with the public installer (requires Rust and Cargo):

```sh
curl -fsSL https://cybercoretech.github.io/cybercore/install.sh | sh
```

The equivalent direct command is `cargo install --locked cybercore --bin
cybercore-theme`. This installs the checker CLI; use `cybercore = "0.8"` as a
Cargo dependency for the Rust library. From a checkout, run it with
`cargo run --bin cybercore-theme -- ...`.

## Shared storage and precedence

Custom theme documents live in `themes/<id>.json`, and the active selection
in `active-theme.json`. The config root is chosen in this order:

1. `CYBERCORE_CONFIG_DIR`, if set (the directory itself is the config root).
2. `$XDG_CONFIG_HOME/cybercore`.
3. `$HOME/.config/cybercore`.

`CYBERGRID_THEME` can override the stored active theme for a process when it
names a theme in the catalog. Otherwise, the stored selection wins, followed
by the schema's built-in active theme. Theme and selection writes use a
temporary file and rename to avoid exposing partially written JSON.

Built-in theme family names are embedded alongside the palette catalog, so
consumers do not need access to the Cybercore source checkout to group themes.
Custom themes may use the `family` field for organization in creator UIs.

## Hub integration

Cyberdeck Hub exposes `GET /api/cybergrid/themes`, `GET /api/cybergrid/events`,
`POST /api/cybergrid/active/:id`,
`POST /api/cybergrid/appearance/:mode`, and
`GET /api/cybergrid/css/:name?appearance=dark|light`. The Hub links to the
standalone `cybercore-theme-studio` app for authoring. Studio owns the save,
delete, and validation APIs, and writes to the same shared config catalog.

## Cybercore Theme Studio

The workspace package `cybercore-theme-studio` runs a standalone creator at
`http://127.0.0.1:8761/` by default:

```sh
cargo run -p cybercore-theme-studio
```

Studio accepts `localhost`, loopback IPs, and names under `.localhost` in the
request `Host` header. A friendly URL such as
`http://cybercore-tech.localhost:8761/` therefore works without changing the
machine's hosts file; the listener remains restricted to loopback.

It exposes its creator API on that local server: `GET /api/themes`,
`GET /api/events`,
`POST /api/themes`, `DELETE /api/themes/:id`, `POST /api/themes/validate`,
`POST /api/active/:id`, `POST /api/appearance/:mode`, and
`GET /api/css/:id?appearance=dark|light`. Saved themes are immediately
available to every app using this catalog and config directory.

The studio has a searchable theme library and app-reference previews for
Cyberdeck Hub, Dockspace, and Cyberdesk, with a mobile viewport and grayscale
proof mode to check hierarchy without color. These are illustrative previews
of each app's common UI patterns, not embedded live app sessions. Keyboard-
focus styling is visible in the preview. Undo/redo (Ctrl+Z / Ctrl+Shift+Z) and
a saved/new/unsaved status keep draft edits reviewable. Its inspector separates
theme identity, semantic color editing, design tokens, and quality checks. The
palette editor supports all eleven color roles with color pickers and direct
hex entry, and its live quality panel reports dark and light contrast checks.
The style controls cover interface, display, and monospace fonts, density,
corner shape, and motion. A custom light palette can be edited separately.

**Edit** loads a theme for changes; saving a custom theme updates its stable
catalog ID even if its display name changes. Built-in themes remain immutable.
Use **Duplicate** to start a custom theme from any built-in or custom theme
without overwriting the source. **New theme** starts a draft from the active
palette. **Make this a custom theme** clones the live preview, including current
palette and design edits, into a new unsaved draft. Import loads a draft and
does not write it until Save; export downloads the current draft as JSON. The
theme library groups entries by family, and the family field can select an
existing pack or define a new one. Pack import and export and curated
collections are available in the library panel.

### Portable theme packs

A pack is a JSON document containing pack metadata and one to 256 complete
theme documents. Metadata includes a pack version, author, license, homepage,
minimum Cybercore compatibility (for example `0.8+`), and up to five validated
preview colors. Existing format-1 packs remain readable with safe metadata
defaults. Studio exports a selected family as
`<pack-id>.cyberpack.json`; **Install theme pack** validates the pack before
showing its theme count and ID conflicts. Choose a policy to reject all
conflicts (the default), skip existing IDs, or replace matching custom themes.
Built-in themes are protected even
under the replace policy. Reject policy checks every ID before writing, so a
conflicting pack cannot partially install.

The Studio pack API is `GET /api/packs` (family counts and metadata),
`GET /api/packs/export?family=<family>`, `POST /api/packs/validate`, and
`POST /api/packs`. Import requests contain `{ "pack": <pack-document>,
"policy": "reject|skip|replace_custom" }`; the policy is optional and
defaults to `reject`. The engine exposes `ThemePackDocument` and
`ThemeCatalog::export_family`, `pack_conflicts`, and `import_pack` for other
Cybercore consumers.

The curated library displays license, version, compatibility, and palette
swatches before installation. Imported packs receive the same metadata and
preview validation before conflict handling or writes.

## Current integration scope

Theme Studio's curated library (`GET /api/library`) lists the reviewed
embedded theme families. **Add collection to my library** copies a chosen
family into the custom catalog using `curated-<family>-<theme-id>` IDs and a
`curated-<family>` family name. Built-ins are unchanged, repeat installs skip
already-copied IDs, and the current active theme is not changed. The install
endpoint is `POST /api/library/:family/install`.

The crate owns the document format, validation, catalog, selection, contrast
report, and CSS generation. Cybercore Theme Studio provides the universal
visual creator. Cyberdeck Hub and Cyberdesk consume the shared catalog and
custom themes. DaemonHall and Dockspace use the same catalog and CSS adapter.
Web clients subscribe to a local server-sent event endpoint. Each adapter
checks the shared catalog revision and emits an event when a theme, selection,
appearance, addition, or removal changes; clients then refresh the catalog
immediately without restarting services. Cyberterm compares the same revision
while running and reapplies the selected palette only when the shared catalog
changes. `ThemeCatalog::revision` is opaque and intended for change detection,
not persistence or comparisons across processes.

The consumer compatibility workflow tests Hub, Cyberdesk, DaemonHall,
Dockspace, and Cyberterm against their locked released dependency every week
and on manual dispatch. Run it before publishing a Cybercore release and after
updating consumer dependency pins.

Consumers should pin a released `cybercore` version for reproducible builds.
During development, pinning a reviewed Git revision is supported; updating a
consumer's dependency revision does not require changing the theme document
format.
