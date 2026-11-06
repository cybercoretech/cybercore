(() => {
  const crateList = document.querySelector('[data-crate-list]');
  const ownRepository = /^https:\/\/github\.com\/cybercoretech\//;
  // Only the framework's own crates belong under "Install the contracts";
  // other cybercoretech crates (cyberterm, gateflow, ...) live on their own sites.
  const frameworkCrate = /^cybercore(-|$)/;
  const status = document.getElementById('copyStatus');

  function setCount(selector, value) {
    document.querySelectorAll(selector).forEach(el => { el.textContent = String(value); });
  }

  setCount('[data-pack-count]', document.querySelectorAll('.pack-row:not(.pack-head)').length);
  if (crateList) setCount('[data-crate-count]', crateList.querySelectorAll('.crate').length);

  // Card = plain container; the name is the link (stretched over the card in
  // CSS) and the copy button sits beside it, so no control nests in another.
  function renderCrates(crates) {
    if (!crateList || !crates.length) return;
    crateList.replaceChildren(...crates.map(crate => {
      const card = document.createElement('div');
      card.className = 'crate';

      const link = document.createElement('a');
      link.href = `https://crates.io/crates/${encodeURIComponent(crate.name)}`;
      const name = document.createElement('b');
      name.textContent = crate.name;
      link.append(name);

      const description = document.createElement('span');
      description.textContent = crate.description || 'Published Cybercore Rust contract.';

      const command = document.createElement('button');
      command.type = 'button';
      command.dataset.copy = `cargo add ${crate.name}`;
      command.setAttribute('aria-label', `Copy: cargo add ${crate.name}`);
      command.textContent = `cargo add ${crate.name}`;

      card.append(link, description, command);
      return card;
    }));
    setCount('[data-crate-count]', crates.length);
  }

  if (crateList) {
    fetch('https://crates.io/api/v1/crates?q=cybercore&per_page=100', {
      headers: { Accept: 'application/json' }
    })
      .then(response => {
        if (!response.ok) throw new Error(`crates.io returned ${response.status}`);
        return response.json();
      })
      .then(payload => {
        const crates = (payload.crates || [])
          .filter(crate => ownRepository.test(crate.repository || '') && frameworkCrate.test(crate.name))
          .sort((a, b) => a.name.localeCompare(b.name));
        renderCrates(crates);
      })
      .catch(error => {
        console.warn('Cybercore crate inventory sync unavailable; using embedded inventory.', error);
      });
  }

  document.addEventListener('click', async event => {
    const button = event.target.closest('[data-copy]');
    if (!button) return;
    event.preventDefault();
    const command = button.dataset.copy;
    const label = button.textContent;
    let copied = true;
    try {
      await navigator.clipboard.writeText(command);
    } catch {
      const input = document.createElement('textarea');
      input.value = command;
      input.style.position = 'fixed';
      input.style.opacity = '0';
      document.body.append(input);
      input.select();
      copied = document.execCommand('copy');
      input.remove();
    }
    button.textContent = copied ? 'Copied' : 'Select + copy';
    // Announce through the page's polite live region (rendered empty at load).
    if (status) {
      status.textContent = '';
      requestAnimationFrame(() => { status.textContent = copied ? `Copied: ${command}` : `Couldn't copy. Command: ${command}`; });
    }
    window.setTimeout(() => { button.textContent = label; }, 1400);
  });

  // Narrow screens hide the inline nav; this button reveals it as a panel.
  const toggle = document.getElementById('navToggle');
  const nav = document.getElementById('primaryNav');
  if (toggle && nav) {
    const setOpen = open => {
      toggle.setAttribute('aria-expanded', String(open));
      nav.classList.toggle('open', open);
    };
    toggle.addEventListener('click', () => setOpen(toggle.getAttribute('aria-expanded') !== 'true'));
    nav.addEventListener('click', event => { if (event.target.closest('a')) setOpen(false); });
    document.addEventListener('keydown', event => {
      if (event.key === 'Escape' && nav.classList.contains('open')) { setOpen(false); toggle.focus(); }
    });
    document.addEventListener('pointerdown', event => {
      if (nav.classList.contains('open') && !nav.contains(event.target) && !toggle.contains(event.target)) setOpen(false);
    });
  }
})();
