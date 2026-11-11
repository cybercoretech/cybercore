/* <cybercore-brand name="logo.symbol" size="48" label="…"> inlines a themed
 * Cybercore brand SVG. Assets resolve relative to this file:
 *   brand/brand.json, brand/<asset.svg>
 * Fetched markup is parsed as XML and cleaned node by node; it never executes script. */

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

// Decision predicates. The parser hands these already-decoded values, so
// character references and tab/newline splitting are handled by the parser.
// Decisions use local names and namespaces, never the prefix a file chose.
const SVG_NS = 'http://www.w3.org/2000/svg';

export function isUnsafeUrl(value) {
  // eslint-disable-next-line no-control-regex
  const v = String(value ?? '').replace(/[\u0000- ]/g, '');
  // Embedded raster images carry no script; every other data: URL stays blocked.
  if (/^data:image\/(?:png|jpeg|gif|webp)[;,]/i.test(v)) return false;
  return /^(?:javascript|vbscript|data):/i.test(v);
}

export function isUnsafeAttribute(localName, value) {
  const n = String(localName).toLowerCase();
  if (n.startsWith('on')) return true;
  return (n === 'href' || n === 'src' || n === 'srcdoc' || n === 'data') && isUnsafeUrl(value);
}

export function isUnsafeElement(namespaceURI, localName, attributeNameValue) {
  if (namespaceURI !== SVG_NS) return true;
  const n = String(localName).toLowerCase();
  if (n === 'script' || n === 'foreignobject') return true;
  if (n === 'set' || n === 'animate' || n === 'animatetransform' || n === 'animatemotion') {
    const a = String(attributeNameValue ?? '').split(':').pop().trim().toLowerCase();
    return a === 'href';
  }
  return false;
}

export function isSvgRoot(el) {
  return !!el && el.namespaceURI === SVG_NS && el.localName === 'svg';
}

// Removes unsafe elements and attributes from a parsed tree, in place.
export function cleanSvgTree(el) {
  for (const attr of Array.from(el.attributes)) {
    if (isUnsafeAttribute(attr.localName, attr.value)) el.removeAttribute(attr.name);
  }
  for (const child of Array.from(el.children)) {
    if (isUnsafeElement(child.namespaceURI, child.localName, child.getAttribute('attributeName'))) child.remove();
    else cleanSvgTree(child);
  }
}

// Parses fetched markup as XML and returns a cleaned <svg> element, or null.
function parseSvg(markup) {
  const doc = new DOMParser().parseFromString(markup, 'image/svg+xml');
  const root = doc.documentElement;
  if (!root || doc.getElementsByTagName('parsererror').length || !isSvgRoot(root)) return null;
  cleanSvgTree(root);
  return root;
}

const BASE = new URL('brand/', import.meta.url);
let indexPromise = null;
export function loadIndex() {
  indexPromise ||= fetch(new URL('brand.json', BASE)).then(r => {
    if (!r.ok) throw new Error(`brand.json: HTTP ${r.status}`);
    return r.json();
  });
  // A failed load is forgotten so the next element retries.
  indexPromise.catch(() => { indexPromise = null; });
  return indexPromise;
}

// One fetch per SVG URL, shared by every element and re-render; failures are forgotten.
const svgTexts = new Map();
export function loadSvgText(rel) {
  const url = new URL(rel, BASE).href;
  let p = svgTexts.get(url);
  if (!p) {
    p = fetch(url).then(r => {
      if (!r.ok) throw new Error(`${rel}: HTTP ${r.status}`);
      return r.text();
    });
    p.catch(() => { if (svgTexts.get(url) === p) svgTexts.delete(url); });
    svgTexts.set(url, p);
  }
  return p;
}

// Sizes and labels the root <svg>. The file's own aria-labelledby/describedby
// would override aria-label, so they are removed first. `size` sets the height
// only; the width follows the viewBox.
export function finishSvg(svg, { label, size } = {}) {
  svg.removeAttribute('aria-labelledby');
  svg.removeAttribute('aria-describedby');
  if (size) { svg.removeAttribute('width'); svg.setAttribute('height', size); }
  svg.setAttribute('role', 'img');
  svg.setAttribute('aria-label', label);
  return svg;
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
        if (!asset) { warn(`unknown brand asset "${name}"`); return; }
        if (!asset.svg) { warn(`brand asset ${name} has no SVG`); return; }
        const parsed = parseSvg(await loadSvgText(asset.svg));
        if (token !== this._token) return;
        if (!parsed) { warn(`invalid svg for "${name}"`); return; }
        const svg = finishSvg(document.importNode(parsed, true), {
          label: this.getAttribute('label') || asset.title || name,
          size: this.getAttribute('size'),
        });
        this.replaceChildren(svg);
      } catch (e) {
        warn(`could not load "${name}" (${e.message})`);
      }
    }
  }
  registry.define('cybercore-brand', CybercoreBrand);
}

if (globalThis.customElements) defineBrandElement();
