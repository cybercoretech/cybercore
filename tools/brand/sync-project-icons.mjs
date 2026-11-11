#!/usr/bin/env node
import { readFile, readdir, stat, mkdir, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const brandRoot = path.join(repoRoot, 'packs', 'cybercore-brand');
const cybercoreRoot = process.env.CYBERCORE_ROOT ? path.resolve(process.env.CYBERCORE_ROOT) : path.resolve(repoRoot, '../..');
const catalogPath = path.join(brandRoot, 'projects.json');
const catalog = JSON.parse(await readFile(catalogPath, 'utf8'));
const allProjects = catalog.projects.map(project => [project.title, null, null, project.repo]);
const onlyRepos = process.env.ONLY_REPOS?.split(',').map(value => value.toLowerCase());
const projects = onlyRepos?.length ? allProjects.filter(project => onlyRepos.includes(project[3].toLowerCase())) : allProjects;
const outDir = path.join(brandRoot, 'projects');
const slug = value => value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'project';

async function walk(dir, depth = 0, results = []) {
  if (depth > 7) return results;
  let entries;
  try { entries = await readdir(dir, { withFileTypes: true }); } catch { return results; }
  for (const entry of entries) {
    if (entry.name === '.git' || entry.name === 'node_modules' || entry.name === 'target' || entry.name === 'book') continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) await walk(full, depth + 1, results);
    else if (/\.(svg|png|webp)$/i.test(entry.name) && /(favicon|icon|mark|logo)/i.test(entry.name)) {
      const info = await stat(full).catch(() => null);
      if (info && info.size < 500_000) results.push(full);
    }
  }
  return results;
}

const localFiles = await walk(cybercoreRoot);
const directories = [];
async function walkDirectories(dir, depth = 0) {
  if (depth > 5) return;
  let entries;
  try { entries = await readdir(dir, { withFileTypes: true }); } catch { return; }
  for (const entry of entries) {
    if (!entry.isDirectory() || ['.git', 'node_modules', 'target', 'book'].includes(entry.name)) continue;
    const full = path.join(dir, entry.name);
    directories.push(full);
    await walkDirectories(full, depth + 1);
  }
}
await walkDirectories(cybercoreRoot);

function localIcon(repo) {
  if (repo.toLowerCase() === 'cybercoretech.github.io') return path.join(brandRoot, 'icons/symbol-128.svg');
  const roots = directories.filter(dir => path.basename(dir).toLowerCase() === repo.toLowerCase());
  const candidates = localFiles.filter(file => roots.some(root => file.startsWith(`${root}${path.sep}`)));
  const rank = file => {
    const base = path.basename(file).toLowerCase();
    const repoName = repo.toLowerCase().replace(/[^a-z0-9]/g, '');
    if (/social|hero|header|horizontal|divider|concept|screenshot/.test(base)) return 99;
    if (base === 'favicon.svg') return 0;
    if (base === `${repoName}-favicon.svg`) return 1;
    if (base === `${repoName}-icon.svg`) return 2;
    if (base === 'icon.svg') return 3;
    if (base === `${repoName}-mark.svg`) return 4;
    if (base === 'mark.svg') return 5;
    if (base === `${repoName}-icon.png` || base === 'icon.png') return 6;
    if (/favicon\.(svg|png|webp)$/.test(base)) return 7;
    if (/icon\.(svg|png|webp)$/.test(base)) return 8;
    if (/mark\.(svg|png|webp)$/.test(base)) return 9;
    return 20;
  };
  return candidates.map(file => ({ file, score:rank(file) })).filter(item => item.score < 99)
    .sort((a, b) => a.score - b.score || a.file.length - b.file.length)[0]?.file;
}

function attrs(tag) {
  const out = {};
  for (const match of tag.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g)) out[match[1].toLowerCase()] = match[2] ?? match[3] ?? match[4] ?? '';
  return out;
}

function asSvg(bytes, mime) {
  if (/svg/i.test(mime) || bytes.toString('utf8', 0, 300).includes('<svg')) {
    let svg = bytes.toString('utf8').replace(/^\uFEFF/, '');
    if (!/<svg\b/i.test(svg)) return null;
    svg = svg.replace(/<script\b[\s\S]*?<\/script\s*>/gi, '').replace(/<foreignObject\b[\s\S]*?<\/foreignObject\s*>/gi, '');
    return svg;
  }
  if (/png/i.test(mime) || bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) {
    const data = bytes.toString('base64');
    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128"><image width="128" height="128" preserveAspectRatio="xMidYMid meet" href="data:image/png;base64,${data}"/></svg>`;
  }
  return null;
}

