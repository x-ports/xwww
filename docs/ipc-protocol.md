<a id="top" name="top"></a>

# IPC protocol

The `xwww` client and `xwww-daemon` communicate over a UNIX stream socket.
Control messages are small fixed-size payloads; bulk data (images, output
lists, screenshot data) travels through shared memory by passing a file
descriptor with `SCM_RIGHTS`.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#socket-location">Socket location</a> &middot;
  <a href="#message-envelope">Message envelope</a> &middot;
  <a href="#message-codes">Message codes</a> &middot;
  <a href="#notable-payloads">Notable payloads</a> &middot;
  <a href="#source-files">Source files</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Socket location](#socket-location)
- [Message envelope](#message-envelope)
- [Message codes](#message-codes)
- [Notable payloads](#notable-payloads)
- [Source files](#source-files)

</details>

<a id="socket-location" name="socket-location"></a>

## Socket location

The daemon creates its socket at:

```
$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY-xwww-daemon[.<namespace>].sock
```

If `XDG_RUNTIME_DIR` is not set, it falls back to `/run/user/<uid>`. If
`WAYLAND_DISPLAY` is not set, it logs a warning and uses `wayland-0`. A
namespace (`-n/--namespace`) lets several independent daemons coexist on the
same Wayland display; the namespace is inserted before `.sock`.

<a id="message-envelope" name="message-envelope"></a>

## Message envelope

Every message is a 16-byte header:

```
bytes 0..8   : message code (u64, native-endian)
bytes 8..16  : shared-memory length in bytes (0 if none)
```

If the length is non-zero, an `SCM_RIGHTS` ancillary message carrying a
shared-memory file descriptor (a sealed `memfd` on Linux, falling back to POSIX
`shm`) accompanies the first byte of the payload. Numeric fields inside the
shared-memory payload are also native-endian.

<a id="message-codes" name="message-codes"></a>

## Message codes

| Code | Name | Direction | Carries shared memory |
|------|------|-----------|-----------------------|
| 0 | `ReqPing` | client to daemon | no |
| 1 | `ReqQuery` | client to daemon | no |
| 2 | `ReqClear` | client to daemon | yes (outputs plus color) |
| 3 | `ReqImg` | client to daemon | yes (pixels, transition, outputs, animation) |
| 4 | `ReqKill` | client to daemon | no |
| 5 | `ResOk` | daemon to client | no |
| 6 | `ResConfigured` | daemon to client | no |
| 7 | `ResAwait` | daemon to client | no |
| 8 | `ResInfo` | daemon to client | yes (output info) |
| 9 | `ReqToggle` | client to daemon | no |
| 10 | `ReqPause` | client to daemon | no |
| 11 | `ReqUnpause` | client to daemon | no |
| 12 | `ReqScreenshot` | client to daemon | yes (output name) |
| 13 | `ResScreenshot` | daemon to client | yes (width, height, RGB) |

New codes are appended so that the numbering of existing ones never changes.
The client pings until the daemon answers `ResConfigured` (`ResAwait` means the
daemon is still initializing).

<a id="notable-payloads" name="notable-payloads"></a>

## Notable payloads

### `ReqClear`

One byte with the number of outputs, then for each output a `u32` length plus
the UTF-8 name, then four bytes of RGBA color.

### `ReqImg`

A serialized `Transition` (51 bytes: type, duration, step, fps, angle, both
position coordinates as a discriminant plus `f32`, bezier, wave and the
invert-y flag), one byte with the number of images, and then for each image:

1. path: `u32` length plus UTF-8 bytes;
2. raw pixels: `u32` length plus bytes;
3. dimensions: two `u32` values;
4. pixel format: one byte (`0` Bgr, `1` Rgb, `2` Abgr, `3` Argb);
5. output count byte plus the output names (each a `u32` length plus bytes);
6. one animation flag byte and, when set, the animation: a `u32` frame count
   followed by each delta-compressed frame and its duration in nanoseconds.

### `ReqScreenshot`

A `u32` length plus the output name, or an empty string to capture the first
available output.

### `ResInfo`

One byte with the number of outputs, then a sequence of serialized `BgInfo`
records: name, dimensions, scale factor (kind plus `i32`), currently displayed
content (`color` plus RGBA, or `image` plus path) and pixel format.

### `ResScreenshot`

`width` and `height` as `u32`, followed by `width * height * 3` bytes of RGB
pixels (no alpha).

<a id="source-files" name="source-files"></a>

## Source files

All serialization and deserialization live in
`common/src/ipc/{mod,types,transmit,socket}.rs`. Shared-memory mapping helpers
(`memfd` plus POSIX `shm`) live in `common/src/mmap.rs`.

<div align="center">
  <a href="#top">Back to top</a>
</div>
