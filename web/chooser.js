/* Cybercore universal theme chooser.
 *
 * One file, served by the org site, used by every Cybercore page:
 *
 *   <link rel="stylesheet" href="https://cybercoretech.github.io/kit/chooser.css">
 *   <script type="module" src="https://cybercoretech.github.io/kit/chooser.js"></script>
 *   <cybercore-theme-chooser shuffle></cybercore-theme-chooser>
 *
 * Data comes from the org site at runtime, so new themes show up everywhere
 * without touching any site:
 *   data/cybergrid.json                 families → theme ids
 *   data/themes/<family>/<id>.json      palettes
 *   data/mashups.json                   signature mashups (base surfaces × accent neons)
 *   data/kitty-themes.json              classic terminal palettes, used only as
 *                                       mashup ingredients (kitty:<name>)
 *
 * Attributes on <cybercore-theme-chooser>:
 *   shuffle            show the random-theme (dice) button
 *   hotkeys            T opens, R randomises, [ and ] step through themes
 *   default="<id>"     theme used when the visitor hasn't picked one
 *                      (falls back to window.SITE.defaultTheme)
 *
 * The first option is always "Site default": the page's own signature look.
 * That is window.SITE.defaultTheme (a shared theme or one of the page's own
 * SITE.mashups) or, when the page sets none, its stylesheet colours. Site
 * mashups are only used for this; the shared list stays 600 + 12 everywhere.
 * SITE.siteLabel names a stylesheet-only default (e.g. 'Tag Redact').
 *   align="left"       open the menu from the left edge (default: right)
 *   compact            below 560px wide, show only the swatch
 *   match="<selector>" copy typography (font, size, spacing, case, colour)
 *                      from one of the site's own elements, e.g. a nav link
 *   adopt              copy the look of the site's own theme button: leave the
 *                      old <button> inside the element and its computed font,
 *                      case, corners, border and fill become the --ccx-* tokens
 *
 * Painting: the chosen palette is written to CSS custom properties on <html>.
 * The default names are the Cybercore ones (--bg, --white, --panel, --line,
 * --muted, --acid, --pink, --purple, --cyan, --orange, --red). A page with its
 * own variable names maps them once:
 *   window.SITE = { themeVars: { bg: '--surface-0', white: ['--text', '--fg'], line: null } }
 * (null = leave that variable alone). Every change is also announced as a
 * `cc:theme` event on document with { id, palette }.
 */
import { paintBrandVars } from './brand-vars.js';

const KIT_DATA = new URL('../data/', import.meta.url).href;

const SLOTS = ['bg', 'white', 'panel', 'line', 'muted', 'acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red'];
const SURFACE = ['bg', 'white', 'panel', 'line', 'muted'];
const DEFAULT_VARS = {
  bg: '--bg', white: '--white', panel: '--panel', line: '--line', muted: '--muted',
  acid_green: '--acid', hot_pink: '--pink', purple: '--purple', cyan: '--cyan', orange: '--orange', red: '--red'
};
const FALLBACK = { bg: '09080f', white: 'f0f0f2', panel: '16151f', line: '32303e', muted: '807f8b', acid_green: '39ff33', hot_pink: 'ff147f', purple: '8a22e2', cyan: '00e5ff', orange: 'ff9000', red: 'f1142d' };
const STORE = 'cybercore-theme';
const STORE_PALETTE = 'cybercore-theme-palette';
const SITE_ID = 'site';
const LABELS = {
  mashup: 'Signature mashups', 'cybercore-tech': 'Cybercore Tech / master mix', default: 'Default / classics'
};
/* Identity colours for the original families; every other family gets a
   stable hue from its name, so new families need no code change. */
const FAMILY_COLORS = {
  mashup: ['#ff2bd6', '#00f0ff'], 'cybercore-tech': ['#39ff33', '#00e5ff'], cyberdyne: ['#ff1f3d', '#ff7a00'],
  cyberpunk: ['#ff0055', '#bf00ff'], default: ['#7dcfff', '#bb9af7'], dystopian: ['#d4e157', '#e65100'],
  neosynth: ['#00ff8c', '#ff2a9d'], synthwave: ['#4cc9f0', '#ff007f']
};
const HEX = /^[0-9a-f]{6}$/i;
const COLOR = /#[0-9a-f]{3,8}\b|rgba?\([^()]*\)/gi;

/* Literal colours -> {r,g,b,a}. Accepts #rgb[a], #rrggbb[aa], rgb()/rgba()
   with commas or spaces (what browsers serialise CSSOM values to). */
