# bart-web

Browser-based BART departure board, compiled to WebAssembly via [Leptos](https://leptos.dev) and [trunk](https://trunkrs.dev).

## Prerequisites

Enter the bart-rs dev shell (from the workspace root), which provides `trunk` and a Rust toolchain with the `wasm32-unknown-unknown` target:

```sh
nix develop
```

## Build

```sh
cd bart-web
trunk build --release
```

Output lands in `bart-web/dist/` — static files ready to serve.

## Dev server

```sh
cd bart-web
trunk serve
```

Opens a local server with hot-reload at `http://localhost:8080`.

## Serve with nginx

Point nginx at `bart-web/dist/`:

```nginx
location /bart/ {
    root /path/to/bart-rs/bart-web/dist;
    try_files $uri $uri/ /index.html;
}
```
