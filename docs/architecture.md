<a id="top" name="top"></a>

# Architecture

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#workspace-layout">Workspace layout</a> &middot;
  <a href="#how-it-works">How it works</a> &middot;
  <a href="#module-layout">Module layout</a> &middot;
  <a href="#cache-and-restore">Cache and restore</a> &middot;
  <a href="#testing">Testing</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Workspace layout](#workspace-layout)
- [How it works](#how-it-works)
- [Module layout](#module-layout)
  - [Client](#client)
  - [Daemon](#daemon)
  - [Common](#common)
- [Cache and restore](#cache-and-restore)
- [Testing](#testing)

</details>

<a id="workspace-layout" name="workspace-layout"></a>

## Workspace layout

`xwww` is a Cargo workspace split into four crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `common` | `no_std` library | IPC protocol, mmap helpers, cache, frame compression, logging. |
| `client` | binary `xwww` | Decodes images and videos, builds requests, talks to the daemon. |
| `daemon` | binary `xwww-daemon` | Wayland compositor client; draws and animates wallpapers. |
| `tests` | integration tests | End-to-end tests (most require a Wayland session) and a spell-check test. |

`common` and `daemon` are `#![no_std]` (with `alloc`); only `client` links the
Rust standard library.

<a id="how-it-works" name="how-it-works"></a>

## How it works

1. The **daemon** connects to the Wayland compositor, creates one `wl_surface`
   per output on the `zwlr_layer_shell_v1` protocol, and listens on a UNIX
   stream socket inside `$XDG_RUNTIME_DIR` (or `/run/user/<uid>` when
   `XDG_RUNTIME_DIR` is not set). The socket name is
   `$WAYLAND_DISPLAY-xwww-daemon[.<namespace>].sock`.
2. The **client** queries the daemon for the output list and dimensions,
   decodes and resizes the wallpaper to match each output (grouping outputs
   that share dimensions and current content), then sends the raw pixels plus
   an optional transition and animation over the socket.
3. The **daemon** blits the pixels into its shared-memory `wl_buffer`, applies
   the requested transition, and commits the surface.

Pixel data travels with zero copies through shared memory: the client writes
into a sealed `memfd` (falling back to POSIX `shm`) and passes the file
descriptor with `SCM_RIGHTS`. See [IPC protocol](ipc-protocol.md) for the
message format.

The client is the only side that does heavy pixel work (decode, resize,
effects, frame compression). The daemon is a frame consumer: it never decodes
a user file and never runs a script.

<a id="module-layout" name="module-layout"></a>

## Module layout

<a id="client" name="client"></a>

### Client

- `cli.rs` - `clap` argument definitions for every subcommand, shared with the
  build script that generates shell completions.
- `main.rs` - orchestration: parses arguments, queries the daemon, builds
  requests and sends them; also hosts the scene run/render loops.
- `imgproc.rs` - image decoding (`image` + `resvg`), resize strategies
  (`fast_image_resize`), frame compression and transition construction.
- `effects.rs` - `--blur`, `--dim` and `--map-palette` post-processing.
- `palette.rs` - the `palette` subcommand (dominant-color extraction).
- `palette_source.rs` - palette loading and normalization (`ScenePalette`) for
  `--map-palette` and scenes: `xwww`, `equisdots`, `file` and `command`
  sources.
- `scene/` - the JavaScript scene engine behind the `scene` feature:
  `canvas.rs` (tiny-skia), `runtime.rs` (QuickJS bindings), `providers.rs`
  (palette and clock).
- `slideshow.rs` - the `slideshow` and `random` subcommands.
- `screenshot.rs` - the `screenshot` subcommand.
- `video.rs` - video decoding through `ffmpeg-next` (behind the `video`
  feature).

<a id="daemon" name="daemon"></a>

### Daemon

- `main.rs` - Wayland event loop, socket handling, signals, the `Daemon`
  struct.
- `animations.rs`, `animations/transitions.rs`, `animations/keyframe.rs` - the
  animator and all transition effects.
- `wallpaper.rs`, `wallpaper/cell.rs`, `wallpaper/bump_pool.rs` - per-output
  layer surface and buffer management.
- `wayland.rs` - generated bindings for the Wayland protocols.
- `clock.rs`, `output_info.rs`, `systemd.rs` - timing, output metadata and
  `sd_notify` ready notification.

<a id="common" name="common"></a>

### Common

- `ipc/` - socket, message transmission, request/answer types.
- `mmap.rs` - shared-memory mapping helpers (memfd plus POSIX `shm`).
- `cache.rs` - per-output cache of the last wallpaper and animation frames.
- `compression/` - SIMD-accelerated frame delta compression (LZ4).
- `log.rs`, `path.rs` - logging and path utilities.

<a id="cache-and-restore" name="cache-and-restore"></a>

## Cache and restore

Every `xwww img` request updates a small per-output cache entry with the image
path and the resize, crop-gravity and filter settings used. When an output is
created or reconfigured, the daemon reads that entry and, if a wallpaper is not
already set, re-runs `xwww img` internally to restore it. `xwww restore` does
the same on demand. Animated wallpapers additionally cache preprocessed frames
keyed by path, dimensions, resize strategy and pixel format.

Scene frames bypass the on-disk image cache (`no_cache` requests) and use a
synthetic `scene:<path>` identifier so that `xwww query` stays meaningful
without polluting `restore`.

<a id="testing" name="testing"></a>

## Testing

- Unit tests live next to the modules (palette parsing, canvas, runtime, path
  handling, compression, and so on).
- `tests/integration.rs` drives a live daemon through `assert_cmd` and is
  ignored by default because it needs a Wayland session.
- `tests/spell_check.rs` runs `codespell` over the sources, man pages and
  markdown documentation; it is ignored because `codespell` is not a build
  dependency.

Run everything (including ignored tests) inside a Wayland session with
`cargo test --workspace -- --include-ignored`.

<div align="center">
  <a href="#top">Back to top</a>
</div>
