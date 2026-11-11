/* Cybercore brand theme variables: maps the chooser's palette slots to
 * --cc-brand-<slot> custom properties that the themed brand SVGs read. */

export const BRAND_SLOTS = ['bg', 'white', 'panel', 'line', 'muted', 'acid_green', 'hot_pink', 'purple', 'cyan', 'orange', 'red'];

export function brandVarName(slot) {
  return `--cc-brand-${String(slot).replace(/_/g, '-')}`;
}

export function brandVarEntries(palette) {
  if (!palette) return [];
  return BRAND_SLOTS.filter(slot => palette[slot]).map(slot => [brandVarName(slot), `#${palette[slot]}`]);
}

// Sets the variables only when the chooser is recolouring; records their names
// in `painted` so the caller's next repaint removes them again.
export function paintBrandVars(style, palette, recolor, painted) {
  if (!recolor) return;
  for (const [name, value] of brandVarEntries(palette)) {
    style.setProperty(name, value);
    painted.push(name);
  }
}
