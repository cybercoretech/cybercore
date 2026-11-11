/* <cybercore-brand name="logo.symbol" size="48" label="…"> inlines a themed
 * Cybercore brand SVG. Assets resolve relative to this file:
 *   brand/brand.json, brand/<asset.svg>
 * Fetched markup is sanitised and never executes script. */

const warned = new Set();
function warn(msg) {
  if (warned.has(msg)) return;
  warned.add(msg);
  console.warn(`cybercore-brand: ${msg}`);
}

export function resolveAsset(index, name) {
  const assets = index && Array.isArray(index.assets) ? index.assets : [];
  return assets.find(a => a && a.id === name) || null;
}

export function sanitizeSvg(markup) {
  return String(markup)
    .replace(/<script\b[\s\S]*?<\/script\s*>/gi, '')
    .replace(/<script\b[^>]*\/>/gi, '')
    .replace(/<foreignObject\b[\s\S]*?<\/foreignObject\s*>/gi, '')
    .replace(/<foreignObject\b[^>]*\/>/gi, '')
    .replace(/\son[a-z]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi, '')
    .replace(/\s(?:xlink:)?href\s*=\s*(?:"\s*javascript:[^"]*"|'\s*javascript:[^']*'|\s*javascript:[^\s>]*)/gi, '');
}

const BASE = new URL('brand/', import.meta.url);
let indexPromise = null;
function loadIndex() {
  indexPromise ||= fetch(new URL('brand.json', BASE)).then(r => {
    if (!r.ok) throw new Error(`brand.json: HTTP ${r.status}`);
    return r.json();
  });
  return indexPromise;
}

export function defineBrandElement(registry = globalThis.customElements) {
  if (!registry || registry.get('cybercore-brand') || typeof HTMLElement === 'undefined') return;
  class CybercoreBrand extends HTMLElement {
    static get observedAttributes() { return ['name', 'size', 'label']; }
    connectedCallback() { this.render(); }
    attributeChangedCallback() { if (this.isConnected) this.render(); }
    async render() {
      const token = (this._token = (this._token || 0) + 1);
      const name = this.getAttribute('name');
      this.replaceChildren();
      try {
        const asset = resolveAsset(await loadIndex(), name);
        if (!asset || !asset.svg) { warn(`unknown brand asset "${name}"`); return; }
        const res = await fetch(new URL(asset.svg, BASE));
        if (!res.ok) throw new Error(`${asset.svg}: HTTP ${res.status}`);
        const markup = sanitizeSvg(await res.text());
        if (token !== this._token) return;
        const tpl = document.createElement('template');
        tpl.innerHTML = markup;
        const svg = tpl.content.querySelector('svg');
        if (!svg) { warn(`no svg in "${name}"`); return; }
        const size = this.getAttribute('size');
        if (size) { svg.setAttribute('width', size); svg.setAttribute('height', size); }
        svg.setAttribute('role', 'img');
        svg.setAttribute('aria-label', this.getAttribute('label') || asset.title || name);
        this.replaceChildren(svg);
      } catch (e) {
        warn(`could not load "${name}" (${e.message})`);
      }
    }
  }
  registry.define('cybercore-brand', CybercoreBrand);
}

if (globalThis.customElements) defineBrandElement();
