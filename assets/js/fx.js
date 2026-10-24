/* CYBERCORE effects layer. No dependencies.
 *
 *   boot        terminal boot sequence (once per session, skippable)
 *   synth       hero canvas: outrun sun, wireframe ridges, perspective grid
 *   trail       neon pointer trail
 *   rain        cyberdyne glyph rain behind the pipeline
 *   scramble    decode effect on section titles
 *   rotator     tagline word cycler
 *   magnetic    buttons that lean toward the pointer
 *   tilt        3D tilt + holo sheen on theme cards and grid tiles
 *   spot        pointer spotlight inside cards
 *   burst       particle burst on primary actions
 *   konami      ↑↑↓↓←→←→BA → OVERDRIVE
 * Every animated piece pauses offscreen and respects prefers-reduced-motion.
 */
(() => {
  const root = document.documentElement;
  const $ = s => document.querySelector(s);
  const $$ = s => [...document.querySelectorAll(s)];
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const fine = matchMedia('(pointer: fine)').matches;
  const DPR = Math.min(devicePixelRatio || 1, 2);
  const css = v => getComputedStyle(root).getPropertyValue(v).trim() || '#ff147f';
  let C = {};
  const readColors = () => { C = { bg: css('--bg'), white: css('--white'), pink: css('--pink'), cyan: css('--cyan'), acid: css('--acid'), purple: css('--purple'), orange: css('--orange'), red: css('--red'), panel: css('--panel'), line: css('--line'), muted: css('--muted') }; };
  readColors();
  document.addEventListener('cc:theme', () => requestAnimationFrame(readColors));
  const rgba = (hex, a) => { const h = hex.replace('#', ''); const n = parseInt(h.length === 3 ? h.replace(/./g, c => c + c) : h, 16); return `rgba(${n >> 16 & 255},${n >> 8 & 255},${n & 255},${a})`; };

  /* ---------- boot ---------- */
  (function boot() {
    let seen = false; try { seen = sessionStorage.getItem('cc-booted'); } catch {}
    if (seen || still || /[?&]noboot\b/.test(location.search)) return;
    root.classList.add('booting');
    const log = $('#bootLog'), bar = $('#bootBar'), el = $('#boot');
    const lines = [
      'CYBERCORE BIOS v0.5.0 <b>OK</b>',
      'mount schema/cybergrid.json <b>OK</b>',
      'load families: cyberdyne · synthwave · neosynth · cyberpunk · dystopian · classics',
      'decrypt 81 palettes × 11 slots <b>OK</b>',
      'link tokens · components · paths <b>OK</b>',
      'neural net <em>ONLINE</em>. welcome, operator.'
    ];
    let i = 0, done = false;
    const finish = () => { if (done) return; done = true; el.classList.add('done'); setTimeout(() => root.classList.remove('booting'), 500); try { sessionStorage.setItem('cc-booted', '1'); } catch {} };
    const step = () => {
      if (done) return;
      if (i >= lines.length) return setTimeout(finish, 260);
      const li = document.createElement('li'); li.innerHTML = '&gt; ' + lines[i++]; log.append(li);
      bar.style.width = `${(i / lines.length) * 100}%`;
      setTimeout(step, 150 + Math.random() * 120);
    };
    setTimeout(step, 200);
    ['keydown', 'pointerdown', 'wheel'].forEach(t => addEventListener(t, finish, { once: true, passive: true }));
  })();

  /* ---------- visibility helper ---------- */
  function whenVisible(el, cb) {
    const io = new IntersectionObserver(es => es.forEach(e => cb(e.isIntersecting)), { threshold: 0 });
    io.observe(el);
  }

  /* ---------- synthwave hero ---------- */
  (function synth() {
    const cv = $('#synth'); if (!cv) return;
    const ctx = cv.getContext('2d');
    let w, h, on = true, t0 = performance.now(), mx = 0, my = 0;
    const stars = Array.from({ length: 120 }, () => ({ x: Math.random(), y: Math.random() * .6, r: Math.random() * 1.2 + .2, p: Math.random() * 6 }));
    // two ridge lines, generated once
    const ridge = (n, amp, seed) => Array.from({ length: n + 1 }, (_, i) => { const x = i / n; return [x, amp * (.35 + .65 * Math.abs(Math.sin(i * 1.7 + seed) * Math.cos(i * .63 + seed * 2)))]; });
    const R1 = ridge(28, .16, 1.3), R2 = ridge(40, .1, 4.1);
    function size() { const r = cv.getBoundingClientRect(); w = r.width; h = r.height; cv.width = w * DPR; cv.height = h * DPR; ctx.setTransform(DPR, 0, 0, DPR, 0, 0); }
    size(); addEventListener('resize', size);
    if (fine) addEventListener('pointermove', e => { mx = e.clientX / innerWidth - .5; my = e.clientY / innerHeight - .5; }, { passive: true });
    whenVisible(cv, v => { on = v; if (v) loop(); });

    function frame(t) {
      const hz = h * (.6 + my * .03);                 // horizon
      const vx = w * (.68 - mx * .06);                // vanishing point x
      ctx.clearRect(0, 0, w, h);
      // sky
      const sky = ctx.createLinearGradient(0, 0, 0, hz);
      sky.addColorStop(0, rgba(C.bg, 0)); sky.addColorStop(.55, rgba(C.purple, .16)); sky.addColorStop(1, rgba(C.pink, .32));
      ctx.fillStyle = sky; ctx.fillRect(0, 0, w, hz);
      // stars
      stars.forEach(s => { ctx.globalAlpha = .35 + .35 * Math.sin(t / 900 + s.p); ctx.fillStyle = C.white; ctx.fillRect(s.x * w, s.y * hz, s.r, s.r); });
      ctx.globalAlpha = 1;
      // sun with scrolling slats
      const sr = Math.min(h * .27, w * .17), sx = vx, sy = hz - sr * .25;
      ctx.save();
      ctx.beginPath(); ctx.rect(0, 0, w, hz); ctx.clip();
      const sg = ctx.createLinearGradient(0, sy - sr, 0, sy + sr);
      sg.addColorStop(0, C.orange); sg.addColorStop(.55, C.pink); sg.addColorStop(1, C.purple);
      ctx.shadowColor = C.pink; ctx.shadowBlur = 60;
      ctx.fillStyle = sg; ctx.beginPath(); ctx.arc(sx, sy, sr, 0, Math.PI * 2); ctx.fill();
      ctx.shadowBlur = 0; ctx.globalCompositeOperation = 'destination-out';
      const off = (t / 60) % 14;
      for (let y = sy - sr * .1 + off, k = 0; y < sy + sr; y += 14, k++) { const th = 1.5 + ((y - sy) / sr) * 7; if (th > 0) ctx.fillRect(sx - sr, y, sr * 2, th); }
      ctx.restore(); ctx.globalCompositeOperation = 'source-over';
      // ridges (wireframe)
      const drawRidge = (R, base, fill, stroke, shift) => {
        ctx.beginPath(); ctx.moveTo(0, hz);
        R.forEach(([x, a]) => ctx.lineTo(x * w + shift, hz - a * h));
        ctx.lineTo(w, hz); ctx.closePath();
        ctx.fillStyle = fill; ctx.fill();
        ctx.strokeStyle = stroke; ctx.lineWidth = 1.2; ctx.shadowColor = stroke; ctx.shadowBlur = 8; ctx.stroke(); ctx.shadowBlur = 0;
        // inner wire verticals
        ctx.globalAlpha = .35; ctx.beginPath();
        R.forEach(([x, a]) => { ctx.moveTo(x * w + shift, hz - a * h); ctx.lineTo(vx + (x * w + shift - vx) * .6, hz); });
        ctx.stroke(); ctx.globalAlpha = 1;
      };
      drawRidge(R2, hz, rgba(C.panel, .92), rgba(C.purple, .9), mx * -20);
      drawRidge(R1, hz, rgba(C.bg, .96), C.cyan, mx * -40);
      // floor
      const fl = ctx.createLinearGradient(0, hz, 0, h);
      fl.addColorStop(0, rgba(C.purple, .35)); fl.addColorStop(1, rgba(C.bg, 1));
      ctx.fillStyle = fl; ctx.fillRect(0, hz, w, h - hz);
      ctx.strokeStyle = C.pink; ctx.shadowColor = C.pink; ctx.shadowBlur = 10; ctx.lineWidth = 1.2;
      const depth = h - hz, speed = (t / 1400) % 1;
      for (let i = 0; i < 18; i++) {                  // horizontals, perspective-spaced
        const z = (i + speed) / 18, y = hz + depth * z * z;
        ctx.globalAlpha = Math.min(1, z * 2.2); ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
      }
      ctx.globalAlpha = .85;
      for (let i = -24; i <= 24; i++) {               // verticals to the vanishing point
        ctx.beginPath(); ctx.moveTo(vx + i * 18, hz); ctx.lineTo(vx + i * w * .11, h); ctx.stroke();
      }
      ctx.shadowBlur = 0; ctx.globalAlpha = 1;
      // horizon line + cyberdyne scan sweep
      ctx.strokeStyle = C.cyan; ctx.lineWidth = 2; ctx.shadowColor = C.cyan; ctx.shadowBlur = 16;
      ctx.beginPath(); ctx.moveTo(0, hz); ctx.lineTo(w, hz); ctx.stroke(); ctx.shadowBlur = 0;
      const sweep = ((t / 4200) % 1) * (w + 400) - 200;
      const sw = ctx.createLinearGradient(sweep - 140, 0, sweep + 140, 0);
      sw.addColorStop(0, rgba(C.red, 0)); sw.addColorStop(.5, rgba(C.red, .55)); sw.addColorStop(1, rgba(C.red, 0));
      ctx.fillStyle = sw; ctx.fillRect(sweep - 140, hz - 1, 280, 3);
    }
    function loop() {
      if (!on) return;
      frame(performance.now() - t0);
      if (!still) requestAnimationFrame(loop);
    }
    document.addEventListener('cc:theme', () => requestAnimationFrame(() => frame(performance.now() - t0)));
    loop();
  })();

  /* ---------- pointer trail ---------- */
  (function trail() {
    const cv = $('#trail'); if (!cv || !fine || still) { if (cv) cv.remove(); return; }
    const ctx = cv.getContext('2d'); const P = [];
    let raf = 0, k = 0;
    const size = () => { cv.width = innerWidth * DPR; cv.height = innerHeight * DPR; ctx.setTransform(DPR, 0, 0, DPR, 0, 0); };
    size(); addEventListener('resize', size);
    addEventListener('pointermove', e => {
      const cols = [C.pink, C.cyan, C.acid, C.purple];
      for (let i = 0; i < 2; i++) P.push({ x: e.clientX, y: e.clientY, vx: (Math.random() - .5) * 1.2, vy: (Math.random() - .5) * 1.2 - .3, life: 1, c: cols[(k++ >> 2) % cols.length] });
      if (P.length > 90) P.splice(0, P.length - 90);
      if (!raf) raf = requestAnimationFrame(tick);
    }, { passive: true });
    function tick() {
      ctx.clearRect(0, 0, innerWidth, innerHeight);
      ctx.globalCompositeOperation = 'lighter';
      for (let i = P.length - 1; i >= 0; i--) {
        const p = P[i]; p.x += p.vx; p.y += p.vy; p.life -= .035;
        if (p.life <= 0) { P.splice(i, 1); continue; }
        ctx.fillStyle = rgba(p.c, p.life * .8); ctx.shadowColor = p.c; ctx.shadowBlur = 8;
        const s = 2.6 * p.life; ctx.fillRect(p.x - s / 2, p.y - s / 2, s, s);
      }
      ctx.shadowBlur = 0; ctx.globalCompositeOperation = 'source-over';
      raf = P.length ? requestAnimationFrame(tick) : 0;
    }
  })();

  /* ---------- glyph rain (pipeline) ---------- */
  (function rain() {
    const cv = $('#rain'); if (!cv) return;
    const ctx = cv.getContext('2d');
    const G = 'アイウエオカキクケコサシスセソ0123456789ABCDEF#{}:;<>/=+*';
    let w, h, cols, drops, on = false, last = 0;
    const fs = 14;
    function size() { const r = cv.getBoundingClientRect(); w = r.width; h = r.height; cv.width = w * DPR; cv.height = h * DPR; ctx.setTransform(DPR, 0, 0, DPR, 0, 0); cols = Math.ceil(w / fs); drops = Array.from({ length: cols }, () => Math.random() * -h / fs); }
    size(); addEventListener('resize', size);
    whenVisible(cv, v => { on = v && !still; if (on) requestAnimationFrame(loop); });
    function loop(t) {
      if (!on) return;
      if (t - last > 55) {
        last = t;
        ctx.fillStyle = rgba(C.bg, .14); ctx.fillRect(0, 0, w, h);
        ctx.font = `${fs}px JetBrains Mono, monospace`;
        for (let i = 0; i < cols; i++) {
          const y = drops[i] * fs;
          ctx.fillStyle = Math.random() > .97 ? C.white : (i % 3 ? C.acid : C.cyan);
          ctx.fillText(G[(Math.random() * G.length) | 0], i * fs, y);
          if (y > h && Math.random() > .975) drops[i] = 0; else drops[i] += 1;
        }
      }
      requestAnimationFrame(loop);
    }
  })();

  /* ---------- text scramble ---------- */
  const GL = '!<>-_\\/[]{}—=+*^?#01ABCDEF';
  function scramble(el) {
    if (el.dataset.scrambled) return; el.dataset.scrambled = '1';
    const walk = n => n.nodeType === 3 ? [n] : [...n.childNodes].flatMap(walk);
    const nodes = walk(el).filter(n => n.textContent.trim());
    nodes.forEach(n => {
      const final = n.textContent; let f = 0; const total = 26;
      const q = [...final].map((ch, i) => ({ ch, start: Math.floor(Math.random() * 8) + i * .4, end: Math.floor(Math.random() * 10) + 12 + i * .4 }));
      const step = () => {
        n.textContent = q.map(o => (f >= o.end || o.ch === ' ' ? o.ch : f >= o.start ? GL[(Math.random() * GL.length) | 0] : o.ch === ' ' ? ' ' : '')).join('');
        if (++f <= total + final.length * .4) requestAnimationFrame(step); else n.textContent = final;
      };
      step();
    });
  }

  /* ---------- reveal on scroll (+ scramble, counters, flow) ---------- */
  const ro = new IntersectionObserver(es => es.forEach(e => {
    if (!e.isIntersecting) return;
    e.target.classList.add('in');
    if (!still) e.target.querySelectorAll('.scramble').forEach(scramble);
    ro.unobserve(e.target);
  }), { threshold: .14 });
  $$('.reveal, .flow').forEach(el => still ? el.classList.add('in') : ro.observe(el));

  function countUp(el) {
    const end = +el.dataset.count; const t0 = performance.now(); const d = 1400;
    const step = t => { const k = Math.max(0, Math.min(1, (t - t0) / d)); el.textContent = Math.round(end * (1 - Math.pow(1 - k, 3))); if (k < 1) requestAnimationFrame(step); };
    still ? (el.textContent = end) : requestAnimationFrame(step);
  }
  const co = new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) { countUp(e.target); co.unobserve(e.target); } }));
  $$('[data-count]').forEach(el => co.observe(el));

  /* ---------- rotator ---------- */
  (function rotator() {
    const el = $('#rotator'); if (!el || still) return;
    const words = ['SCHEMA', 'THEMES', 'TOKENS', 'PATHS', '254 THEMES', 'MASHUPS'];
    let i = 0;
    setInterval(() => {
      i = (i + 1) % words.length; const target = words[i]; let f = 0;
      const step = () => {
        el.textContent = [...target].map((c, k) => (k < f / 2 ? c : GL[(Math.random() * GL.length) | 0])).join('');
        if (++f <= target.length * 2) requestAnimationFrame(step); else el.textContent = target;
      };
      step();
    }, 2600);
  })();

  /* ---------- magnetic buttons ---------- */
  if (fine && !still) $$('.magnetic').forEach(b => {
    b.addEventListener('pointermove', e => { const r = b.getBoundingClientRect(); b.style.transform = `translate(${(e.clientX - r.left - r.width / 2) * .22}px, ${(e.clientY - r.top - r.height / 2) * .32}px)`; });
    b.addEventListener('pointerleave', () => { b.style.transform = ''; });
  });

  /* ---------- tilt + sheen + spotlight (delegated, so new cards just work) ---------- */
  if (fine && !still) {
    document.addEventListener('pointermove', e => {
      const t = e.target.closest('.tcard, .tile, .spot');
      if (!t) return;
      const r = t.getBoundingClientRect(); const x = (e.clientX - r.left) / r.width, y = (e.clientY - r.top) / r.height;
      t.style.setProperty('--mx', `${x * 100}%`); t.style.setProperty('--my', `${y * 100}%`);
      if (t.matches('.tcard, .tile')) { t.style.setProperty('--rx', `${(.5 - y) * 10}deg`); t.style.setProperty('--ry', `${(x - .5) * 12}deg`); }
    }, { passive: true });
    document.addEventListener('pointerout', e => {
      const t = e.target.closest('.tcard, .tile'); if (t && !t.contains(e.relatedTarget)) { t.style.setProperty('--rx', '0deg'); t.style.setProperty('--ry', '0deg'); }
    });
    // HUD parallax
    const hud = $('#hud');
    if (hud) addEventListener('pointermove', e => { hud.style.setProperty('--hx', `${(e.clientX / innerWidth - .5) * 14}deg`); hud.style.setProperty('--hy', `${(.5 - e.clientY / innerHeight) * 10}deg`); }, { passive: true });
  }
  // HUD signal jitter
  const sig = $('#hudSig');
  if (sig && !still) setInterval(() => { sig.textContent = `${(96 + Math.random() * 4).toFixed(1)}%`; }, 900);

  /* ---------- particle burst ---------- */
  function burst(x, y, n = 26) {
    if (still) return;
    const cols = [C.pink, C.cyan, C.acid, C.orange, C.purple];
    for (let i = 0; i < n; i++) {
      const s = document.createElement('i'); s.className = 'spark';
      s.style.setProperty('--c', cols[i % cols.length]); s.style.left = `${x}px`; s.style.top = `${y}px`;
      document.body.append(s);
      const a = Math.random() * Math.PI * 2, d = 50 + Math.random() * 110;
      s.animate([{ transform: 'translate(-50%,-50%) scale(1)', opacity: 1 }, { transform: `translate(calc(-50% + ${Math.cos(a) * d}px), calc(-50% + ${Math.sin(a) * d}px)) scale(0) rotate(${Math.random() * 360}deg)`, opacity: 0 }],
        { duration: 650 + Math.random() * 400, easing: 'cubic-bezier(.2,.8,.2,1)' }).onfinish = () => s.remove();
    }
  }
  document.addEventListener('click', e => {
    const b = e.target.closest('.burst, .btn.primary, #shuffle, .tcard'); if (b) burst(e.clientX || innerWidth / 2, e.clientY || innerHeight / 2, b.matches('.tcard') ? 14 : 26);
  });

  /* ---------- scroll progress + active nav ---------- */
  const prog = $('#progress');
  const navLinks = $$('.nav a');
  const sections = navLinks.map(a => $(a.getAttribute('href')));
  addEventListener('scroll', () => {
    const max = document.documentElement.scrollHeight - innerHeight;
    if (prog) prog.style.setProperty('--p', max > 0 ? scrollY / max : 0);
    let idx = -1; sections.forEach((s, i) => { if (s && s.getBoundingClientRect().top < innerHeight * .4) idx = i; });
    navLinks.forEach((a, i) => a.classList.toggle('on', i === idx));
  }, { passive: true });

  /* ---------- tabs ---------- */
  $$('.tabs [role="tab"]').forEach(tab => tab.addEventListener('click', () => {
    const list = tab.parentElement;
    list.querySelectorAll('[role="tab"]').forEach(t => { const on = t === tab; t.setAttribute('aria-selected', String(on)); $('#' + t.getAttribute('aria-controls')).hidden = !on; });
  }));

  /* ---------- copy buttons ---------- */
  $$('pre[data-copy]').forEach(pre => {
    const b = document.createElement('button'); b.className = 'copy'; b.type = 'button'; b.innerHTML = '<svg><use href="#i-copy"/></svg>COPY';
    b.addEventListener('click', async () => {
      try { await navigator.clipboard.writeText(pre.querySelector('code').innerText); window.CC?.toast('COPIED TO CLIPBOARD'); } catch { window.CC?.toast('SELECT + COPY'); }
    });
    pre.append(b);
  });
  $$('[data-copy-command]').forEach(button => button.addEventListener('click', async () => {
    const command = button.dataset.copyCommand;
    try {
      await navigator.clipboard.writeText(command);
    } catch {
      const input = document.createElement('textarea');
      input.value = command;
      input.style.position = 'fixed';
      input.style.opacity = '0';
      document.body.append(input);
      input.select();
      document.execCommand('copy');
      input.remove();
    }
    button.textContent = 'COPIED';
    window.CC?.toast('COPIED TO CLIPBOARD');
    window.setTimeout(() => { button.textContent = 'COPY'; }, 1400);
  }));

  /* ---------- terminal type-on ---------- */
  $$('[data-type] code').forEach(code => {
    if (still) return;
    const html = code.innerHTML.split('\n'); code.innerHTML = '';
    const io = new IntersectionObserver(es => {
      if (!es.some(e => e.isIntersecting)) return; io.disconnect();
      html.forEach((line, i) => setTimeout(() => { code.innerHTML += (i ? '\n' : '') + line; document.dispatchEvent(new CustomEvent('cc:typed')); }, 180 * i));
    }, { threshold: .4 });
    io.observe(code);
  });
  // the typed lines contain live slot spans; repaint them once they exist
  document.addEventListener('cc:typed', () => { if (window.CC && CC.id) { const p = CC.palette; $$('[data-slot-hex]').forEach(el => { el.textContent = `"${p[el.dataset.slotHex]}"`; }); $$('[data-slot-css]').forEach(el => { el.textContent = `"#${p[el.dataset.slotCss]}"`; }); $$('[data-active-theme-raw]').forEach(el => { el.textContent = CC.id; }); } });

  /* ---------- motion lanes: travel the real lane width ---------- */
  const setLanes = () => $$('.lane').forEach(l => l.style.setProperty('--lane-w', `${l.clientWidth}px`));
  setLanes(); addEventListener('resize', setLanes);
  $$('.lane').forEach(l => l.addEventListener('click', () => l.classList.toggle('run')));

  /* ---------- konami → overdrive ---------- */
  const K = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];
  let ki = 0;
  addEventListener('keydown', e => {
    ki = e.key.toLowerCase() === K[ki].toLowerCase() ? ki + 1 : (e.key === K[0] ? 1 : 0);
    if (ki === K.length) {
      ki = 0; const on = root.classList.toggle('overdrive');
      window.CC?.toast(on ? 'OVERDRIVE ENGAGED' : 'OVERDRIVE OFF');
      burst(innerWidth / 2, innerHeight / 2, 60);
    }
  });
})();
