// Sets the app version in every file that holds it, or checks that they agree.
// `src-tauri/tauri.conf.json` is the reference: it is what the app reports
// (`getVersion()`) and what the updater compares against `latest.json`. The
// other files follow it so that none contradicts it. See docs/releasing.md.
//
// Usage:
//   node scripts/bump-version.mjs <version>            set the version, e.g. 0.4.0
//   node scripts/bump-version.mjs --check [<version>]  fail unless every file holds
//                                                       the same version (and that one)
//
// A version may start with `v`, as a release tag does.

import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Semantic Versioning 2.0.0, which Tauri and Cargo both require. */
const SEMVER =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

const REFERENCE = "src-tauri/tauri.conf.json";

/**
 * Where the version lives: the first match of `pattern` in each file, the
 * version being its second group. Only that group is replaced, so formatting
 * is preserved.
 */
const VERSION_FILES = [
  // Top-level field of the JSON manifests (two-space indent).
  { path: "package.json", pattern: /^( {2}"version": ")([^"]*)"/m },
  { path: REFERENCE, pattern: /^( {2}"version": ")([^"]*)"/m },
  // The `[package]` table, before the next table.
  {
    path: "src-tauri/Cargo.toml",
    pattern: /^(\[package\]\r?\n(?:(?!\[).*\r?\n)*?version = ")([^"]*)"/m,
  },
  // The lock file records the app crate's version too.
  {
    path: "src-tauri/Cargo.lock",
    pattern: /^(name = "claude-sessions"\r?\nversion = ")([^"]*)"/m,
  },
];

/** `version` without the tag's `v`; throws unless it is a semantic version. */
function parseVersion(version) {
  const plain = version.startsWith("v") ? version.slice(1) : version;
  if (!SEMVER.test(plain)) {
    throw new Error(`"${version}" is not a semantic version, such as 1.2.0`);
  }
  return plain;
}

function findVersion(root, { path, pattern }) {
  const content = readFileSync(join(root, path), "utf8");
  const match = pattern.exec(content);
  if (!match) throw new Error(`${path} holds no version where expected`);
  return { content, start: match.index + match[1].length, version: match[2] };
}

function readVersions(root) {
  return Object.fromEntries(
    VERSION_FILES.map((file) => [file.path, findVersion(root, file).version]),
  );
}

function setVersion(root, version) {
  const plain = parseVersion(version);
  for (const file of VERSION_FILES) {
    const { content, start, version: current } = findVersion(root, file);
    writeFileSync(
      join(root, file.path),
      content.slice(0, start) + plain + content.slice(start + current.length),
    );
  }
}

/** Every disagreement with the reference file, and with `expected` when given. */
function versionProblems(root, expected) {
  const versions = readVersions(root);
  const reference = versions[REFERENCE];
  const problems = Object.entries(versions)
    .filter(([, version]) => version !== reference)
    .map(([path, version]) => `${path} holds ${version}, ${REFERENCE} ${reference}`);
  if (expected !== undefined && parseVersion(expected) !== reference) {
    problems.push(`the version is ${reference}, not ${parseVersion(expected)}`);
  }
  return problems;
}

const root = resolve(fileURLToPath(import.meta.url), "../..");
const args = process.argv.slice(2);

if (args[0] === "--check" && args.length <= 2) {
  const problems = versionProblems(root, args[1]);
  for (const problem of problems) console.error(problem);
  if (problems.length > 0) {
    console.error("Run `node scripts/bump-version.mjs <version>` and commit the result.");
    process.exit(1);
  }
  console.log(`Every file holds version ${readVersions(root)[REFERENCE]}.`);
} else if (args.length === 1 && !args[0].startsWith("-")) {
  setVersion(root, args[0]);
  console.log(`Version set to ${parseVersion(args[0])}.`);
} else {
  console.error("Usage: node scripts/bump-version.mjs <version> | --check [<version>]");
  process.exit(2);
}
