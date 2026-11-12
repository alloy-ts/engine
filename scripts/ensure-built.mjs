// Ensures dist/ is built from the current build inputs (Rust crate + build
// script), not from a particular commit. Instead of keying the build marker on
// `git rev-parse HEAD` (which changes on every commit and forces a rebuild per
// commit, and also misses uncommitted source edits), we hash the actual build
// inputs and rebuild only when that hash changes.
//
// Build inputs that affect dist/index.js / dist/index.mjs:
//   - Cargo.toml, Cargo.lock   (crate deps + toolchain)
//   - build.ts                 (the build driver that runs before vp pack)
//   - src/**/*.rs              (Rust sources; the native crate)
// JS/TS files under src/ (e.g. lib.test.ts) are not compiled into the native
// addon, so they are deliberately excluded to avoid spurious rebuilds.
import { execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const MARKER = join(root, ".build-head");

// Named files (relative to repo root) whose contents feed the native build.
const NAMED_INPUTS = ["Cargo.toml", "Cargo.lock", "build.ts"];

function hashFile(p) {
  return createHash("sha256").update(readFileSync(p)).digest("hex");
}

// The set of build-input files: the named files plus every tracked .rs file
// under src/. `git ls-files` reads working-tree content, so uncommitted edits
// to these files change the hash and trigger a rebuild (catching the "edited
// src but HEAD unchanged" case).
function listInputFiles() {
  const files = [];
  for (const p of NAMED_INPUTS) {
    if (existsSync(join(root, p))) files.push(p);
  }
  const srcFiles = execSync("git ls-files -- src", {
    cwd: root,
    encoding: "utf8",
  })
    .split("\n")
    .filter(Boolean)
    .filter((f) => f.endsWith(".rs"));
  files.push(...srcFiles);
  return files;
}

function inputsHash() {
  const h = createHash("sha256");
  for (const f of listInputFiles().sort()) {
    h.update(f + "\0");
    h.update(hashFile(join(root, f)));
  }
  return h.digest("hex");
}

function distMissing() {
  return !existsSync(join(root, "dist/index.js")) && !existsSync(join(root, "dist/index.mjs"));
}

function needsBuild() {
  if (distMissing()) return true;
  if (!existsSync(MARKER)) return true;
  try {
    return readFileSync(MARKER, "utf8").trim() !== inputsHash();
  } catch {
    return true;
  }
}

if (needsBuild()) {
  console.log("[ensure-built] build inputs changed (or dist missing) — building…");
  execSync("npm run build", { cwd: root, stdio: "inherit" });
  writeFileSync(MARKER, inputsHash() + "\n");
  console.log("[ensure-built] build complete.");
} else {
  console.log("[ensure-built] dist up to date with build inputs — skipping build.");
}
