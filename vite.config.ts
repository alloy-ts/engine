import { defineConfig } from "vite-plus";

export default defineConfig({
  pack: {
    entry: ["./dist/index.js"],
    format: "esm",
    outDir: "dist",
    clean: false,
    exports: true,
    dts: {
      generator: "tsgo",
    },
    deps: {
      resolveDepSubpath: true,
    },
  },
  test: {
    include: ["src/**/*.{test,spec}.ts"],
  },
  staged: {
    "*": "vp check --fix",
    // Cargo-backed steps for Rust sources. A function task receives the matched
    // staged files and controls invocation (lint-staged does not auto-append
    // files to function tasks, and runs no shell). `cargo clippy --fix` runs on
    // the whole crate; `cargo fmt -- <files>` formats only the staged files so
    // unrelated working-tree .rs files are left untouched.
    "*.rs": (files) => [
      "cargo clippy --fix --all-targets --allow-dirty --allow-staged",
      `cargo fmt -- ${files.join(" ")}`,
    ],
  },
  fmt: {},
  lint: {
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: { "vite-plus/prefer-vite-plus-imports": "error" },
    options: { typeAware: true, typeCheck: true },
  },
  run: {
    cache: true,
  },
});
