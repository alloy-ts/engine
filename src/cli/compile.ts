import { Alloy } from "../core/runtime";

export async function compile(input: string, output: string, options: any = {}) {
    console.log(`Compiling ${input} to ${output}...`);

    // Stage 1: Strip TS
    // Stage 2: runBeforeBundle
    // Stage 3: CPS transform
    // Stage 4: alloyc_compile via Alloy.shell

    // return Alloy.spawn("alloyc_compile", ["-i", input, "-o", output]);
}
