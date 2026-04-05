import { Transpiler } from "./transpiler";
import { build } from "./build";

export const Alloy: any = (function () {
    const global: any = globalThis;
    let AlloyInternal = global.AlloyInternal;

    const AlloyObj: any = {
        bind_global: function (name: string, value: any) {
            const internal = AlloyInternal || (globalThis as any).AlloyInternal;
            if (internal && internal.bindGlobal) {
                internal.bindGlobal(name, value);
            }
        },
        secureEval: function (code: string) {
            try {
                const transformed = new Transpiler().transformSync(code);
                const internal = AlloyInternal || (globalThis as any).AlloyInternal;
                if (!internal) throw new Error("AlloyInternal not defined");
                return internal.secureEval(transformed);
            } catch (e: any) {
                return { error: "transpile_failed", message: e.message };
            }
        },
        spawnSync: function (cmd: string, args?: string[], opts?: any) {
            return AlloyInternal.spawnSync(cmd, args, opts);
        },
        spawn: function (cmd: string, args?: string[], opts?: any) {
            return AlloyInternal.spawn(cmd, args, opts);
        },
        Transpiler: Transpiler,
        build: build
    };

    if (typeof AlloyInternal === "undefined" && typeof globalThis.AlloyInternal !== "undefined") {
        AlloyInternal = globalThis.AlloyInternal;
    }

    if (AlloyInternal) {
        AlloyObj.on = function(event: string, listener: any) { AlloyInternal.on(event, listener); };
        AlloyObj.off = function(event: string, listener: any) { AlloyInternal.off(event, listener); };
        AlloyObj.emit = function(event: string, arg: any) { AlloyInternal.emit(event, arg); };
        AlloyObj.nextTick = function(fn: any) { AlloyInternal.nextTick(fn); };
        AlloyObj.getMmap = function() { return AlloyInternal.getMmap(); };
        AlloyObj.readMmap = function(region: any) { return AlloyInternal.readMmap(region); };
    }

    global.Alloy = AlloyObj;
    return AlloyObj;
})();
