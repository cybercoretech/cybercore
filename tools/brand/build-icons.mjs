import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const brand = path.join(root, "packs", "cybercore-brand");
const icons = path.join(brand, "icons");
const master = await readFile(path.join(brand, "logos", "symbol.svg"), "utf8");
await mkdir(icons, { recursive: true });
for (const size of [16, 32, 48, 64, 96, 128, 192, 256, 384, 512]) {
  const svg = master.replace('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"', `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 512 512"`);
  await writeFile(path.join(icons, `symbol-${size}.svg`), svg);
}
console.log("Built ten SVG Cybercore icon sizes. Raster PNG exports are generated separately with ImageMagick.");
