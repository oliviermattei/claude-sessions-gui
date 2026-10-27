// Writes `latest.json`, the manifest the updater of every installed app reads
// from the latest GitHub Release to learn of a newer version (endpoint in
// src-tauri/tauri.conf.json, see docs/releasing.md).
//
// Each updater bundle of the release folder that has a `.sig` next to it (made
// by `tauri build` with the updater key) becomes the entry of its platform. The
// bundles carry the names build.yml gives them: `<Product>-<version>-<label>.<ext>`.
//
// Usage: node scripts/updater-manifest.mjs <release folder> <version> <download URL>
//   where the download URL is that of the release files, e.g.
//   https://github.com/<owner>/<repo>/releases/download/v1.2.0

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Updater bundles by the end of their file name, with the platform keys the
 * updater looks up: `<os>-<arch>-<installer>` first, then `<os>-<arch>`.
 */
const BUNDLES = [
  { suffix: "-win-Setup.exe", platform: "windows-x86_64", installer: "nsis" },
  { suffix: "-macOS-arm64.app.tar.gz", platform: "darwin-aarch64", installer: "app" },
  { suffix: "-macOS-x64.app.tar.gz", platform: "darwin-x86_64", installer: "app" },
  { suffix: "-linux-x86_64.AppImage", platform: "linux-x86_64", installer: "appimage" },
  { suffix: "-linux-arm64.AppImage", platform: "linux-aarch64", installer: "appimage" },
];

const [folder, rawVersion, rawUrl, ...rest] = process.argv.slice(2);
if (!folder || !rawVersion || !rawUrl || rest.length > 0) {
  console.error(
    "Usage: node scripts/updater-manifest.mjs <release folder> <version> <download URL>",
  );
  process.exit(2);
}
const version = rawVersion.replace(/^v/, "");
const downloadUrl = rawUrl.replace(/\/+$/, "");

const names = new Set(readdirSync(folder));
const platforms = {};
for (const name of [...names].sort()) {
  const bundle = BUNDLES.find(({ suffix }) => name.endsWith(suffix));
  if (!bundle || !names.has(`${name}.sig`)) continue;
  const entry = {
    signature: readFileSync(join(folder, `${name}.sig`), "utf8").trim(),
    url: `${downloadUrl}/${encodeURIComponent(name)}`,
  };
  for (const key of [`${bundle.platform}-${bundle.installer}`, bundle.platform]) {
    if (key in platforms) throw new Error(`Two bundles are for ${key}`);
    platforms[key] = entry;
  }
}

// Every platform the release is built for must be updatable: a missing one would
// strand its users on the previous version without any error.
const missing = BUNDLES.filter(({ platform }) => !(platform in platforms));
if (missing.length > 0) {
  console.error(
    `No signed updater bundle for ${missing.map((b) => b.platform).join(", ")}.`,
  );
  process.exit(1);
}

const manifest = { version, pub_date: new Date().toISOString(), platforms };
writeFileSync(join(folder, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`latest.json ${version} lists ${Object.keys(platforms).join(", ")}.`);
