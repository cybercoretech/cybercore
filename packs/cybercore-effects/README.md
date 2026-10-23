# 📦 Cybercore Effects Pack Family

Each effect is a governed resource with an implementation, static fallback,
reduced-motion behavior, and a manifest entry. Prefer `transform` and `opacity`
for animation; high-risk Canvas/WebGL recipes must document their fallback and
mobile budget before publication.

## 📦 Effect resource layout

```text
effect-name/
  effect.css
  effect.svg or effect.js
  fallback.css
  reduced-motion.css
  manifest.json
  README.md
```
