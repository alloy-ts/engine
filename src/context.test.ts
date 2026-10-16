import { describe, expect, test } from "vite-plus/test";
import { Context, Runtime } from "./main.ts";

describe("napi: Context", () => {
  test("Context and Runtime exports are defined", () => {
    expect(Context).toBeDefined();
    expect(Runtime).toBeDefined();
  });
});
