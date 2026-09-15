# xwww documentation

`xwww` is a Wayland wallpaper daemon (forked from [`awww`](https://codeberg.org/LGFae/awww), which
is itself a fork of `swww`). It runs in the background, draws wallpapers on the `wlr-layer-shell`
protocol, and is controlled at runtime by the `xwww` client.

## Table of contents

- [Architecture](architecture.md) — how the client, daemon and shared library fit together.
- [Commands](commands.md) — reference for every `xwww` subcommand.
- [Transitions](transitions.md) — the available transition effects (including `glitch` and `decrypt`).
- [Image effects](image-effects.md) — `--blur` and `--dim`.
- [Palette](palette.md) — extracting dominant colors (`xwww palette`).
- [Slideshow](slideshow.md) — cycling wallpapers (`xwww slideshow`).
- [Screenshot](screenshot.md) — capturing the current wallpaper (`xwww screenshot`).
- [Video](video.md) — video wallpaper support.
- [IPC protocol](ipc-protocol.md) — the socket + shared-memory message format.
- [Cargo features](cargo-features.md) — build-time feature flags.
- [Future ideas](future-ideas.md) — a wishlist of features and effects to add.

## Quick start

```sh
# build (video support is on by default)
cargo build --release

# start the daemon
./target/release/xwww-daemon &

# set a wallpaper
./target/release/xwww img path/to/image.png

# set a wallpaper with a transition
./target/release/xwww img path/to/image.png --transition-type glitch
```
