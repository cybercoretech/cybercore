(() => {
  const crateList = document.querySelector('[data-crate-list]');
  const ownRepository = /^https:\/\/github\.com\/cybercoretech\//;

  function renderCrates(crates) {
    if (!crateList || !crates.length) return;
    crateList.replaceChildren(...crates.map(crate => {
      const link = document.createElement('a');
      link.href = `https://crates.io/crates/${encodeURIComponent(crate.name)}`;

      const name = document.createElement('b');
      name.textContent = crate.name;
      const description = document.createElement('span');
      description.textContent = crate.description || 'Published Cybercore Rust contract.';
      const command = document.createElement('code');
      command.textContent = `cargo add ${crate.name}`;

      link.append(name, description, command);
      return link;
    }));
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
          .filter(crate => ownRepository.test(crate.repository || ''))
          .sort((a, b) => a.name.localeCompare(b.name));
        renderCrates(crates);
      })
      .catch(error => {
        console.warn('Cybercore crate inventory sync unavailable; using embedded inventory.', error);
      });
  }

  document.querySelectorAll('[data-copy]').forEach(button => {
    button.addEventListener('click', async event => {
      event.preventDefault();
      const command = button.dataset.copy;
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
      window.setTimeout(() => { button.textContent = 'COPY'; }, 1400);
    });
  });
})();