function parseColor(text) {
  const t = String(text).trim();
  let m = /^#([0-9a-f]{3,8})$/i.exec(t);
  if (m) {
    let h = m[1];
    if (h.length === 3 || h.length === 4) h = [...h].map(c => c + c).join('');
    if (h.length !== 6 && h.length !== 8) return null;
    const n = i => parseInt(h.slice(i, i + 2), 16);
    return { r: n(0), g: n(2), b: n(4), a: h.length === 8 ? n(6) / 255 : 1 };
  }
  m = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)(?:\s*[,/]\s*([\d.]+%?))?\s*\)$/i.exec(t);
  if (!m) return null;
  const a = m[4] == null ? 1 : m[4].endsWith('%') ? parseFloat(m[4]) / 100 : parseFloat(m[4]);
  return { r: +m[1], g: +m[2], b: +m[3], a };
}
const hexRgb = h => parseColor(`#${h}`);
const lum = c => (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b) / 255;

const titleCase = s => String(s).split(/[-_\s]+/).filter(Boolean).map(w => (/\d/.test(w) ? w.toUpperCase() : w[0].toUpperCase() + w.slice(1))).join(' ');
const hueOf = name => [...name].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7) % 360;
const familyColors = family => FAMILY_COLORS[family] || [`hsl(${hueOf(family)} 95% 62%)`, `hsl(${(hueOf(family) + 150) % 360} 95% 66%)`];
const reducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
const safeGet = key => { try { return localStorage.getItem(key); } catch { return null; } };
const safeSet = (key, value) => { try { value == null ? localStorage.removeItem(key) : localStorage.setItem(key, value); } catch { /* storage blocked */ } };

/* Palettes come in two shapes: the original 11 slots and the 20-role format
   (border/surface instead of line/panel). Normalise to the 11 slots. */
function normalise(raw) {
  const p = { ...raw };
  p.line ||= p.border;
  p.panel ||= p.surface;
  p.white ||= p.text_primary;
  for (const k of SLOTS) if (!HEX.test(p[k] || '')) p[k] = FALLBACK[k];
  return p;
}
const blend = (base, accent) => { const out = { ...accent }; SURFACE.forEach(k => { out[k] = base[k]; }); return out; };

