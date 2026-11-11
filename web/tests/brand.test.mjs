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

import { isUnsafeUrl, isUnsafeAttribute, isUnsafeElement, cleanSvgTree } from '../brand.js';

test('isUnsafeUrl catches decoded obfuscations', () => {
  for (const bad of ['javascript:x()', 'JaVaScRiPt:x()', '  javascript:x()', '\x01javascript:x()', 'java\tscript:x()', 'java\nscr ipt:x()', '\x00j a\x1fvascript:x()'])
    assert.equal(isUnsafeUrl(bad), true, JSON.stringify(bad));
  for (const ok of ['#a', 'https://cybercoretech.net/x', 'svg/logo.svg', '', 'javascripty', null, undefined])
    assert.equal(isUnsafeUrl(ok), false, String(ok));
});

test('isUnsafeAttribute flags handlers and javascript hrefs only', () => {
  assert.equal(isUnsafeAttribute('onload', 'x()'), true);
  assert.equal(isUnsafeAttribute('OnClick', 'x()'), true);
  assert.equal(isUnsafeAttribute('href', 'javascript:x()'), true);
  assert.equal(isUnsafeAttribute('xlink:href', ' JavaScript:x()'), true);
  assert.equal(isUnsafeAttribute('XLINK:HREF', 'java\tscript:x()'), true);
  assert.equal(isUnsafeAttribute('href', '#a'), false);
  assert.equal(isUnsafeAttribute('fill', 'var(--cc-brand-bg, #000)'), false);
  assert.equal(isUnsafeAttribute('x', '1'), false);
});

test('isUnsafeElement flags script, foreignObject and href animation', () => {
  assert.equal(isUnsafeElement('script', null), true);
  assert.equal(isUnsafeElement('SCRIPT', null), true);
  assert.equal(isUnsafeElement('foreignObject', null), true);
  assert.equal(isUnsafeElement('foreignobject', null), true);
  for (const el of ['set', 'animate', 'animateTransform'])
    for (const a of ['href', 'xlink:href', ' HREF ', 'XLink:Href'])
      assert.equal(isUnsafeElement(el, a), true, `${el} ${a}`);
  assert.equal(isUnsafeElement('animate', 'fill'), false);
  assert.equal(isUnsafeElement('set', null), false);
  assert.equal(isUnsafeElement('rect', null), false);
});

test('cleanSvgTree walks a tree using only the predicates', () => {
  const mk = (name, attrs = {}, kids = []) => {
    const el = { localName: name, children: [], attributes: Object.entries(attrs).map(([name, value]) => ({ name, value })),
      getAttribute: k => attrs[k] ?? null, removeAttribute(k) { this.attributes = this.attributes.filter(a => a.name !== k); },
      remove() { this.parent.children = this.parent.children.filter(c => c !== this); } };
    kids.forEach(k => { k.parent = el; el.children.push(k); });
    return el;
  };
  const root = mk('svg', { onload: 'x()' }, [
    mk('script'), mk('rect', { x: '1', onclick: 'y()' }),
    mk('a', { 'xlink:href': '\x01javascript:x()', href: '#a' }),
    mk('set', { attributeName: 'href', to: 'javascript:x()' }), mk('g', {}, [mk('foreignObject')]),
  ]);
  cleanSvgTree(root);
  assert.deepEqual(root.children.map(c => c.localName), ['rect', 'a', 'g']);
  assert.deepEqual(root.attributes, []);
  assert.deepEqual(root.children[1].attributes.map(a => a.name), ['href']);
  assert.deepEqual(root.children[2].children, []);
});
