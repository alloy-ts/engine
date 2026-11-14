# Controlled Node-API test addon

## Purpose

The first native fixture validates the real C ABI. It must compile against the
selected official Node-API headers and call exported symbols through the ABI;
it must not invoke Rust helpers directly.

## Build variants

- Node reference variant: normal Node headers/link behavior, executed by Node.
- Alloy controlled variant: explicitly linked/routed to the Alloy ABI bridge.

The Alloy variant is not evidence that arbitrary existing `.node` binaries can
be loaded.

## Initial exports

Once their manifest dependencies are implemented, the fixture exports native
functions that test:

- module registration and API-version negotiation;
- undefined, null, boolean, number, and string round trips;
- `napi_typeof` results;
- handle-scope close and escapable-scope behavior;
- stale handle and cross-environment rejection;
- object creation and named properties;
- native callbacks, callback data, arguments, and `this`;
- thrown values and pending-exception retrieval;
- references, wrapping, and finalizer scheduling.

Each fixture operation returns structured primitive results so Node and Alloy
outcomes can be normalized and compared.

## Required negative tests

- wrong environment for a value/reference/scope;
- closed or mismatched handle scope;
- escape called twice;
- invalid output pointer;
- API call while an exception is pending;
- unsupported requested Node-API version;
- initialization returning null with and without a pending exception;
- disposal with outstanding references and callbacks.

## Symbol-routing proof

The Alloy bridge exposes a test-only diagnostic identity through the controlled
fixture. Initialization fails unless calls resolve to that identity. This
prevents a nested test from accidentally passing while using the outer Node
process's Node-API implementation.

## Phase 0 deliverable

This document specifies the fixture only. No C source, build system, bridge
library, or dynamic loading is added until the host/loader designs are
accepted.