/* ---------------------------------------------------------------- store */
function createStore() {
  const site = window.SITE || {};
  const dataRoot = String(site.dataRoot || KIT_DATA).replace(/\/?$/, '/');
  const vars = { ...DEFAULT_VARS, ...(site.themeVars || {}) };
  const listeners = new Set();
  const cache = new Map();
  let families = {};
  let mashups = [];
  let siteBlends = [];
  let kitty = {};
  const isKitty = id => typeof id === 'string' && id.startsWith('kitty:') && Boolean(kitty[id.slice(6)]);
  const ingredient = id => familyOf.has(id) || isKitty(id);
  let familyOf = new Map();
  let ids = [];
  let current = null;
  let currentPalette = null;
  let painted = [];
  /* Most sites hard-code colours (panel fills, borders, glows) next to their
     CSS variables, so variables alone leave much of the page unthemed. On a
     theme pick every literal colour in the page's own stylesheets and inline
     styles is re-expressed against the nearest site variable (its original
     value read before any theme paints) and moved onto the theme's colour
     for that slot, keeping its offset and alpha. Edits happen in place, so
     the cascade is untouched, and "Site default" restores every original.
     Opt out with window.SITE.recolor = false. */
  let anchors = null;
  let edits = [];
  const paintedNames = new Set(Object.values(vars).flat().filter(Boolean));
  function readAnchors() {
    const cs = getComputedStyle(document.documentElement);
    const out = [];
    for (const [slot, names] of Object.entries(vars)) {
      for (const name of [].concat(names || [])) {
        const c = parseColor(cs.getPropertyValue(name));
        if (c && c.a === 1) { out.push({ slot, c }); break; } // translucent tints are not a slot's identity
      }
    }
    return out;
  }
  function restoreLiterals() {
    for (let i = edits.length - 1; i >= 0; i--) { const e = edits[i]; e.style.setProperty(e.prop, e.value, e.priority); }
    edits = [];
  }
  function recolorLiterals(p) {
    if (!anchors.length || site.recolor === false) return;
    const map = (text, prop) => {
      const c = parseColor(text);
      if (!c) return text;
      let best = null;
      let dist = Infinity;
      for (const a of anchors) {
        const d = Math.hypot(c.r - a.c.r, c.g - a.c.g, c.b - a.c.b);
        if (d < dist) { dist = d; best = a; }
      }
      if (!best || dist > 96 || !p[best.slot]) return text;
      if (prop.includes('shadow') && lum(c) < 0.06) return text; // plain dark drop shadows stay
      const to = hexRgb(p[best.slot]);
      const surface = best.slot === 'bg' || best.slot === 'panel' || best.slot === 'line';
      const flip = surface && (lum(best.c) < 0.5) !== (lum(to) < 0.5) ? -1 : 1; // surface tints follow a light theme the other way
      const ch = k => Math.max(0, Math.min(255, Math.round(to[k] + flip * (c[k] - best.c[k]))));
      const alpha = Math.round(c.a * 1000) / 1000;
      return alpha < 1 ? `rgba(${ch('r')}, ${ch('g')}, ${ch('b')}, ${alpha})` : `rgb(${ch('r')}, ${ch('g')}, ${ch('b')})`;
    };
    const fix = style => {
      const props = Array.from(style);
      for (const prop of props) {
        if (paintedNames.has(prop) || prop.startsWith('--ccx')) continue;
        const value = style.getPropertyValue(prop);
        if (!value || !COLOR.test(value)) { COLOR.lastIndex = 0; continue; }
        COLOR.lastIndex = 0;
        const next = value.replace(COLOR, m => map(m, prop));
        if (next === value) continue;
        const priority = style.getPropertyPriority(prop);
        edits.push({ style, prop, value, priority });
        style.setProperty(prop, next, priority);
      }
    };
    const walk = rules => { for (const r of rules) { if (r.style) fix(r.style); if (r.cssRules) walk(r.cssRules); } };
    for (const sheet of document.styleSheets) {
      if (/\/kit\/chooser\.css(\?|$)/.test(sheet.href || '')) continue;
      let rules;
      try { rules = sheet.cssRules; } catch { continue; } // cross-origin sheet
      if (rules) walk(rules);
    }
    document.querySelectorAll('body [style]').forEach(el => { if (!el.closest('cybercore-theme-chooser')) fix(el.style); });
  }

  const mashOf = id => mashups.find(m => m.id === id) || siteBlends.find(m => m.id === id);
  const parseMix = id => {
    if (!id || !id.startsWith('mix:')) return null;
    const [, base, accent] = id.split(':');
    return ingredient(base) && ingredient(accent) ? { id, base, accent, label: `${titleCase(base)} × ${titleCase(accent)}` } : null;
  };
  const blendOf = id => mashOf(id) || parseMix(id);
  const known = id => id === SITE_ID || familyOf.has(id) || Boolean(blendOf(id));
  const siteTheme = () => (site.defaultTheme && site.defaultTheme !== SITE_ID && (familyOf.has(site.defaultTheme) || blendOf(site.defaultTheme)) ? site.defaultTheme : null);
  const labelOf = id => {
    if (id !== SITE_ID) return blendOf(id)?.label || titleCase(id);
    const own = siteTheme();
    const name = own ? titleCase(blendOf(own)?.label || own) : site.siteLabel;
    return name ? `Site default · ${name}` : 'Site default';
  };
  /* Button text: just the look's name, without the "Site default" prefix. */
  const shortLabel = id => {
    if (id !== SITE_ID) return labelOf(id);
    const own = siteTheme();
    return own ? titleCase(blendOf(own)?.label || own) : (site.siteLabel || 'Site default');
  };

  const ready = (async () => {
    const optional = (file, empty) => fetch(`${dataRoot}${file}`).then(r => (r.ok ? r.json() : empty)).catch(() => empty);
    const [grid, mash, kit] = await Promise.all([
      fetch(`${dataRoot}cybergrid.json`).then(r => { if (!r.ok) throw new Error(`cybergrid.json: HTTP ${r.status}`); return r.json(); }),
      optional('mashups.json', { mashups: [] }),
      optional('kitty-themes.json', { themes: {} })
    ]);
    kitty = kit.themes || {};
    families = grid.families || {};
    familyOf = new Map(Object.entries(families).flatMap(([f, names]) => names.map(n => [n, f])));
    const valid = m => m && m.id && ingredient(m.base) && ingredient(m.accent);
    mashups = (mash.mashups || []).filter(valid);
    siteBlends = (Array.isArray(site.mashups) ? site.mashups : []).filter(m => valid(m) && !mashups.some(x => x.id === m.id));
    ids = mashups.map(m => m.id).concat([...familyOf.keys()]);
  })();

  async function palette(id) {
    if (cache.has(id)) return cache.get(id);
    if (isKitty(id)) { const k = Promise.resolve(normalise(kitty[id.slice(6)])); cache.set(id, k); return k; }
    const family = familyOf.get(id);
    if (!family) throw new Error(`unknown theme ${id}`);
    const pending = fetch(`${dataRoot}themes/${family}/${id}.json`)
      .then(r => { if (!r.ok) throw new Error(`${id}: HTTP ${r.status}`); return r.json(); })
      .then(normalise);
    cache.set(id, pending);
    pending.catch(() => cache.delete(id));
    return pending;
  }
  async function resolve(id) {
    if (id === SITE_ID) { const own = siteTheme(); return own ? resolve(own) : null; }
    const b = blendOf(id);
    if (!b) return palette(id);
    const [base, accent] = await Promise.all([palette(b.base), palette(b.accent)]);
    return blend(base, accent);
  }

  function writeVars(p, recolor = true) {
    const root = document.documentElement;
    restoreLiterals();
    painted.forEach(v => root.style.removeProperty(v));
    painted = [];
    anchors ||= readAnchors();
    if (!p) return;
    if (recolor) recolorLiterals(p);
    for (const [slot, names] of Object.entries(vars)) {
      if (!names || !p[slot]) continue;
      for (const name of [].concat(names)) { root.style.setProperty(name, `#${p[slot]}`); painted.push(name); }
    }
    paintBrandVars(root.style, p, recolor, painted);
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.content = `#${p.bg}`;
  }
  function paint(id, p) {
    const go = () => {
      writeVars(p, id !== SITE_ID && id !== siteTheme()); // the site's own look stays exactly as designed
      document.documentElement.dataset.theme = id;
    };
    if (current !== null && document.startViewTransition && !reducedMotion()) document.startViewTransition(go);
    else go();
    current = id; currentPalette = p;
    document.dispatchEvent(new CustomEvent('cc:theme', { detail: { id, palette: p, label: labelOf(id) } }));
    listeners.forEach(fn => fn(id, p));
  }

  async function apply(id, { persist = true } = {}) {
    let p;
    try { p = await resolve(id); } catch (e) { console.warn('Cybercore theme:', e); return false; }
    paint(id, p);
    if (persist) {
      safeSet(STORE, id === SITE_ID ? null : id);
      safeSet(STORE_PALETTE, p && id !== SITE_ID ? JSON.stringify({ id, palette: p }) : null);
      const url = new URL(location.href);
      if (id === SITE_ID) url.searchParams.delete('theme'); else url.searchParams.set('theme', id);
      history.replaceState(history.state, '', url);
    }
    return true;
  }

  /* Paint the last palette straight from storage so the page doesn't flash
     its default colours while the registry loads. */
  function paintCached(startId) {
    try {
      const cached = JSON.parse(safeGet(STORE_PALETTE) || 'null');
      if (cached && cached.id === startId && cached.palette) writeVars(normalise(cached.palette), startId !== site.defaultTheme);
    } catch { /* ignore a corrupt cache */ }
  }

  return {
    ready, apply, resolve, palette, known, labelOf, shortLabel, mashOf, blendOf, paintCached,
    get families() { return families; },
    get mashups() { return mashups; },
    get ids() { return ids; },
    get total() { return ids.length; },
    get current() { return current; },
    get currentPalette() { return currentPalette; },
    familyOf: id => (mashOf(id) ? 'mashup' : familyOf.get(id)),
    cached: id => cache.get(id),
    onChange(fn) { listeners.add(fn); return () => listeners.delete(fn); },
    started: false
  };
}

