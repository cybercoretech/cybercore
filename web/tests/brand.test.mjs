import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resolveAsset, sanitizeSvg, defineBrandElement } from '../brand.js';

test('resolveAsset returns null for unknown names', () => {
  const index = { assets: [{ id: 'logo.symbol', svg: 'svg/logo/symbol.svg', title: 'Cybercore' }] };
  assert.equal(resolveAsset(index, 'logo.symbol').svg, 'svg/logo/symbol.svg');
  assert.equal(resolveAsset(index, 'logo.nope'), null);
  assert.equal(resolveAsset(null, 'logo.symbol'), null);
});

test('sanitizeSvg strips active content', () => {
  const out = sanitizeSvg('<svg onload="x()"><script>x()</script><a href="javascript:x()"><rect onclick="y()"/></a><foreignObject><p/></foreignObject></svg>');
  for (const bad of ['<script', 'onload', 'onclick', 'javascript:', 'foreignObject']) assert.ok(!out.includes(bad), bad);
  assert.ok(out.includes('<rect'));
});

test('sanitizeSvg strips xlink:href javascript and single-quoted handlers, keeps safe hrefs', () => {
  const out = sanitizeSvg(`<svg><use xlink:href=" JavaScript:x()"/><g onclick='z()'/><use href="#a"/></svg>`);
  assert.ok(!/javascript:/i.test(out));
  assert.ok(!out.includes('onclick'));
  assert.ok(out.includes('href="#a"'));
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
