# 🌐 GitHub Wiki gateway

The public GitHub Wiki is a generated onboarding gateway. It links visitors to
the canonical public documentation, live gallery, docs.rs, community channels,
and the built-in theme catalog. It is not a second copy of operational or
private documentation.

The catalog source is `schema/themes/`. Generate it into a checkout of the
repository Wiki:

```sh
git clone https://github.com/cybercoretech/cybercore.wiki.git /tmp/cybercore.wiki
python tools/generate-github-wiki.py --output /tmp/cybercore.wiki
python tools/generate-github-wiki.py --output /tmp/cybercore.wiki --check
```

Review the generated Markdown, then commit and push from the Wiki checkout.
The generator removes only the legacy scaffold pages it replaces and preserves
other Wiki files.

## ✅ Validation

Before publishing, verify that the generator can reproduce the checkout:

```sh
python tools/generate-github-wiki.py --output /tmp/cybercore.wiki --check
```

Run the generator whenever canonical built-in themes change. Do not add
machine-specific paths, credentials, private Wiki content, personal data, or
internal operational records to the public Wiki.