const store = window.CybercoreThemes ||= createStore();

/* Pick the starting theme once per page, however many choosers it has. */
async function start(defaultId) {
  if (store.started) return store.ready;
  store.started = true;
  const requested = new URLSearchParams(location.search).get('theme');
  const saved = safeGet(STORE);
  const fallback = defaultId || SITE_ID;
  if (!requested) store.paintCached(saved || fallback);
  try { await store.ready; } catch (e) {
    console.warn('Cybercore theme registry offline:', e);
    document.querySelectorAll('cybercore-theme-chooser').forEach(el => el.offline());
    return;
  }
  const first = [requested, saved, fallback].find(id => id && store.known(id)) || SITE_ID;
  if (!(await store.apply(first, { persist: Boolean(requested) }))) await store.apply(SITE_ID, { persist: false });
  document.querySelectorAll('[data-cc-theme-count]').forEach(el => { el.textContent = String(store.total); });
  document.dispatchEvent(new CustomEvent('cc:themes-ready', { detail: { total: store.total, mashups: store.mashups.length, families: Object.keys(store.families).length } }));
}

/* -------------------------------------------------------------- element */
/* Read a site's own button styling into --ccx-* tokens (see `adopt`). */
function adoptStyle(from, to) {
  const cs = getComputedStyle(from);
  const set = (k, v) => { if (v && v !== 'none' && v !== 'normal' && v !== 'auto' && v !== 'rgba(0, 0, 0, 0)') to.style.setProperty(k, v); };
  set('--ccx-font', cs.fontFamily);
  if (parseFloat(cs.fontSize) >= 12) set('--ccx-size', cs.fontSize);
  set('--ccx-tracking', cs.letterSpacing);
  if (cs.textTransform === 'lowercase' || cs.textTransform === 'capitalize' || cs.textTransform === 'uppercase') set('--ccx-case', cs.textTransform);
  set('--ccx-weight', cs.fontWeight);
  set('--ccx-clip', cs.clipPath);
  const bordered = cs.borderTopStyle !== 'none' && parseFloat(cs.borderTopWidth) > 0;
  to.style.setProperty('--ccx-button-border', bordered ? cs.borderTopColor : 'transparent');
  to.style.setProperty('--ccx-button-bg', cs.backgroundColor);
  to.style.setProperty('--ccx-button-fg', cs.color);
  if (parseFloat(cs.height) >= 24) set('--ccx-height', cs.height);
  to.style.setProperty('--ccx-button-width', 'auto');
}

