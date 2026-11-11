import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolveAsset, defineBrandElement, finishSvg, loadIndex, loadSvgText } from '../brand.js';

test('resolveAsset returns null for unknown names', () => {
  const index = { assets: [{ id: 'logo.symbol', svg: 'svg/logo/symbol.svg', title: 'Cybercore' }] };
  assert.equal(resolveAsset(index, 'logo.symbol').svg, 'svg/logo/symbol.svg');
  assert.equal(resolveAsset(index, 'logo.nope'), null);
  assert.equal(resolveAsset(null, 'logo.symbol'), null);
});

test('defineBrandElement is side-effect free without a registry and defines once', () => {
  assert.doesNotThrow(() => defineBrandElement(undefined));
  const defined = new Map();
  const reg = { get: n => defined.get(n), define: (n, c) => defined.set(n, c) };
  globalThis.HTMLElement ??= class {};
  defineBrandElement(reg); defineBrandElement(reg);
  assert.ok(defined.has('cybercore-brand'));
});

import { isUnsafeUrl, isUnsafeAttribute, isUnsafeElement, isSvgRoot, cleanSvgTree } from '../brand.js';

const SVG = 'http://www.w3.org/2000/svg';
const XHTML = 'http://www.w3.org/1999/xhtml';

test('isUnsafeUrl catches decoded obfuscations', () => {
  for (const bad of ['javascript:x()', 'JaVaScRiPt:x()', '  javascript:x()', '\x01javascript:x()', 'java\tscript:x()', 'java\nscr ipt:x()', '\x00j a\x1fvascript:x()',
    'data:text/html,<script>', ' DATA:text/html,x', 'vbscript:x', 'vb\tscript:x'])
    assert.equal(isUnsafeUrl(bad), true, JSON.stringify(bad));
  for (const ok of ['#a', 'https://cybercoretech.net/x', 'svg/logo.svg', '', 'javascripty', null, undefined])
    assert.equal(isUnsafeUrl(ok), false, String(ok));
});

test('isUnsafeUrl allows only embedded raster data images', () => {
  for (const ok of ['data:image/png;base64,iVBOR', 'DATA:image/JPEG;base64,/9j/', 'data:image/gif,GIF8', 'data:image/webp;base64,UklG', ' data:image/png;base64,AA', 'da\tta:image/png;base64,AA'])
    assert.equal(isUnsafeUrl(ok), false, ok);
  for (const bad of ['data:image/svg+xml;base64,PHN2Zz4=', 'data:image/svg+xml,<svg/>', 'data:text/html,<script>', 'data:', 'data:image/png', 'data:image/pngx;base64,AA', 'data:application/javascript,x', 'vbscript:x'])
    assert.equal(isUnsafeUrl(bad), true, bad);
});

test('isUnsafeAttribute decides on the local name, any namespace prefix', () => {
  assert.equal(isUnsafeAttribute('onload', 'x()'), true);
  assert.equal(isUnsafeAttribute('OnClick', 'x()'), true);
  assert.equal(isUnsafeAttribute('href', 'javascript:x()'), true);
  assert.equal(isUnsafeAttribute('HREF', ' JavaScript:x()'), true);
  assert.equal(isUnsafeAttribute('href', 'data:image/svg+xml,x'), true);
  for (const n of ['src', 'srcdoc', 'data']) assert.equal(isUnsafeAttribute(n, 'javascript:x()'), true, n);
  assert.equal(isUnsafeAttribute('href', '#a'), false);
  assert.equal(isUnsafeAttribute('fill', 'var(--cc-brand-bg, #000)'), false);
  assert.equal(isUnsafeAttribute('x', '1'), false);
});

test('isUnsafeElement allows only SVG-namespace elements and blocks script/animation of href', () => {
  assert.equal(isUnsafeElement(SVG, 'rect', null), false);
  assert.equal(isUnsafeElement(XHTML, 'iframe', null), true);
  assert.equal(isUnsafeElement(XHTML, 'rect', null), true);
  assert.equal(isUnsafeElement(null, 'rect', null), true);
  assert.equal(isUnsafeElement(SVG, 'script', null), true);
  assert.equal(isUnsafeElement(SVG, 'SCRIPT', null), true);
  assert.equal(isUnsafeElement(SVG, 'foreignObject', null), true);
  for (const el of ['set', 'animate', 'animateTransform', 'animateMotion'])
    for (const a of ['href', 'xlink:href', ' HREF ', 'XLink:Href', 'x:href', 'a:b:href'])
      assert.equal(isUnsafeElement(SVG, el, a), true, `${el} ${a}`);
  assert.equal(isUnsafeElement(SVG, 'animate', 'fill'), false);
  assert.equal(isUnsafeElement(SVG, 'animate', 'x:fill'), false);
  assert.equal(isUnsafeElement(SVG, 'set', null), false);
});

test('isSvgRoot requires the SVG namespace', () => {
  assert.equal(isSvgRoot({ namespaceURI: SVG, localName: 'svg' }), true);
  assert.equal(isSvgRoot({ namespaceURI: XHTML, localName: 'svg' }), false);
  assert.equal(isSvgRoot({ namespaceURI: SVG, localName: 'g' }), false);
  assert.equal(isSvgRoot(null), false);
});

