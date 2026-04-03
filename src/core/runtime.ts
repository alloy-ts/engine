import { Transpiler } from "./transpiler";
import { build, type BuildOptions } from "./build";

declare global {
  const AlloyInternal: {
    parse(code: string): string | null;
    on(event: string, listener: Function): void;
    off(event: string, listener: Function): void;
    emit(event: string, arg: any): void;
    nextTick(func: Function): void;
    getMmap(): any;
    readMmap(region: any): ArrayBuffer;
  };
}

export const Alloy = {
  Transpiler: Transpiler,

  build: build,

  secureEval(source: string): any {
    const transpiler = new Transpiler();
    try {
      const transformed = transpiler.transformSync(source, "tsx");
      const parseError = (globalThis as any).AlloyInternal.parse(transformed);
      if (parseError) return { error: "transpile_failed", message: parseError };
      return (globalThis as any).AlloyInternal.secureEval(transformed);
    } catch (e: any) {
      return { error: "transpile_failed", message: e.message };
    }
  },

  on(event: string, listener: Function) {
    (globalThis as any).AlloyInternal.on(event, listener);
  },

  off(event: string, listener: Function) {
    (globalThis as any).AlloyInternal.off(event, listener);
  },

  emit(event: string, arg: any) {
    (globalThis as any).AlloyInternal.emit(event, arg);
  },

  nextTick(func: Function) {
    (globalThis as any).AlloyInternal.nextTick(func);
  },

  getMmap() {
    return (globalThis as any).AlloyInternal.getMmap();
  },

  readMmap(region: any) {
    return (globalThis as any).AlloyInternal.readMmap(region);
  }
};

(globalThis as any).Alloy = Alloy;
