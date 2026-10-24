(() => {
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
