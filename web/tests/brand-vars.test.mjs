import { test } from 'node:test';
import assert from 'node:assert/strict';
import { BRAND_SLOTS, brandVarName, brandVarEntries, paintBrandVars } from '../brand-vars.js';

function fakeStyle() {
  const m = new Map();
  m.setProperty = (k, v) => { Map.prototype.set.call(m, k, v); };
  m.removeProperty = (k) => { m.delete(k); };
  return m;
}

test('maps slots to brand variables', () => {
  assert.equal(brandVarName('hot_pink'), '--cc-brand-hot-pink');
  assert.deepEqual(brandVarEntries(null), []);
  assert.deepEqual(brandVarEntries({ cyan: '00e5ff', bg: '09080f' }),
    [['--cc-brand-bg', '#09080f'], ['--cc-brand-cyan', '#00e5ff']]);
});

test('BRAND_SLOTS lists the 11 chooser slots in order', () => {
  assert.deepEqual(BRAND_SLOTS, ['bg', 'white', 'panel', 'line', 'muted', 'acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red']);
});

test('brand variables follow chosen themes and clear on site default', () => {
  const style = fakeStyle(); let painted = [];
  paintBrandVars(style, { cyan: '00e5ff' }, true, painted);
  assert.equal(style.get('--cc-brand-cyan'), '#00e5ff');
  assert.deepEqual(painted, ['--cc-brand-cyan']);
  painted.forEach(v => style.removeProperty(v)); painted = [];
  paintBrandVars(style, { cyan: '00e5ff' }, false, painted);
  assert.equal(style.get('--cc-brand-cyan'), undefined);
  assert.deepEqual(painted, []);
});