/* Typography only, from a reference element on the page (see `match`). */
function adoptType(from, to) {
  const cs = getComputedStyle(from);
  if (cs.fontFamily) to.style.setProperty('--ccx-font', cs.fontFamily);
  if (parseFloat(cs.fontSize) >= 12) to.style.setProperty('--ccx-size', cs.fontSize);
  if (cs.letterSpacing && cs.letterSpacing !== 'normal') to.style.setProperty('--ccx-tracking', cs.letterSpacing);
  to.style.setProperty('--ccx-case', cs.textTransform === 'none' ? 'none' : cs.textTransform);
  if (cs.fontWeight) to.style.setProperty('--ccx-weight', cs.fontWeight);
  if (cs.color) to.style.setProperty('--ccx-button-fg', cs.color);
}

const DICE = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3.5" y="3.5" width="17" height="17" rx="3"/><circle cx="8.5" cy="8.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="15.5" cy="8.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.1" fill="currentColor" stroke="none"/><circle cx="8.5" cy="15.5" r="1.1" fill="currentColor" stroke="none"/><circle cx="15.5" cy="15.5" r="1.1" fill="currentColor" stroke="none"/></svg>';
let seq = 0;

class CybercoreThemeChooser extends HTMLElement {
  connectedCallback() {
    if (this.built) return;
    this.built = true;
    // mdBook and similar pages shrink the root font (62.5% = 10px); size the
    // chooser from 16px there so it matches every other site (see chooser.css).
    const html = document.documentElement;
    if (parseFloat(getComputedStyle(html).fontSize) < 14 && !html.style.getPropertyValue('--ccx-rem')) html.style.setProperty('--ccx-rem', '16px');
    const uid = `ccx${++seq}`;
    const legacy = this.hasAttribute('adopt') && this.querySelector('button');
    if (legacy) adoptStyle(legacy, this);
    const ref = this.getAttribute('match') && document.querySelector(this.getAttribute('match'));
    if (ref) adoptType(ref, this);
    this.classList.add('ccx');
    // data-lenis-prevent: pages with Lenis smooth scroll (e.g. fdroid) would
    // otherwise take the wheel and scroll the page instead of the list.
    this.innerHTML = `
      <div class="ccx-picker">
        <button class="ccx-button" type="button" aria-haspopup="listbox" aria-expanded="false" aria-controls="${uid}-menu" title="Theme">
          <i class="ccx-swatch" aria-hidden="true"></i><span class="ccx-label">Loading themes…</span><span class="ccx-chev" aria-hidden="true">⌄</span>
        </button>
        <div class="ccx-menu" id="${uid}-menu" data-lenis-prevent hidden>
          <div class="ccx-search"><input type="search" placeholder="Filter themes…" aria-label="Filter themes" autocomplete="off" spellcheck="false"></div>
          <div class="ccx-list" id="${uid}-list" role="listbox" aria-label="Theme"></div>
          <p class="ccx-empty" hidden>No themes match.</p>
        </div>
      </div>
      ${this.hasAttribute('shuffle') ? `<button class="ccx-shuffle" type="button" aria-label="Random theme" title="Random theme">${DICE}</button>` : ''}`;
    this.button = this.querySelector('.ccx-button');
    this.menu = this.querySelector('.ccx-menu');
    this.list = this.querySelector('.ccx-list');
    this.filter = this.querySelector('.ccx-search input');
    this.empty = this.querySelector('.ccx-empty');
    this.labelEl = this.querySelector('.ccx-label');
    this.uid = uid;
    if (this.getAttribute('align') === 'left') this.menu.classList.add('ccx-left');
    // Open menus live in the browser's top layer so no page section, logo or
    // stacking context on any site can cover them.
    this.layered = typeof this.menu.showPopover === 'function';
    if (this.layered) { this.menu.setAttribute('popover', 'manual'); this.menu.classList.add('ccx-layer'); }
    this.wire();
    this.unsubscribe = store.onChange((id, p) => this.sync(id, p));
    store.ready.then(() => { this.build(); if (store.current) this.sync(store.current, store.currentPalette); }).catch(() => {});
    start(this.getAttribute('default') || null);
    requestAnimationFrame(() => this.fit());
    document.fonts?.ready.then(() => this.fit());
  }
  /* If the full button would run past the edge of the screen (a header that
     doesn't wrap, a long theme name), show just the swatch, like `compact`.
     Measured with the full label in the same frame, so nothing flickers. */
  fit() {
    if (!this.button) return;
    this.classList.remove('ccx-fit');
    const vw = document.documentElement.clientWidth;
    const host = this.parentElement?.getBoundingClientRect();
    if (host && (host.left >= vw || host.right <= 0)) return; // inside an off-canvas drawer
    const r = this.button.getBoundingClientRect();
    if (r.width && (r.right > vw - 2 || r.left < 2)) this.classList.add('ccx-fit');
  }
  disconnectedCallback() { this.unsubscribe?.(); this.observer?.disconnect(); document.removeEventListener('pointerdown', this.outside); document.removeEventListener('keydown', this.hotkey); }

