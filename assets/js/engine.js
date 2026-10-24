/* CYBERCORE theme engine.
 *
 * One registry for every Cybercore site, served by the org site:
 *   <dataRoot>/cybergrid.json                    families → theme names
 *   <dataRoot>/themes/<family>/<name>.json       11-slot palettes
 * On top of that:
 *   - SITE.mashups: signature blends (surfaces from `base`, neons from `accent`)
 *   - lab blends:   "mix:<base>:<accent>", built in the Mashup Lab and shareable by URL
 * The choice is stored under `cybercore-theme`, the same key the project sites use.
 * Anything that wants the colours listens for the `cc:theme` event.
 */
(() => {
  const root = document.documentElement;
  const SITE = window.SITE || { mashups: [] };
  const DATA = (SITE.dataRoot || '/data').replace(/\/$/, '');
  const STORE = 'cybercore-theme';
  const SLOTS = ['bg', 'white', 'panel', 'line', 'muted', 'acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red'];
  const SURFACE = ['bg', 'white', 'panel', 'line', 'muted'];
  const NEON = ['acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red'];
  const VARS = { bg: '--bg', white: '--white', acid_green: '--acid', hot_pink: '--pink', purple: '--purple', cyan: '--cyan', orange: '--orange', red: '--red', panel: '--panel', line: '--line', muted: '--muted' };
  const SLOT_INFO = {
    bg: 'BACKGROUND', white: 'TEXT', panel: 'PANEL', line: 'LINE', muted: 'MUTED',
    acid_green: 'ACID', hot_pink: 'HOT PINK', purple: 'PURPLE', cyan: 'CYAN', orange: 'ORANGE', red: 'RED'
  };
  const FALLBACK = { bg: '09080f', white: 'f0f0f2', acid_green: '39ff33', hot_pink: 'ff147f', purple: '8a22e2', cyan: '00e5ff', orange: 'ff9000', red: 'f1142d', panel: '16151f', line: '32303e', muted: '807f8b' };
  const LABELS = { mashup: 'SIGNATURE MASHUPS', 'cybercore-tech': 'CYBERCORE TECH / MASTER MIX', cyberdyne: 'CYBERDYNE', cyberpunk: 'CYBERPUNK', default: 'DEFAULT / CLASSICS', dystopian: 'DYSTOPIAN', neosynth: 'NEOSYNTH', synthwave: 'SYNTHWAVE' };
  const SHORT = { mashup: 'MASHUPS', 'cybercore-tech': 'MASTER', cyberdyne: 'CYBERDYNE', cyberpunk: 'CYBERPUNK', default: 'CLASSICS', dystopian: 'DYSTOPIAN', neosynth: 'NEOSYNTH', synthwave: 'SYNTHWAVE' };
  const $ = s => document.querySelector(s);
  const $$ = s => [...document.querySelectorAll(s)];
  const display = s => s.replaceAll('-', ' ').toUpperCase();
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches;

  let families = {};
  let current = null;
  let currentPalette = FALLBACK;
  const cache = {};

  /* ---------- registry ---------- */
  const familyOf = name => Object.keys(families).find(f => families[f].includes(name));
  const allNames = () => Object.values(families).flat();
  const mashOf = id => SITE.mashups.find(m => m.id === id);
  const parseMix = id => {
    if (!id || !id.startsWith('mix:')) return null;
    const [, base, accent] = id.split(':');
    return familyOf(base) && familyOf(accent) ? { id, base, accent, label: `${display(base)} × ${display(accent)}` } : null;
  };
  const blendOf = id => mashOf(id) || parseMix(id);
  const known = id => Boolean(id && (blendOf(id) || familyOf(id)));
  const labelOf = id => { const b = blendOf(id); return b ? b.label : display(id); };
  const familyLabel = id => mashOf(id) ? 'SIGNATURE MASHUP' : parseMix(id) ? 'LAB MASHUP' : (LABELS[familyOf(id)] || '');

  async function palette(name) {
    if (cache[name]) return cache[name];
    const family = familyOf(name);
    if (!family) throw new Error(`unknown theme ${name}`);
    const res = await fetch(`${DATA}/themes/${family}/${name}.json`);
    if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
    return (cache[name] = await res.json());
  }
  function blend(base, accent) {
    const out = { ...accent };
    SURFACE.forEach(k => { out[k] = base[k]; });
    return out;
  }
  async function resolve(id) {
    const b = blendOf(id);
    if (!b) return palette(id);
    const [base, accent] = await Promise.all([palette(b.base), palette(b.accent)]);
    return blend(base, accent);
  }

  /* ---------- painting ---------- */
  function writeVars(el, p) {
    Object.entries(VARS).forEach(([k, v]) => p[k] && el.style.setProperty(v, `#${p[k]}`));
  }
  function paint(id, p) {
    writeVars(root, p);
    root.style.setProperty('--fg', `#${p.white}`);
    root.dataset.theme = id;
    current = id; currentPalette = p;
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.content = `#${p.bg}`;
    $$('[data-active-theme]').forEach(el => { el.textContent = labelOf(id); });
    $$('[data-active-theme-raw]').forEach(el => { el.textContent = id; });
    $$('[data-active-family]').forEach(el => { el.textContent = familyLabel(id); });
    $$('[data-active-path]').forEach(el => { el.textContent = blendOf(id) ? 'mashup' : id; });
    $$('[data-slot-hex]').forEach(el => { el.textContent = `"${p[el.dataset.slotHex]}"`; });
    $$('[data-slot-css]').forEach(el => { el.textContent = `"#${p[el.dataset.slotCss]}"`; });
    const hs = $('#hudSwatches');
    if (hs) hs.innerHTML = NEON.map(k => `<i style="background:#${p[k]};color:#${p[k]}"></i>`).join('');
    const label = $('#themeLabel');
    if (label) label.textContent = labelOf(id);
    renderSlots(p); renderJson(id, p); syncMenus(id); syncCards(id);
    document.dispatchEvent(new CustomEvent('cc:theme', { detail: { id, palette: p } }));
  }

  /* Apply with a circular reveal from `origin` (View Transitions), plus the CRT tear. */
  async function apply(id, { persist = true, origin = null } = {}) {
    let p;
    try { p = await resolve(id); } catch (e) { console.warn('theme', e); id = 'cybercore-tech'; p = FALLBACK; }
    const go = () => paint(id, p);
    if (origin && !still && document.startViewTransition) {
      root.style.setProperty('--vx', `${origin.x}px`); root.style.setProperty('--vy', `${origin.y}px`);
      document.startViewTransition(go);
    } else go();
    if (!still && current !== null) {
      const f = $('#flash'); if (f) { f.classList.remove('go'); void f.offsetWidth; f.classList.add('go'); }
    }
    if (persist) {
      try { localStorage.setItem(STORE, id); } catch {}
      const url = new URL(location.href); url.searchParams.set('theme', id); history.replaceState(null, '', url);
    }
  }
  const originOf = ev => ev && 'clientX' in ev && ev.clientX ? { x: ev.clientX, y: ev.clientY } : (ev && ev.currentTarget && ev.currentTarget.getBoundingClientRect ? (r => ({ x: r.left + r.width / 2, y: r.top + r.height / 2 }))(ev.currentTarget.getBoundingClientRect()) : null);

  /* ---------- 01: slots + live JSON ---------- */
  function renderSlots(p) {
    const el = $('#slots'); if (!el) return;
    if (!el.children.length) {
      el.innerHTML = SLOTS.map(k => `<button type="button" class="slot ${SURFACE.includes(k) ? 'surface' : 'neon'}" data-k="${k}"><small>${SURFACE.includes(k) ? 'SURFACE' : 'NEON'}</small><b>${SLOT_INFO[k]}</b><code>${VARS[k]}</code><code class="hex"></code></button>`).join('')
        + `<div class="slot" style="--c:transparent;border-style:dashed;cursor:default"><small>RULE</small><b>NO HARD-CODED HEX</b><code>read a slot</code></div>`;
      el.querySelectorAll('button.slot').forEach(b => b.addEventListener('click', async () => {
        const hex = `#${currentPalette[b.dataset.k]}`;
        try { await navigator.clipboard.writeText(hex); toast(`COPIED ${hex.toUpperCase()}`); } catch { toast(hex.toUpperCase()); }
        b.classList.add('copied'); setTimeout(() => b.classList.remove('copied'), 1200);
      }));
    }
    el.querySelectorAll('button.slot').forEach(b => {
      const k = b.dataset.k;
      b.style.setProperty('--c', `#${p[k]}`);
      b.querySelector('.hex').textContent = `#${p[k]}`.toUpperCase();
      // dark text on light slots
      const [r, g, bl] = [0, 2, 4].map(i => parseInt(p[k].slice(i, i + 2), 16));
      b.style.color = (r * 299 + g * 587 + bl * 114) / 1000 > 150 ? '#0a0a0a' : '';
    });
  }
  function renderJson(id, p) {
    const code = $('#liveJson code'); if (!code) return;
    const b = blendOf(id);
    const head = b ? `<span class="c">// ${b.label}: ${b.base} surfaces × ${b.accent} neons</span>\n` : `<span class="c">// ${familyOf(id) || ''}/${id}.json</span>\n`;
    code.innerHTML = head + '{\n' + SLOTS.map((k, i) =>
      `  <span class="k">"${k}"</span>: <i class="sw" style="background:#${p[k]}"></i><span class="h">"${p[k]}"</span>${i < SLOTS.length - 1 ? ',' : ''}`).join('\n') + '\n}';
  }

  /* ---------- dropdown menus (header + lab) ---------- */
  const groups = () => [['mashup', SITE.mashups.map(m => m.id)]].concat(Object.entries(families));
  function buildList(list, { onPick, include = () => true, withMashups = true }) {
    list.innerHTML = '';
    groups().forEach(([family, names]) => {
      if (family === 'mashup' && !withMashups) return;
      const items = names.filter(include);
      if (!items.length) return;
      const h = document.createElement('div');
      h.className = 'theme-group-label'; h.dataset.family = family;
      h.innerHTML = `<svg aria-hidden="true"><use href="#fam-${family}"/></svg>${LABELS[family] || family.toUpperCase()}<em>${items.length}</em>`;
      list.append(h);
      items.forEach((name, i) => {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'theme-option'; b.setAttribute('role', 'option');
        b.dataset.value = name; b.dataset.family = family; b.dataset.tone = i % 2 ? 'alt' : 'base';
        b.innerHTML = `<span>${labelOf(name)}</span><span class="dots"></span>`;
        b.addEventListener('click', ev => onPick(name, ev));
        list.append(b);
        dotsLater(b, name);
      });
    });
  }
  // swatch dots fill in as palettes arrive (preloadAll)
  const pendingDots = [];
  function dotsLater(btn, name) { pendingDots.push([btn, name]); }
  function flushDots() {
    pendingDots.forEach(([btn, name]) => {
      const p = cache[name] || (blendOf(name) && cache[blendOf(name).base] && cache[blendOf(name).accent] && blend(cache[blendOf(name).base], cache[blendOf(name).accent]));
      if (p) btn.querySelector('.dots').innerHTML = ['acid_green', 'hot_pink', 'cyan'].map(k => `<i style="background:#${p[k]}"></i>`).join('');
    });
  }

  function wireMenu(button, menu, list, filter) {
    const open = () => {
      menu.hidden = false; button.setAttribute('aria-expanded', 'true');
      const sel = list.querySelector('[aria-selected="true"]');
      if (sel) list.scrollTop = sel.offsetTop - list.clientHeight / 2;
      if (filter && matchMedia('(pointer:fine)').matches) filter.focus({ preventScroll: true });
    };
    const close = () => { menu.hidden = true; button.setAttribute('aria-expanded', 'false'); if (filter) { filter.value = ''; runFilter(list, ''); } };
    button.addEventListener('click', () => (menu.hidden ? open() : close()));
    document.addEventListener('pointerdown', e => { if (!menu.hidden && !menu.parentElement.contains(e.target)) close(); });
    menu.addEventListener('keydown', e => {
      const opts = [...list.querySelectorAll('.theme-option:not([hidden])')];
      const i = opts.indexOf(document.activeElement);
      if (e.key === 'Escape') { close(); button.focus(); }
      else if (e.key === 'ArrowDown') { e.preventDefault(); (opts[i + 1] || opts[0])?.focus(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); (i <= 0 ? filter || opts.at(-1) : opts[i - 1])?.focus(); }
      else if (e.key === 'Enter' && document.activeElement === filter && opts[0]) { e.preventDefault(); opts[0].click(); }
    });
    if (filter) filter.addEventListener('input', () => runFilter(list, filter.value));
    return { open, close };
  }
  function runFilter(list, q) {
    q = q.trim().toLowerCase().replaceAll(' ', '-');
    let lastHead = null, any = false;
    [...list.children].forEach(el => {
      if (el.classList.contains('theme-group-label')) { if (lastHead) lastHead.hidden = !any; lastHead = el; any = false; return; }
      const hit = !q || el.dataset.value.includes(q) || el.dataset.family.includes(q) || el.textContent.toLowerCase().replaceAll(' ', '-').includes(q);
      el.hidden = !hit; any = any || hit;
    });
    if (lastHead) lastHead.hidden = !any;
  }
  function syncMenus(id) {
    $$('#themeList .theme-option').forEach(o => o.setAttribute('aria-selected', String(o.dataset.value === id)));
  }

  let header;
  function buildHeader() {
    const list = $('#themeList');
    buildList(list, { onPick: (name, ev) => { apply(name, { origin: originOf(ev) }); header.close(); setTimeout(() => $('#themeButton').focus({ preventScroll: true })); } });
    header = wireMenu($('#themeButton'), $('#themeMenu'), list, $('#themeFilter'));
    const total = SITE.mashups.length + allNames().length;
    const f = $('#themeFilter'); if (f) f.placeholder = `FILTER ${total} THEMES…`;
    $$('[data-theme-count]').forEach(el => { el.textContent = allNames().length; });
  }

  /* ---------- 02: matrix ---------- */
  let activeFamily = 'all';
  function buildFamilies() {
    const el = $('#families'); if (!el) return;
    const fams = [['all', allNames()], ...groups()];
    el.innerHTML = fams.map(([f, names]) => `<button class="fam" role="tab" data-family="${f}" aria-selected="${f === activeFamily}" style="${f === 'all' ? '--fa:var(--white)' : ''}"><svg aria-hidden="true"><use href="#fam-${f === 'all' ? 'mashup' : f}"/></svg>${f === 'all' ? 'ALL' : SHORT[f]}<em>${names.length}</em></button>`).join('');
    el.querySelectorAll('.fam').forEach(b => b.addEventListener('click', () => {
      activeFamily = b.dataset.family;
      el.querySelectorAll('.fam').forEach(x => x.setAttribute('aria-selected', String(x === b)));
      renderCards();
    }));
  }
  function scene(id, p) {
    const m = mashOf(id) || parseMix(id);
    if (window.CC_SCENE) return window.CC_SCENE(id, p, familyOf(id), m && { baseFamily: familyOf(m.base), accentFamily: familyOf(m.accent) });
    return '';
  }
  function cardStyle(p) { return Object.entries({ bg: 'bg', white: 'white', panel: 'panel', line: 'line', muted: 'muted', acid: 'acid_green', pink: 'hot_pink', cyan: 'cyan' }).map(([v, k]) => `--t-${v}:#${p[k]}`).join(';'); }
  async function renderCards() {
    const el = $('#cards'); if (!el) return;
    const names = activeFamily === 'all' ? allNames() : activeFamily === 'mashup' ? SITE.mashups.map(m => m.id) : families[activeFamily] || [];
    el.innerHTML = '<div class="loading">DECRYPTING PALETTES…</div>';
    const pals = await Promise.all(names.map(n => resolve(n).catch(() => null)));
    el.innerHTML = names.map((n, i) => {
      const p = pals[i]; if (!p) return '';
      const fam = mashOf(n) ? 'SIGNATURE MASHUP' : (LABELS[familyOf(n)] || '');
      return `<button type="button" class="tcard" data-value="${n}" aria-pressed="${n === current}" style="${cardStyle(p)};--i:${i}">${scene(n, p)}
        <div class="t-meta"><div class="t-name">${labelOf(n)}</div><div class="t-dots">${NEON.map(k => `<i style="background:#${p[k]}"></i>`).join('')}</div><div class="t-fam">${fam}</div></div></button>`;
    }).join('');
    el.querySelectorAll('.tcard').forEach(c => c.addEventListener('click', ev => apply(c.dataset.value, { origin: originOf(ev) })));
    document.dispatchEvent(new CustomEvent('cc:cards'));
  }
  function syncCards(id) { $$('.tcard').forEach(c => c.setAttribute('aria-pressed', String(c.dataset.value === id))); }

  /* ---------- 03: mashup lab ---------- */
  const lab = { base: 'cyberdyne-steel', accent: 'synthwave-outrun' };
  const dds = {};
  function buildLab() {
    $$('.dd').forEach(dd => {
      const role = dd.dataset.dd;
      dd.innerHTML = `<button type="button" aria-haspopup="listbox" aria-expanded="false"><span></span><span class="dots"></span><svg class="chev" aria-hidden="true"><use href="#i-chev"/></svg></button><div class="theme-menu" hidden><div class="theme-search"><input type="search" placeholder="FILTER…" aria-label="Filter ${role} themes" autocomplete="off" spellcheck="false"></div><div class="theme-list" role="listbox"></div></div>`;
      const [btn, menu] = dd.children; const list = menu.querySelector('.theme-list');
      let ctl;
      buildList(list, { withMashups: false, onPick: name => { lab[role] = name; ctl.close(); updateLab(); } });
      ctl = wireMenu(btn, menu, list, menu.querySelector('input'));
      dds[role] = { btn, list };
    });
    const pre = $('#presets');
    if (pre) {
      pre.innerHTML = SITE.mashups.map(m => `<button type="button" class="chip" data-id="${m.id}">${m.label}</button>`).join('');
      pre.querySelectorAll('.chip').forEach(c => c.addEventListener('click', () => { const m = mashOf(c.dataset.id); lab.base = m.base; lab.accent = m.accent; updateLab(); }));
    }
    $('#labShuffle')?.addEventListener('click', () => { const all = allNames(); lab.base = pick(all); lab.accent = pick(all.filter(n => n !== lab.base)); updateLab(); });
    $('#labApply')?.addEventListener('click', ev => apply(labId(), { origin: originOf(ev) }));
    $('#labLink')?.addEventListener('click', async () => {
      const url = new URL(location.href); url.searchParams.set('theme', labId()); url.hash = 'lab';
      try { await navigator.clipboard.writeText(url.toString()); toast('LINK COPIED'); } catch { toast(url.toString()); }
    });
  }
  const labId = () => { const m = SITE.mashups.find(x => x.base === lab.base && x.accent === lab.accent); return m ? m.id : `mix:${lab.base}:${lab.accent}`; };
  async function updateLab() {
    const [b, a] = await Promise.all([palette(lab.base), palette(lab.accent)]).catch(() => [FALLBACK, FALLBACK]);
    const p = blend(b, a);
    const prev = $('#labPreview'); writeVars(prev, p);
    const m = SITE.mashups.find(x => x.base === lab.base && x.accent === lab.accent);
    $('#labRecipe').textContent = m ? `${m.label} · ${display(lab.base)} × ${display(lab.accent)}` : `${display(lab.base)} × ${display(lab.accent)}`;
    $('#labSwatches').innerHTML = SLOTS.map(k => `<i data-k="${k} #${p[k]}" style="background:#${p[k]}"></i>`).join('');
    $$('#presets .chip').forEach(c => c.setAttribute('aria-pressed', String(m && c.dataset.id === m.id)));
    ['base', 'accent'].forEach(role => {
      const d = dds[role]; if (!d) return;
      const src = role === 'base' ? b : a;
      d.btn.querySelector('span').textContent = display(lab[role]);
      d.btn.querySelector('.dots').innerHTML = (role === 'base' ? ['bg', 'panel', 'white'] : ['acid_green', 'hot_pink', 'cyan']).map(k => `<i style="background:#${src[k]};outline:1px solid #${src.line}"></i>`).join('');
      d.list.querySelectorAll('.theme-option').forEach(o => o.setAttribute('aria-selected', String(o.dataset.value === lab[role])));
    });
  }
  const pick = a => a[Math.floor(Math.random() * a.length)];

  /* ---------- toast ---------- */
  let tt;
  function toast(msg) { const t = $('#toast'); if (!t) return; t.textContent = msg; t.classList.add('on'); clearTimeout(tt); tt = setTimeout(() => t.classList.remove('on'), 1600); }

  /* ---------- keyboard ---------- */
  function cycle(dir) {
    const list = SITE.mashups.map(m => m.id).concat(allNames());
    const i = list.indexOf(current);
    apply(list[(i + dir + list.length) % list.length]);
  }
  document.addEventListener('keydown', e => {
    if (e.target.closest('input, textarea, [contenteditable]') || e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 't' || e.key === 'T') { e.preventDefault(); $('#themeButton').click(); }
    else if (e.key === 'r' || e.key === 'R') randomTheme();
    else if (e.key === ']') cycle(1);
    else if (e.key === '[') cycle(-1);
  });
  function randomTheme(ev) {
    const list = SITE.mashups.map(m => m.id).concat(allNames()).filter(n => n !== current);
    apply(pick(list), { origin: originOf(ev) || { x: innerWidth / 2, y: innerHeight / 2 } });
  }
  $('#shuffle')?.addEventListener('click', randomTheme);

  /* ---------- boot ---------- */
  async function preloadAll() {
    const warm = allNames().slice(0, 12);
    await Promise.all(warm.map(n => palette(n).catch(() => null)));
    flushDots();
  }
  async function boot() {
    try {
      const schema = await (await fetch(`${DATA}/cybergrid.json`)).json();
      families = schema.families || {};
    } catch (e) {
      console.warn('theme registry offline', e);
      $('#themeLabel').textContent = 'REGISTRY OFFLINE';
      paint('cybercore-tech', FALLBACK);
      return;
    }
    buildHeader(); buildFamilies(); buildLab();
    const params = new URLSearchParams(location.search);
    const requested = params.get('theme');
    let saved = null; try { saved = localStorage.getItem(STORE); } catch {}
    const start = [requested, saved, SITE.defaultTheme, 'cybercore-tech'].find(known);
    await apply(start, { persist: Boolean(requested) });
    const asBlend = blendOf(start);
    if (asBlend) { lab.base = asBlend.base; lab.accent = asBlend.accent; }
    const fam = mashOf(start) ? 'mashup' : familyOf(start);
    activeFamily = fam && fam !== 'cybercore-tech' && !parseMix(start) ? fam : 'synthwave';
    $$('#families .fam').forEach(x => x.setAttribute('aria-selected', String(x.dataset.family === activeFamily)));
    updateLab(); renderCards(); preloadAll();
  }

  window.CC = { apply, randomTheme, toast, get palette() { return currentPalette; }, get id() { return current; } };
  boot();
})();
