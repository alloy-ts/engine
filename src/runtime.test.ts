import { describe, expect, test } from "vite-plus/test";
import { Loader, Runtime } from "./main.ts";

describe("napi: Runtime & Loader", () => {
  test("Runtime and Loader exports are defined", () => {
    expect(Runtime).toBeDefined();
    expect(Loader).toBeDefined();
  });
});
