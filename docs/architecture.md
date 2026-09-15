# Architecture

`xwww` is a Cargo workspace split into four crates:

| Crate   | Type       | Purpose                                                        |
|---------|------------|----------------------------------------------------------------|
| `common`| `no_std` lib | IPC protocol, mmap helpers, cache, frame compression, logging. |
| `client`| binary `xwww` | Decodes images/videos, builds requests, talks to the daemon.   |
| `daemon`| binary `xwww-daemon` | Wayland compositor client; draws and animates wallpapers. |
| `tests` | integration tests | End-to-end tests (require a Wayland session).                |

## How it works

1. The **daemon** connects to the Wayland compositor, creates a `wl_surface` per output on the
   `zwlr_layer_shell_v1` protocol, and listens on a UNIX socket in `$XDG_RUNTIME_DIR`
   (named `$WAYLAND_DISPLAY-xwww-daemon.socket`).
2. The **client** reads an image (or video), decodes and resizes it to match each output, then
   sends the raw pixels plus an optional animation to the daemon over the socket.
3. The **daemon** blits the pixels into its shared-memory `wl_buffer`, applies the requested
   transition, and commits the surface.

Pixel data is transferred with zero copies through shared memory: the client writes into a
`memfd`/`shm` mapping and passes the file descriptor to the daemon via `SCM_RIGHTS`. See
[IPC protocol](ipc-protocol.md) for details.

## Client module layout

- `cli.rs` — `clap` argument definitions for every subcommand.
- `main.rs` — orchestration: parses args, builds requests, sends them.
- `imgproc.rs` — image decoding (`image` + `resvg`), resizing (`fast_image_resize`), frame
  compression.
- `effects.rs` — `--blur` and `--dim` post-processing.
- `palette.rs` — the `palette` subcommand (dominant-color extraction).
- `slideshow.rs` — the `slideshow` subcommand.
- `screenshot.rs` — the `screenshot` subcommand.
- `video.rs` — video decoding via `ffmpeg-next` (behind the `video` feature).

## Daemon module layout

- `main.rs` — event loop, socket handling, signal handling, the `Daemon` struct.
- `animations.rs` / `animations/transitions.rs` / `animations/keyframe.rs` — the animator and all
  transition effects.
- `wallpaper.rs` / `wallpaper/cell.rs` / `wallpaper/bump_pool.rs` — per-output surface + buffer
  management.
- `wayland.rs` — generated bindings for the Wayland protocols.
- `clock.rs`, `output_info.rs`, `systemd.rs` — timing, output metadata, `sd_notify` support.

## Common module layout

- `ipc/` — socket, message transmission, request/answer types.
- `mmap.rs` — shared-memory mapping helpers (memfd + POSIX shm).
- `cache.rs` — per-output cache of the last wallpaper.
- `compression/` — SIMD-accelerated frame delta compression (LZ4).
- `log.rs`, `path.rs` — logging and path utilities.
