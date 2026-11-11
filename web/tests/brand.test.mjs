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
