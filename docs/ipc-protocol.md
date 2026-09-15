# IPC protocol

The `xwww` client and daemon communicate over a UNIX stream socket. Control messages are small
fixed-size payloads; bulk data (images, output lists, screenshot data) travels through shared
memory by passing a file descriptor with `SCM_RIGHTS`.

## Socket location

The daemon creates its socket at `$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY-xwww-daemon[.namespace].sock`.
A namespace (`-n/--namespace`) allows multiple independent daemons to coexist on one Wayland
display.

## Message envelope

Every message is a 16-byte header:

```
bytes 0..8   : message code (u64, little-endian)
bytes 8..16  : shared-memory length in bytes (0 if none)
```

If the length is non-zero, an `SCM_RIGHTS` ancillary message carrying a shared-memory file
descriptor (a sealed `memfd` on Linux, falling back to POSIX `shm`) accompanies the first byte of
the payload.

## Message codes

| Code | Name | Direction | Carries shm |
|------|------|-----------|-------------|
| 0  | `ReqPing`       | client → daemon | no |
| 1  | `ReqQuery`      | client → daemon | no |
| 2  | `ReqClear`      | client → daemon | yes (outputs + color) |
| 3  | `ReqImg`        | client → daemon | yes (pixels + transition + outputs) |
| 4  | `ReqKill`       | client → daemon | no |
| 5  | `ResOk`         | daemon → client | no |
| 6  | `ResConfigured` | daemon → client | no |
| 7  | `ResAwait`      | daemon → client | no |
| 8  | `ResInfo`       | daemon → client | yes (output info) |
| 9  | `ReqToggle`     | client → daemon | no |
| 10 | `ReqPause`      | client → daemon | no |
| 11 | `ReqUnpause`    | client → daemon | no |
| 12 | `ReqScreenshot` | client → daemon | yes (output name) |
| 13 | `ResScreenshot` | daemon → client | yes (width, height, RGB) |

## Notable payloads

- **`ReqImg`**: a serialized `Transition` (52 bytes), an image count byte, then for each image its
  path, raw pixels, dimensions, pixel format, output names, and an optional delta-compressed
  animation (see `common/src/ipc/types.rs`).
- **`ResInfo`**: one byte count, then a sequence of serialized `BgInfo` records (name, dimensions,
  scale, currently-displayed image, pixel format).
- **`ResScreenshot`**: `width` (u32), `height` (u32), followed by `width * height * 3` bytes of
  RGB pixels.

All serialization/deserialization lives in `common/src/ipc/{mod,types,transmit,socket}.rs`.
