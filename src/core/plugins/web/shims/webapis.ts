// src/core/plugins/web/shims/webapis.ts

export class URL {
    constructor(public url: string, public base?: string) {}
}

export class TextEncoder {
    encode(str: string) { return new Uint8Array(); }
}

export class TextDecoder {
    decode(data: any) { return ""; }
}
