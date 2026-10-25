# Public documentation policy

`docs/public-book` is the only documentation source permitted for the public
Cybercore Pages site.

It must never be generated from, copied from, or synchronized with the private
Wiki. Private Wiki content, machine-specific paths, credentials, personal
data, internal notes, and local operational records are prohibited.

Before publishing:

```sh
python tools/generate-public-docs.py
python tools/verify-public-docs.py
mdbook build docs/public-book
python tools/verify-public-docs.py docs/public-book/book
```

The generated `docs/public-book/book` directory is intentionally local-only
and is ignored by Git. A Pages deployment must be built from the sanitized
public-book source and reviewed before it is pushed.