function initials(name) {
  const parts = String(name).replace(/([a-z])([A-Z])/g, '$1 $2').match(/[\p{L}\p{N}]+/gu) || [];
  return (parts.length > 1 ? `${parts[0][0]}${parts[1][0]}` : (parts[0] || 'SYS').slice(0, 2)).toUpperCase();
}
function monogram(name, repo) {
  const tones = ['#91bec8','#c194b3','#adc487','#c8ab78','#a69acd','#ca8d98'];
  let seed = 2166136261;
  for (const char of repo.toLowerCase()) { seed ^= char.charCodeAt(0); seed = Math.imul(seed, 16777619); }
  const tone = tones[(seed >>> 0) % tones.length];
  const label = String(name).replace(/[<>&"']/g, '');
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128" role="img" aria-label="${label} wordmark"><rect x="1" y="1" width="126" height="126" rx="18" fill="#0d1015" stroke="#343943" stroke-width="2"/><path d="M18 32V18h14m64 0h14v14m0 50v14h-14m-64 0H18V82" fill="none" stroke="${tone}" stroke-opacity=".55" stroke-width="2"/><circle cx="64" cy="64" r="40" fill="${tone}" fill-opacity=".045"/><text x="64" y="76" text-anchor="middle" fill="${tone}" font-family="Oxanium,sans-serif" font-size="37" font-weight="700" letter-spacing="1">${initials(name)}</text></svg>`;
}

async function remoteIcon(repo) {
  const page = repo.toLowerCase() === 'cybercoretech.github.io' || repo === '.github'
    ? 'https://cybercoretech.github.io/'
    : `https://cybercoretech.github.io/${encodeURIComponent(repo)}/`;
  let response;
  try { response = await fetch(page, { signal:AbortSignal.timeout(18000), headers:{ 'User-Agent':'CybercoreProjectBrandSync/1.0' } }); }
  catch { return null; }
  if (!response.ok) return null;
  const html = await response.text();
  const candidates = [...html.matchAll(/<link\b[^>]*>/gi)].map(match => attrs(match[0]))
    .filter(link => /(?:^|\s)(?:shortcut\s+)?icon(?:\s|$)|apple-touch-icon|mask-icon/i.test(link.rel || ''))
    .sort((a, b) => (/svg/i.test(b.type || '') ? 1 : 0) - (/svg/i.test(a.type || '') ? 1 : 0));
  for (const link of candidates) {
    if (!link.href) continue;
    if (/^data:image\/(?:svg\+xml|png);/i.test(link.href)) {
      const [header, body] = link.href.split(',', 2);
      try {
        const raw = /;base64/i.test(header) ? Buffer.from(body, 'base64') : Buffer.from(decodeURIComponent(body));
        const svg = asSvg(raw, header.includes('png') ? 'image/png' : 'image/svg+xml');
        if (svg) return { svg, source:`${page} (inline ${header.includes('png') ? 'PNG' : 'SVG'} favicon)` };
      } catch { /* Try another icon link. */ }
      continue;
    }
    try {
      const url = new URL(link.href, page);
      if (url.protocol !== 'https:') continue;
      const icon = await fetch(url, { signal:AbortSignal.timeout(12000), headers:{ 'User-Agent':'CybercoreProjectBrandSync/1.0' } });
      if (!icon.ok) continue;
      const svg = asSvg(Buffer.from(await icon.arrayBuffer()), icon.headers.get('content-type') || '');
      if (svg) return { svg, source:url.href };
    } catch { /* Try the next link. */ }
  }
  return null;
}

async function repositoryIcon(repo) {
  let response;
  try {
    response = await fetch(`https://api.github.com/repos/cybercoretech/${encodeURIComponent(repo)}/git/trees/HEAD?recursive=1`, {
      signal:AbortSignal.timeout(18000),
      headers:{ Accept:'application/vnd.github+json', 'X-GitHub-Api-Version':'2022-11-28', 'User-Agent':'CybercoreProjectBrandSync/1.0' },
    });
  } catch { return null; }
  if (!response.ok) return null;
  const tree = await response.json();
  if (process.env.DEBUG_REPO_ART === '1') {
    const artFiles = (tree.tree || []).filter(item => item.type === 'blob' && /\.(svg|png|webp|jpg|jpeg|ico)$/i.test(item.path));
    console.log(`${repo} image files: ${artFiles.map(item => item.path).slice(0, 50).join(' | ') || '(none)'}`);
  }
  const candidates = (tree.tree || []).filter(item => item.type === 'blob' && /\.(svg|png|webp)$/i.test(item.path)
    && /(favicon|icon|mark|logo)/i.test(path.basename(item.path))
    && !/(node_modules|target|social|hero|header|horizontal|divider|concept|screenshot)/i.test(item.path));
  const rank = item => {
    const base = path.basename(item.path).toLowerCase();
    const repoName = repo.toLowerCase().replace(/[^a-z0-9]/g, '');
    if (base === 'favicon.svg') return 0;
    if (base === `${repoName}-favicon.svg`) return 1;
    if (base === `${repoName}-icon.svg`) return 2;
    if (base === 'icon.svg') return 3;
    if (base === `${repoName}-mark.svg`) return 4;
    if (base === 'mark.svg') return 5;
    if (base.endsWith('-icon.svg')) return 6;
    if (base.endsWith('-mark.svg')) return 7;
    return 10 + (item.path.startsWith('assets/') ? 0 : item.path.startsWith('public/') || item.path.startsWith('static/') ? 1 : 2);
  };
  candidates.sort((a, b) => rank(a) - rank(b) || a.path.length - b.path.length);
  for (const item of candidates) {
    try {
      const raw = await fetch(`https://raw.githubusercontent.com/cybercoretech/${encodeURIComponent(repo)}/${tree.sha}/${item.path}`, { signal:AbortSignal.timeout(12000), headers:{ 'User-Agent':'CybercoreProjectBrandSync/1.0' } });
      if (!raw.ok) continue;
      const svg = asSvg(Buffer.from(await raw.arrayBuffer()), raw.headers.get('content-type') || path.extname(item.path));
      if (svg) return { svg, source:`github:${repo}/${item.path}` };
    } catch { /* Try another icon-like asset in the repo. */ }
  }
  return null;
}

await mkdir(outDir, { recursive:true });
const manifest = {};
const failures = [];
const queue = [...projects];
await Promise.all(Array.from({ length:6 }, async () => {
  while (queue.length) {
    const [name, , , repo] = queue.shift();
    let result = await remoteIcon(repo);
    if (!result) {
      const local = localIcon(repo);
      if (local) {
        try {
          const bytes = await readFile(local);
          result = { svg:asSvg(bytes, path.extname(local) === '.png' ? 'image/png' : 'image/svg+xml'), source:path.relative(cybercoreRoot, local) };
        } catch { /* Report if no valid local brand mark exists. */ }
      }
    }
    if (!result) result = await repositoryIcon(repo);
    if (!result?.svg) {
      const fallback = monogram(name, repo);
      result = { svg:fallback, source:'Cybercore fallback wordmark; no upstream favicon or compact logo exists' };
      failures.push(repo);
      console.warn(`${repo}: no upstream icon exists; using a subdued name-based wordmark fallback.`);
    }
    const file = `${slug(repo)}.svg`;
    await writeFile(path.join(outDir, file), result.svg);
    manifest[repo] = { file, source:result.source };
    console.log(`${repo}: ${result.source}`);
  }
}));

const byRepo = new Map(catalog.projects.map(project => [project.repo, project]));
for (const [repo, item] of Object.entries(manifest)) {
  const project = byRepo.get(repo);
  project.icon = `projects/${item.file}`;
  const match = item.source.match(/^https:\/\/\S+/);
  project.upstream = match ? match[0] : null;
}
if (!onlyRepos) {
  const wanted = new Set(catalog.projects.map(project => path.basename(project.icon)));
  for (const file of await readdir(outDir)) {
    if (file.endsWith('.svg') && !wanted.has(file)) await rm(path.join(outDir, file));
  }
}
catalog.projects.sort((a, b) => (a.repo < b.repo ? -1 : a.repo > b.repo ? 1 : 0));
await writeFile(catalogPath, `${JSON.stringify(catalog, null, 2)}\n`);
const fallbackCount = Object.values(manifest).filter(item => item.source.startsWith('Cybercore fallback wordmark')).length;
console.log(`\nStored ${Object.keys(manifest).length}/${projects.length} project marks (${projects.length - fallbackCount} upstream/local brand assets, ${fallbackCount} name-wordmark fallbacks).`);
if (failures.length) console.warn(`\n${failures.length} projects have no upstream favicon/logo; a subdued wordmark fallback was generated.`);
