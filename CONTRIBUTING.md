# 🤝 Contributing to Cybercore

Thanks for helping improve the Cybercore Systems Framework.

## 💬 Choose the right channel

- Use [GitHub Discussions](https://github.com/cybercoretech/cybercore/discussions)
  for questions, design ideas, help, and project showcases.
- Use [issue forms](https://github.com/cybercoretech/cybercore/issues/new/choose)
  for reproducible bugs and focused feature requests.
- Use pull requests for reviewed code, documentation, theme, and test changes.

Never post credentials, personal data, private repository material, or
machine-specific operational records in a public issue, discussion, or pull
request.

## 🛠️ Development workflow

1. Fork the repository and create a focused branch from `main`.
2. Make a small, documented change with tests for behavioral changes.
3. Run the relevant local checks:

   ```sh
   cargo fmt --all -- --check
   cargo check --workspace --all-targets --all-features
   cargo test --workspace --all-targets --all-features
   ```

4. Describe the user-visible outcome and validation in the pull request.

For public documentation changes, build and review only material authored in
this public repository. The private Wiki is never a source for GitHub Pages
content.

## 🎨 Themes and design-system changes

Keep shared tokens, schemas, and consumer contracts aligned. New themes and
packs must pass the repository's existing contrast, schema, and compatibility
checks. Include a concise visual or behavioral description when the change
affects a rendered surface.
