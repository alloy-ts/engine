import { describe, expect, test } from "vite-plus/test";
import { Loader, Resolver } from "./main.ts";

describe("napi: Loader and Resolver", () => {
  test("Resolver resolves relative module paths", () => {
    const resolver = new Resolver();
    expect(resolver.resolve("src/app.js", "./utils.js")).toBe("src/utils.js");
    expect(resolver.resolve("src/app.js", "lodash")).toBe("lodash");
  });

  test("Loader loads module content by name", () => {
    const loader = new Loader();
    const content = loader.load("./utils.js");
    expect(content).toContain("// Module: ./utils.js");
  });
});
