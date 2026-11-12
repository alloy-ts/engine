// Ensures dist/ is built from the current Git HEAD.
// If the recorded build HEAD doesn't match the current HEAD (or dist is
// missing), runs `npm run build` so downstream type-checks have fresh output.
// This avoids rebuilding on every `npm run fix` invocation when nothing that
// affects the build has changed.
import { execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";

const MARKER = ".build-head";

function head() {
  return execSync("git rev-parse HEAD", { encoding: "utf8" }).trim();
}

function distMissing() {
  return !existsSync("dist/index.js") && !existsSync("dist/index.mjs");
}

function needsBuild() {
  if (distMissing()) return true;
  if (!existsSync(MARKER)) return true;
  try {
    return readFileSync(MARKER, "utf8").trim() !== head();
  } catch {
    return true;
  }
}

if (needsBuild()) {
  console.log("[ensure-built] dist not built from current HEAD — building…");
  execSync("npm run build", { stdio: "inherit" });
  writeFileSync(MARKER, head() + "\n");
  console.log("[ensure-built] build complete.");
} else {
  console.log(
    "[ensure-built] dist already built from current HEAD " + head() + " — skipping build.",
  );
}