  offline() { this.labelEl.textContent = 'Themes offline'; this.button.disabled = true; }

  build() {
    const groups = [];
    groups.push([SITE_ID, [SITE_ID]], ['mashup', store.mashups.map(m => m.id)], ...Object.entries(store.families));
    const frag = document.createDocumentFragment();
    for (const [family, names] of groups) {
      if (!names.length) continue;
      const [fa, fb] = family === SITE_ID ? ['var(--ccx-fg)', 'var(--ccx-fg)'] : familyColors(family);
      const gid = `${this.uid}-g-${family}`;
      if (family !== SITE_ID) {
        const h = document.createElement('div');
        h.className = 'ccx-group'; h.id = gid; h.setAttribute('aria-hidden', 'true');
        h.style.cssText = `--fa:${fa};--fb:${fb}`;
        h.innerHTML = `<span></span><em>${names.length}</em>`;
        h.firstChild.textContent = LABELS[family] || titleCase(family);
        frag.append(h);
      }
      names.forEach((id, i) => {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'ccx-option'; b.tabIndex = -1;
        b.setAttribute('role', 'option'); b.setAttribute('aria-selected', 'false');
        if (family !== SITE_ID) b.setAttribute('aria-describedby', gid);
        b.dataset.value = id; b.dataset.family = family;
        b.dataset.search = `${id} ${family} ${store.labelOf(id)}`.toLowerCase().replace(/[\s_/]+/g, '-');
        b.style.cssText = `--fa:${i % 2 ? fb : fa}`;
        b.innerHTML = '<span class="ccx-name"></span><span class="ccx-dots" aria-hidden="true"></span>';
        b.firstChild.textContent = store.labelOf(id);
        frag.append(b);
      });
    }
    this.list.replaceChildren(frag);
    this.filter.placeholder = `Filter ${store.total} themes…`;
    this.button.title = `Theme (${store.total})`;
    // Swatch dots load as options scroll into view, never all 612 up front.
    this.observer = new IntersectionObserver(entries => entries.forEach(e => {
      if (!e.isIntersecting) return;
      this.observer.unobserve(e.target);
      this.dots(e.target);
    }), { root: this.list, rootMargin: '120px 0px' });
    this.list.querySelectorAll('.ccx-option').forEach(o => this.observer.observe(o));
  }
  async dots(option) {
    try {
      const p = await store.resolve(option.dataset.value);
      if (!p) return;
      option.querySelector('.ccx-dots').innerHTML = ['acid_green', 'hot_pink', 'cyan'].map(k => `<i style="background:#${p[k]}"></i>`).join('');
    } catch { /* leave the dots empty */ }
  }