test('cleanSvgTree walks a tree using only the predicates', () => {
  const attrList = attrs => Object.entries(attrs).map(([name, value]) => {
    const [pre, local] = name.includes(':') ? name.split(':') : [null, name];
    return { name, localName: local, namespaceURI: pre === 'x' ? 'http://www.w3.org/1999/xlink' : null, value };
  });
  const mk = (name, attrs = {}, kids = [], ns = SVG) => {
    const el = { localName: name, namespaceURI: ns, children: [], attributes: attrList(attrs),
      getAttribute: k => attrs[k] ?? null, removeAttribute(k) { this.attributes = this.attributes.filter(a => a.name !== k); },
      remove() { this.parent.children = this.parent.children.filter(c => c !== this); } };
    kids.forEach(k => { k.parent = el; el.children.push(k); });
    return el;
  };
  const root = mk('svg', { onload: 'x()' }, [
    mk('script'), mk('rect', { x: '1', onclick: 'y()' }),
    mk('a', { 'x:href': '\x01javascript:x()', href: '#a' }),
    mk('set', { attributeName: 'x:href', to: 'javascript:x()' }), mk('g', {}, [mk('foreignObject')]),
    mk('iframe', { srcdoc: '<script>' }, [], XHTML),
  ]);
  cleanSvgTree(root);
  assert.deepEqual(root.children.map(c => c.localName), ['rect', 'a', 'g']);
  assert.deepEqual(root.attributes, []);
  assert.deepEqual(root.children[1].attributes.map(a => a.name), ['href']);
  assert.deepEqual(root.children[2].children, []);
});

// A tiny stand-in for an Element with attribute storage.
function fakeEl(attrs = {}) {
  const map = new Map(Object.entries(attrs));
  return {
    getAttribute: k => (map.has(k) ? map.get(k) : null),
    hasAttribute: k => map.has(k),
    setAttribute: (k, v) => map.set(k, String(v)),
    removeAttribute: k => map.delete(k),
    attrs: map,
  };
}

test('finishSvg drops aria-labelledby/describedby so the label attribute names the image', () => {
  const svg = fakeEl({ role: 'img', 'aria-labelledby': 'logo-symbol-title logo-symbol-desc', 'aria-describedby': 'logo-symbol-desc', viewBox: '0 0 512 512' });
  finishSvg(svg, { label: 'Cybercore' });
  assert.equal(svg.getAttribute('aria-labelledby'), null);
  assert.equal(svg.getAttribute('aria-describedby'), null);
  assert.equal(svg.getAttribute('role'), 'img');
  assert.equal(svg.getAttribute('aria-label'), 'Cybercore');
});

test('finishSvg size sets the height only so wide art keeps its aspect ratio', () => {
  const svg = fakeEl({ width: '1200', height: '300', viewBox: '0 0 1200 300' });
  finishSvg(svg, { label: 'x', size: '48' });
  assert.equal(svg.getAttribute('height'), '48');
  assert.equal(svg.getAttribute('width'), null);
  const keep = fakeEl({ width: '1200', height: '300' });
  finishSvg(keep, { label: 'x' });
  assert.deepEqual([keep.getAttribute('width'), keep.getAttribute('height')], ['1200', '300']);
});

test('loadIndex forgets a failed load so the next element retries', async () => {
  const real = globalThis.fetch;
  let calls = 0;
  try {
    globalThis.fetch = async () => { calls++; return { ok: false, status: 503 }; };
    await assert.rejects(loadIndex(), /503/);
    globalThis.fetch = async () => { calls++; return { ok: true, json: async () => ({ assets: [] }) }; };
    assert.deepEqual(await loadIndex(), { assets: [] });
    assert.deepEqual(await loadIndex(), { assets: [] });
    assert.equal(calls, 2);
  } finally { globalThis.fetch = real; }
});

test('loadSvgText fetches each SVG URL once and retries after a failure', async () => {
  const real = globalThis.fetch;
  const seen = [];
  try {
    globalThis.fetch = async url => { seen.push(String(url)); return { ok: true, text: async () => '<svg/>' }; };
    const [a, b] = await Promise.all([loadSvgText('svg/logo/symbol.svg'), loadSvgText('svg/logo/symbol.svg')]);
    assert.equal(a, '<svg/>'); assert.equal(b, '<svg/>');
    assert.equal(await loadSvgText('svg/logo/symbol.svg'), '<svg/>');
    assert.equal(seen.length, 1);
    assert.ok(seen[0].endsWith('/brand/svg/logo/symbol.svg'));
    globalThis.fetch = async url => { seen.push(String(url)); return { ok: false, status: 404 }; };
    await assert.rejects(loadSvgText('svg/logo/nope.svg'), /404/);
    globalThis.fetch = async url => { seen.push(String(url)); return { ok: true, text: async () => '<svg id="2"/>' }; };
    assert.equal(await loadSvgText('svg/logo/nope.svg'), '<svg id="2"/>');
  } finally { globalThis.fetch = real; }
});

test('sanitizeSvg is gone (the render path is parseSvg + cleanSvgTree)', async () => {
  const mod = await import('../brand.js');
  assert.equal(mod.sanitizeSvg, undefined);
});
