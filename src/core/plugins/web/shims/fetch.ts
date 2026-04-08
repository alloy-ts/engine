// src/core/plugins/web/shims/fetch.ts

declare const AlloyInternal: {
    nativeInvokeWithCap(command: string, args: string, capToken: string): string;
};

export async function fetch(url: string, options: any = {}) {
    const resJson = AlloyInternal.nativeInvokeWithCap("web:fetch", JSON.stringify({ url, options }), "web:fetch-cap");
    const res = JSON.parse(resJson);
    return new Response(res.body, res);
}

export class Request {
    constructor(public url: string, public options: any = {}) {}
}

export class Response {
    constructor(private body: any, public options: any = {}) {}
    async text() { return this.body; }
    async json() { return JSON.parse(this.body); }
}
