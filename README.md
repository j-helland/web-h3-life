# About

Game of life simulation on a hexagon grid over the surface of the Earth. The primary goal was to learn about the Rust / WASM ecosystem. I used SolidJS just to cram yet another buzzword in here.

# Development

The core simulation is written in Rust and transpiled to WASM via [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen). The frontend is a simple single-page app written in Typescript with [SolidJS](https://www.solidjs.com). 

## Building

```bash
> npm run build:core   # Compile Rust core -> WASM
> npm run build        # Compile TS
> npm run copy:assets  # ./assets/ contents to dist
> npm run serve        # verify output
```

For faster dev iteration, use
```bash
> npm run dev
```
