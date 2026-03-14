# Lumo Playground

A web playground built with **Vite + SolidJS + Monaco Editor** that runs both:

- the `lumo-compiler` (for diagnostics/stats), and
- the `lumo-lsp` server bridge (for semantic tokens)

inside WebAssembly.

## Run

From the repository root:

```bash
pnpm playground:dev
```

This runs:

1. `wasm-pack build ../../crates/playground-wasm --target web --out-dir ./src/wasm`
2. `vite`

## Build

```bash
pnpm playground:build
```
