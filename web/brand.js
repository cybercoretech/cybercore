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

export function sanitizeSvg(markup) {
  return String(markup)
    .replace(/<script\b[\s\S]*?<\/script\s*>/gi, '')
    .replace(/<script\b[^>]*\/>/gi, '')
    .replace(/<foreignObject\b[\s\S]*?<\/foreignObject\s*>/gi, '')
    .replace(/<foreignObject\b[^>]*\/>/gi, '')
    .replace(/\son[a-z]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi, '')
    .replace(/\s(?:xlink:)?href\s*=\s*(?:"\s*javascript:[^"]*"|'\s*javascript:[^']*'|\s*javascript:[^\s>]*)/gi, '');
}

// Decision predicates. The parser hands these already-decoded values, so
// character references and tab/newline splitting are handled by the parser.
// Decisions use local names and namespaces, never the prefix a file chose.
const SVG_NS = 'http://www.w3.org/2000/svg';

export function isUnsafeUrl(value) {
  // eslint-disable-next-line no-control-regex
  return /^(?:javascript|vbscript|data):/i.test(String(value ?? '').replace(/[\u0000- ]/g, ''));
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
        const parsed = parseSvg(await res.text());
        if (token !== this._token) return;
        if (!parsed) { warn(`invalid svg for "${name}"`); return; }
        const svg = document.importNode(parsed, true);
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
