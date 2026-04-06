import { Transpiler } from "./transpiler";
import { build } from "./build";
import { ConfigLoader } from "./config/loader";
import { ConfigValidator } from "./config/validator";

declare global {
  const AlloyInternal: {
    parse(code: string): string | null;
    on(event: string, listener: Function): void;
    off(event: string, listener: Function): void;
    emit(event: string, arg: any): void;
    nextTick(func: Function): void;
    getMmap(): any;
    readMmap(region: any): ArrayBuffer;
    useConfig(pluginName: string, schema: any): any;
    configSet(pluginName: string, key: string, value: any, options?: any): void;
  };
}

export const Alloy = {
  Transpiler: Transpiler,
  build: build,
  config: {} as any,
  plugins: {} as any,

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
  },

  useConfig(schema: any) {
    // Should be injected by C host in plugin context
    return (globalThis as any).__alloy_use_config(schema);
  }
};

(globalThis as any).Alloy = Alloy;

// Load initial configuration
try {
  const loader = new ConfigLoader();
  const config = loader.load();
  Alloy.config = Object.freeze(config);
} catch (e) {
  // Config loader handles its own printing/aborting if invalid
}
