# clik-keycap

A [clik](https://clik.aiocean.io) plugin that adds a keyboard key shape for pointing out shortcuts in screenshots: a rounded key with a lighter top face, a darker bottom edge and a centered label. The label shrinks to fit small keys.

## Steps

| Step | Role | What it does |
|---|---|---|
| `keycap` | shape | Draws a keyboard key |

## Props

| Prop | Kind | Default | Meaning |
|---|---|---|---|
| `label` | text | `⌘K` | Text on the key |
| `theme` | `light` \| `dark` | `light` | Key colors |
| `size` | number 10–48 | `16` | Label font size |

No config, no network access.

## Install

In clik, open Settings → Plugins and either find it under **Browse**, or choose **Install from GitHub** and enter `aiocean/clik-keycap`.

## Build

Needs the `wasm32-unknown-unknown` target and `wasm-tools`.

```bash
cargo test
cargo build --release --target wasm32-unknown-unknown
mkdir -p pkg
wasm-tools component new target/wasm32-unknown-unknown/release/clik_keycap.wasm -o pkg/plugin.wasm
cp plugin.toml pkg/
```

Pushing a `v*` tag matching `version` in `plugin.toml` publishes `keycap-<version>.clikplugin.tar.gz` as a GitHub release (`.github/workflows/release.yml`).

## License

MIT
