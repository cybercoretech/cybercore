// Map Cybercore palette slots to a site's own CSS variables.
// Load this before kit/chooser.js. Slots not listed use the Cybercore names
// (--bg, --white, --panel, --line, --muted, --acid, --pink, --purple, --cyan,
// --orange, --red). null leaves a slot unpainted.
window.SITE = Object.assign(window.SITE || {}, {
  siteLabel: "My Tool",            // name of the page's own look ("Site default")
  defaultTheme: "cybercore-tech",  // optional: start on a shared theme instead
  themeVars: {
    bg: "--surface-0",
    white: ["--text", "--fg"],
    acid_green: "--accent",
    purple: "--accent-2",
    line: null
  }
});
