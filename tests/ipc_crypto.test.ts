import { expect, test, beforeAll } from "bun:test";
import * as fc from "fast-check";
import { alloy_ipc_encrypt, alloy_ipc_decrypt } from "../src/core/ipc_sim";

// Feature: alloyscript-dual-engine, Property 5: AEAD encryption round-trip
test("Property 5: AEAD encryption round-trip", () => {
    const rt = { next_outbound_seq: 1, last_inbound_seq: 0, ipc_state: { session_key: new Uint8Array(32), next_outbound_seq: 1, last_inbound_seq: 0 } };
    fc.assert(
        fc.property(
            fc.uint8Array({ minLength: 1, maxLength: 1024 }),
            (payload) => {
                const msg: any = {
                    nonce: new Uint8Array(12),
                    tag: new Uint8Array(16),
                    payload: new Uint8Array(payload),
                    payload_len: payload.length,
                    type: 1
                };

                alloy_ipc_encrypt(rt, msg);
                const res = alloy_ipc_decrypt(rt, msg);

                return res === 0 && msg.payload.every((v: number, i: number) => v === payload[i]);
            }
        ),
        { numRuns: 100 }
    );
});
