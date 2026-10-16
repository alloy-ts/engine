import { describe, expect, test } from "vite-plus/test";
import { Context, Engine } from "./main.ts";

describe("napi: Context", () => {
  test("Context and Engine exports are defined", () => {
    expect(Context).toBeDefined();
    expect(Engine).toBeDefined();
  });
});