  sync(id, p) {
    this.labelEl.textContent = store.shortLabel(id);
    this.button.setAttribute('aria-label', `Theme: ${store.labelOf(id)}`);
    this.fit();
    const swatch = p || null;
    this.style.setProperty('--ccx-s1', swatch ? `#${swatch.acid_green}` : 'var(--ccx-accent)');
    this.style.setProperty('--ccx-s2', swatch ? `#${swatch.hot_pink}` : 'var(--ccx-focus)');
    this.style.setProperty('--ccx-s3', swatch ? `#${swatch.cyan}` : 'var(--ccx-fg)');
    this.style.setProperty('--ccx-s4', swatch ? `#${swatch.purple}` : 'var(--ccx-muted)');
    // Roving tabindex: the listbox is one Tab stop, on the selected option.
    const opts = this.list.querySelectorAll('.ccx-option');
    let any = false;
    opts.forEach(o => { const on = o.dataset.value === id; o.setAttribute('aria-selected', String(on)); o.tabIndex = on ? 0 : -1; any ||= on; });
    if (!any && opts[0]) opts[0].tabIndex = 0;
  }

  isOpen() { return !this.menu.hidden; }
  open() {
    this.menu.hidden = false;
    if (this.layered) { try { this.menu.showPopover(); } catch { /* already open */ } }
    this.button.setAttribute('aria-expanded', 'true');
    this.place();
    const sel = this.list.querySelector('[aria-selected="true"]');
    if (sel) this.list.scrollTop = sel.offsetTop - this.list.clientHeight / 2 + sel.offsetHeight / 2;
    if (matchMedia('(pointer: fine)').matches) this.filter.focus({ preventScroll: true });
    else (sel || this.list.querySelector('.ccx-option'))?.focus({ preventScroll: true });
  }
  close(refocus = false) {
    if (!this.isOpen()) return;
    if (this.layered) { try { this.menu.hidePopover(); } catch { /* already closed */ } }
    this.menu.hidden = true;
    this.button.setAttribute('aria-expanded', 'false');
    this.filter.value = ''; this.runFilter('');
    if (refocus) this.button.focus({ preventScroll: true });
  }
  /* On narrow screens the menu becomes a fixed sheet under the button so it
     can never run off the edge of the page. */
  place() {
    if (this.layered) return this.placeLayer();
    const narrow = innerWidth < 560;
    this.menu.classList.toggle('ccx-sheet', narrow);
    const m = this.menu.style;
    m.top = m.left = m.width = m.bottom = '';
    this.list.style.maxHeight = '';
    this.menu.classList.remove('ccx-up');
    if (!narrow) {
      // Fit the list in the viewport; open upward when there's more room above.
      const b = this.button.getBoundingClientRect();
      const chrome = 80; // search box + padding
      const below = innerHeight - b.bottom - 16 - chrome;
      const above = b.top - 16 - chrome;
      const want = Math.min(416, innerHeight * 0.6);
      if (below < Math.min(want, 240) && above > below) {
        this.menu.classList.add('ccx-up');
        this.list.style.maxHeight = `${Math.max(140, Math.min(want, above))}px`;
      } else {
        this.list.style.maxHeight = `${Math.max(140, Math.min(want, below))}px`;
      }
      return;
    }
    const gutter = 16;
    const top = Math.max(8, Math.min(this.button.getBoundingClientRect().bottom + 8, innerHeight * 0.4));
    const width = document.documentElement.clientWidth - gutter * 2;
    m.top = `${top}px`; m.left = `${gutter}px`; m.width = `${width}px`;
    // A transformed or filtered ancestor turns position:fixed into
    // position:absolute-within-it; measure and cancel that offset.
    m.animation = 'none';
    const r = this.menu.getBoundingClientRect();
    m.animation = '';
    m.left = `${gutter * 2 - r.left}px`;
    m.top = `${top * 2 - r.top}px`;
    this.list.style.maxHeight = `${Math.max(160, innerHeight - top - 96)}px`;
  }
  /* Top-layer placement: fixed to the viewport, anchored to the button.
     Desktop: under it (or above when there's more room), right-aligned, or
     left-aligned with align="left". Narrow screens: a full-width sheet. */
  placeLayer() {
    const m = this.menu.style;
    const b = this.button.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    const gutter = innerWidth < 560 ? 16 : 8;
    this.menu.classList.remove('ccx-up');
    this.menu.classList.toggle('ccx-sheet', innerWidth < 560);
    m.top = m.bottom = m.left = m.width = '';
    if (innerWidth < 560) {
      const top = Math.max(8, Math.min(b.bottom + 8, innerHeight * 0.4));
      m.top = `${top}px`; m.left = `${gutter}px`; m.width = `${vw - gutter * 2}px`;
      this.list.style.maxHeight = `${Math.max(160, innerHeight - top - 96)}px`;
      return;
    }
    const chrome = 80; // search box + padding
    const below = innerHeight - b.bottom - 16 - chrome;
    const above = b.top - 16 - chrome;
    const want = Math.min(416, innerHeight * 0.6);
    const up = below < Math.min(want, 240) && above > below;
    this.list.style.maxHeight = `${Math.max(140, Math.min(want, up ? above : below))}px`;
    const w = this.menu.offsetWidth;
    const left = this.menu.classList.contains('ccx-left') ? b.left : b.right - w;
    m.left = `${Math.round(Math.max(gutter, Math.min(left, vw - w - gutter)))}px`;
    if (up) m.bottom = `${Math.round(innerHeight - b.top + 8)}px`; else m.top = `${Math.round(b.bottom + 8)}px`;
  }
  runFilter(raw) {
    const q = raw.trim().toLowerCase().replace(/[\s_/]+/g, '-');
    let head = null, any = false, total = 0;
    for (const el of this.list.children) {
      if (el.classList.contains('ccx-group')) { if (head) head.hidden = !any; head = el; any = false; continue; }
      const hit = !q || el.dataset.search.includes(q);
      el.hidden = !hit; any ||= hit; total += hit ? 1 : 0;
    }
    if (head) head.hidden = !any;
    this.empty.hidden = total > 0;
  }
  async pick(id) {
    this.close(true);
    await store.apply(id);
  }
  random() {
    const pool = store.ids.filter(id => id !== store.current);
    if (pool.length) store.apply(pool[Math.floor(Math.random() * pool.length)]);
  }
  step(dir) {
    const ids = store.ids;
    if (!ids.length) return;
    const i = ids.indexOf(store.current);
    store.apply(ids[(i + dir + ids.length) % ids.length]);
  }

