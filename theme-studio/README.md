# Cybercore Theme Studio

Cybercore Theme Studio is the standalone visual creator for CYBERGRID themes.
The built-in registry contains 242 themes across the curated families, and
each built-in theme exposes governed CyberDark wallpaper options when the
Control Plane metadata is available.
It is a small local Rust web app that uses the `cybercore::theme::ThemeCatalog`
directly, so themes it saves are immediately available to Cyberdeck Hub,
Cyberdesk, and other apps using the same Cybercore user configuration.

## Run locally

From the Cybercore repository root:

```bash
cargo run -p cybercore-theme-studio
```

Then open <http://127.0.0.1:8761/>. The server only binds to loopback and
rejects unexpected `Host` headers. For a friendly local hostname, a name under
`.localhost` (for example, `http://cybercore-tech.localhost:8761/`) is accepted
and resolves to loopback without a hosts-file change. Set
`CYBERCORE_THEME_STUDIO_BIND` to use a different loopback address or port.
Cyberdeck Hub has a **Theme Studio** link to the default URL.

The studio provides a searchable theme library, app-reference previews with a
mobile viewport, and an inspector organized around theme identity, semantic
colors, design tokens, and quality checks. App-reference previews cover
Cyberdeck Hub, Dockspace, and Cyberdesk, with grayscale proof and keyboard
focus visibility. These are illustrative app-pattern previews rather than
embedded live app sessions. Undo/redo (Ctrl+Z / Ctrl+Shift+Z) and a
saved/new/unsaved indicator protect the draft workflow. It supports built-in
and custom themes, dark and optional light palettes, typography, density,
corners, and motion tokens. Theme edits update the preview as you work, with
contrast feedback for both appearances. Import and export versioned theme
documents or share complete theme packs.

The **Curated pack library** offers the checked-in Cybercore theme families
as collections. Each preview shows pack version, author, license, engine
compatibility, and palette swatches before installation. **Add collection to my library** copies the selected family
under `curated-<family>` with namespaced IDs; it leaves embedded themes and
the active selection alone. Repeating the action safely skips themes already
copied. The library endpoints are `GET /api/library` and
`POST /api/library/:family/install`.

Themes and active selection use the same storage contract documented in the
[Cybercore theme engine guide](../docs/theme-engine.md):
`$XDG_CONFIG_HOME/cybercore` or `~/.config/cybercore`, with
`CYBERCORE_CONFIG_DIR` available for an explicit config root.

## Dashboard telemetry connectors

The dashboard keeps local Cybercore telemetry available without credentials.
Optional provider requests run server-side, are cached for five minutes, and
return `NOT CONFIGURED` or `ERROR` instead of fake zeroes. Configure only the
providers you want before starting Theme Studio:

```bash
export GITHUB_TOKEN=...
export GITHUB_REPOSITORY=owner/repository

export CLOUDFLARE_API_TOKEN=...
export CLOUDFLARE_ZONE_ID=...

export GOOGLE_PAGESPEED_URL=https://example.com
export GOOGLE_PAGESPEED_API_KEY=... # optional
```

GitHub reports repository basics, Cloudflare verifies the token and zone
access, and PageSpeed reports Lighthouse scores and core audit values. The
Google Analytics and Search Console variables are reserved for their
OAuth-backed adapters; they remain explicitly unavailable until those
property credentials are configured. Never put any of these values in the
browser or commit them to the repository.
