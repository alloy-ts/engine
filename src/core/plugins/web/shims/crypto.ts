// src/core/plugins/web/shims/crypto.ts

declare const AlloyInternal: {
    nativeInvokeWithCap(command: string, args: string, capToken: string): string;
};

export const crypto = {
    subtle: {
        async digest(algo: string, data: any) {
            return AlloyInternal.nativeInvokeWithCap("web:cryptoDigest", JSON.stringify({ algo, data }), "web:crypto-cap");
        }
    },
    getRandomValues(array: any) {
        // Implementation
        return array;
    }
};
