# Screenshot

`xwww screenshot` captures the wallpaper that the daemon is currently displaying on an output and
saves it as a PNG file. This captures *the wallpaper only* (not the full desktop), which is handy
for previews and scripting.

## Usage

```sh
xwww screenshot wallpaper.png
xwww screenshot -o HDMI-A-1 wallpaper.png
xwww screenshot -n myns wallpaper.png
```

## Options

| Option | Description |
|--------|-------------|
| `<output-file>` | Destination path (PNG). |
| `-o, --monitor` | Which output to capture (default: first available). |
| `-n, --namespace` | Daemon namespace. |

## How it works

1. The client sends a `Screenshot` request carrying the output name.
2. The daemon finds the matching wallpaper, reads its current canvas buffer, and normalizes it to
   standard RGB (dropping any alpha channel).
3. The daemon sends the raw RGB bytes back over the socket (via shared memory).
4. The client re-encodes them as a PNG with the `image` crate.

The implementation spans `client/src/screenshot.rs` (client side) and `daemon/src/main.rs`
(`RequestRecv::Screenshot` handler + `normalize_rgb`).