  wire() {
    this.button.addEventListener('click', () => (this.isOpen() ? this.close() : this.open()));
    this.querySelector('.ccx-shuffle')?.addEventListener('click', () => this.random());
    this.list.addEventListener('click', e => { const o = e.target.closest('.ccx-option'); if (o) this.pick(o.dataset.value); });
    this.filter.addEventListener('input', () => this.runFilter(this.filter.value));
    this.outside = e => { if (this.isOpen() && !this.querySelector('.ccx-picker').contains(e.target)) this.close(); };
    document.addEventListener('pointerdown', this.outside);
    addEventListener('resize', () => { this.fit(); if (this.isOpen()) this.place(); }, { passive: true });
    addEventListener('scroll', e => { if (this.isOpen() && this.layered && !this.menu.contains(e.target)) this.place(); }, { passive: true, capture: true });
    this.menu.addEventListener('keydown', e => {
      const opts = [...this.list.querySelectorAll('.ccx-option:not([hidden])')];
      const i = opts.indexOf(document.activeElement);
      if (e.key === 'Escape') { e.preventDefault(); this.close(true); }
      else if (e.key === 'Tab') this.close();
      else if (e.key === 'ArrowDown') { e.preventDefault(); (opts[i + 1] || opts[0])?.focus(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); (i <= 0 ? this.filter : opts[i - 1])?.focus(); }
      else if (e.key === 'Home' && i >= 0) { e.preventDefault(); opts[0]?.focus(); }
      else if (e.key === 'End' && i >= 0) { e.preventDefault(); opts.at(-1)?.focus(); }
      else if (e.key === 'Enter' && document.activeElement === this.filter && opts[0]) { e.preventDefault(); this.pick(opts[0].dataset.value); }
    });
    this.hotkey = e => {
      if (!this.hasAttribute('hotkeys') || e.metaKey || e.ctrlKey || e.altKey) return;
      if (e.target.closest?.('input, textarea, select, [contenteditable]')) return;
      if (e.key === 't' || e.key === 'T') { e.preventDefault(); this.isOpen() ? this.close(true) : this.open(); }
      else if (e.key === 'r' || e.key === 'R') this.random();
      else if (e.key === ']') this.step(1);
      else if (e.key === '[') this.step(-1);
    };
    document.addEventListener('keydown', this.hotkey);
  }
}

if (!customElements.get('cybercore-theme-chooser')) customElements.define('cybercore-theme-chooser', CybercoreThemeChooser);

export default store;
