import { BUILTIN_DEFAULTS } from "./config";

export function buildWhitelist(capabilities: string[]): { isAllowed(binary: string): boolean } {
  const allowed = new Set(capabilities.map((c) => c.replace(/^use-/, "")));
  allowed.add("npm");
  allowed.add("npx");
  // auto-whitelist binaries from package.json would go here
  return {
    isAllowed(binary: string) {
      const b = binary === "npm" ? "alloy" : binary === "npx" ? "alloyx" : binary;
      return allowed.has(binary);
    },
  };
}
