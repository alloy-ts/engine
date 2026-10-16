---
title: 'Loader and Resolver'
description: Module loading and resolution interfaces for QuickJS.
---

# Loader and Resolver

The `Loader` and `Resolver` classes provide module loading and path normalization capabilities.

## Usage

```ts
import { Loader, Resolver } from "@alloy-ts/engine";

const loader = new Loader();

const resolver = new Resolver();
const resolvedPath = resolver.resolve("/app/src/index.js", "./utils.js");
// Returns "/app/src/utils.js"
```
