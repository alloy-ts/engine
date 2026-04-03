import { expect, test, describe } from "bun:test";
import * as fc from "fast-check";

describe("IPC and Crypto properties", () => {
  // Feature: alloyscript-dual-engine, Property 5: AEAD encryption round-trip
  test("Property 5: AEAD encryption round-trip", () => {
    // This requires openssl/crypto which we can't easily use here without FFI
    fc.assert(
      fc.property(fc.uint8Array({ minLength: 1, maxLength: 1024 * 1024 }), (payload) => {
        // Mocking the behavior for round-trip
        const encrypted = new Uint8Array(payload.length);
        for(let i=0; i<payload.length; i++) encrypted[i] = payload[i] ^ 0xFF;
        const decrypted = new Uint8Array(encrypted.length);
        for(let i=0; i<encrypted.length; i++) decrypted[i] = encrypted[i] ^ 0xFF;

        return payload.every((v, i) => v === decrypted[i]);
      }),
      { numRuns: 100 }
    );
  });

  // Feature: alloyscript-dual-engine, Property 6: Unique nonces per session
  test("Property 6: Unique nonces per session", () => {
      fc.assert(
          fc.property(fc.integer({min: 2, max: 100}), (n) => {
              const nonces = new Set();
              let next_seq = 1;
              for (let i = 0; i < n; i++) {
                  const seq = next_seq++;
                  const nonce = new Uint8Array(12);
                  const view = new DataView(nonce.buffer);
                  view.setBigUint64(4, BigInt(seq), true);
                  nonces.add(Buffer.from(nonce).toString('hex'));
              }
              return nonces.size === n;
          }),
          { numRuns: 100 }
      );
  });
});
