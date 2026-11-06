/* Cybercore project-site kit: theme chooser mount + interactions.
 *
 * Themes, signature mashups and the menu itself come from the universal
 * chooser (kit/chooser.js), which reads the org registry at /data. Sites
 * no longer list their own mashups; SITE.mashups is ignored.
 *
 * The additive SITE.web contract applies allowlisted motion and effect
 * recipes without changing the legacy theme API.
 */
(() => {
  const root = document.documentElement;
  const SITE = window.SITE || {};
  const WEB = SITE.web || {};
  const MOTIONS = new Set(['full', 'reduced', 'none']);
  const EFFECTS = new Set(['scanline', 'grid-glow', 'telemetry-pulse']);
  const COMPONENTS = new Set(['theme-chooser', 'detail-dialog', 'install-console']);
  const display = s => s.replaceAll('-', ' ').toUpperCase();
  const $ = s => document.querySelector(s);

  /* Themes are handled by the universal chooser (kit/chooser.js): one menu,
     one registry, the same 600 themes + signature mashups on every site.
     This adapter swaps the old .theme-picker for <cybercore-theme-chooser>
     and copies the site's own button styling onto it, so it keeps the
     site's look. */
  const KIT = new URL('..', document.currentScript ? document.currentScript.src : 'https://cybercoretech.github.io/kit/v1/cybercore.js').href;
  function loadChooser() {
    if (!document.querySelector('link[data-ccx-css]')) {
      const link = document.createElement('link');
      link.rel = 'stylesheet'; link.href = `${KIT}chooser.css`; link.dataset.ccxCss = '';
      document.head.append(link);
    }
    return import(`${KIT}chooser.js`);
  }
  function adoptStyle(from, to) {
    const cs = getComputedStyle(from);
    const set = (k, v) => { if (v && v !== 'none' && v !== 'normal' && v !== 'rgba(0, 0, 0, 0)') to.style.setProperty(k, v); };
    set('--ccx-font', cs.fontFamily);
    if (parseFloat(cs.fontSize) >= 12) set('--ccx-size', cs.fontSize);
    set('--ccx-tracking', cs.letterSpacing);
    if (cs.textTransform === 'lowercase' || cs.textTransform === 'capitalize') to.style.setProperty('--ccx-case', cs.textTransform);
    set('--ccx-weight', cs.fontWeight);
    set('--ccx-radius', cs.borderTopLeftRadius === '0px' ? '' : cs.borderTopLeftRadius);
    set('--ccx-clip', cs.clipPath);
    if (cs.borderTopStyle !== 'none' && parseFloat(cs.borderTopWidth) > 0) set('--ccx-accent', cs.borderTopColor);
    set('--ccx-button-bg', cs.backgroundColor);
  }
  function mountChooser() {
    const picker = document.querySelector('.theme-picker');
    const legacy = picker && picker.querySelector('#themeButton');
    const el = document.createElement('cybercore-theme-chooser');
    if (SITE.defaultTheme) el.setAttribute('default', SITE.defaultTheme);
    if (picker) {
      if (legacy) adoptStyle(legacy, el);
      el.className = picker.className;
      if (picker.getAttribute('style')) el.setAttribute('style', `${picker.getAttribute('style')};${el.getAttribute('style') || ''}`);
      picker.replaceWith(el);
    } else {
      // No visible picker on this page: still apply the visitor's theme.
      el.hidden = true;
      document.body.append(el);
    }
  }
  document.addEventListener('cc:theme', e => {
    const { id, palette: p, label } = e.detail || {};
    if (!id) return;
    const mash = window.CybercoreThemes && window.CybercoreThemes.blendOf(id);
    document.querySelectorAll('[data-active-theme]').forEach(el => { el.textContent = label || display(id); });
    const recipe = $('#themeRecipe');
    if (recipe) recipe.textContent = mash ? `${display(mash.base)} × ${display(mash.accent)}` : 'FAMILY PALETTE';
    const sw = $('#swatches');
    if (sw && p) sw.innerHTML = ['acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red'].map(k => `<i title="${k}" style="background:#${p[k]}"></i>`).join('');
  });
  document.addEventListener('cc:themes-ready', e => {
    const count = $('#themeCount');
    if (count) count.textContent = `${e.detail.total} THEMES`;
  });

  function applyWebRecipes() {
    const requestedMotion = MOTIONS.has(WEB.motion) ? WEB.motion : 'full';
    const motion = matchMedia('(prefers-reduced-motion: reduce)').matches
      ? 'reduced'
      : requestedMotion;
    root.dataset.ccMotion = motion;

    const effects = WEB.effects && typeof WEB.effects === 'object' ? WEB.effects : {};
    document.querySelectorAll('[data-cc-effect]').forEach(el => {
      const name = el.dataset.ccEffect;
      if (!EFFECTS.has(name)) {
        console.warn(`Cybercore Web Kit: ignored unknown effect recipe "${name}"`);
        return;
      }
      const options = effects[name];
      if (options && typeof options === 'object') {
        for (const [key, value] of Object.entries(options)) {
          if (typeof value === 'number' && Number.isFinite(value)) {
            el.style.setProperty(`--cc-${name}-${key}`, String(value));
          }
        }
      }
    });

    document.querySelectorAll('[data-cc-component]').forEach(el => {
      const name = el.dataset.ccComponent;
      if (!COMPONENTS.has(name)) {
        console.warn(`Cybercore Web Kit: ignored unknown component recipe "${name}"`);
        return;
      }
      el.dataset.ccComponentReady = name;
    });
  }

  window.CybercoreWebKit = Object.freeze({
    version: '1.0.0',
    applyRecipes: applyWebRecipes
  });

  /* ---------- ambience ---------- */
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
  applyWebRecipes();
  window.addEventListener('pointermove', e => { root.style.setProperty('--x', `${e.clientX}px`); root.style.setProperty('--y', `${e.clientY}px`); }, { passive: true });
  if (!still) {
    const brand = $('.brand');
    if (brand) { let phase = 0; setInterval(() => { phase = (phase + 1 + Math.floor(Math.random() * 5)) % 6; brand.dataset.brandPhase = phase; }, 4700); }
    // Type-on terminal lines.
    document.querySelectorAll('[data-type]').forEach(block => {
      const lines = [...block.children]; lines.forEach(l => { l.style.visibility = 'hidden'; });
      const io = new IntersectionObserver(entries => {
        if (!entries.some(e => e.isIntersecting)) return; io.disconnect();
        lines.forEach((l, i) => setTimeout(() => { l.style.visibility = 'visible'; l.classList.add('typed'); }, 160 * i));
      }, { threshold: 0.3 });
      io.observe(block);
    });
    // Reveal-on-scroll.
    const ro = new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) { e.target.classList.add('in'); ro.unobserve(e.target); } }), { threshold: 0.12 });
    document.querySelectorAll('.reveal').forEach(el => ro.observe(el));
  } else {
    document.querySelectorAll('.reveal').forEach(el => el.classList.add('in'));
  }
  // Copy buttons on code blocks.
  document.querySelectorAll('pre[data-copy]').forEach(pre => {
    const b = document.createElement('button'); b.className = 'copy'; b.type = 'button'; b.textContent = 'COPY';
    b.addEventListener('click', async () => { try { await navigator.clipboard.writeText(pre.querySelector('code').innerText); b.textContent = 'COPIED'; setTimeout(() => { b.textContent = 'COPY'; }, 1400); } catch { b.textContent = 'SELECT + COPY'; } });
    pre.append(b);
  });
  const year = $('#year'); if (year) year.textContent = new Date().getFullYear();

  mountChooser();
  loadChooser().catch(e => console.warn('Cybercore theme chooser unavailable', e));
})();
