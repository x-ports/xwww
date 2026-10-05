<a id="top" name="top"></a>

# Screenshot

`xwww screenshot` captures the wallpaper the daemon is currently displaying on
an output and saves it as a PNG file. It captures the wallpaper only, not the
full desktop, which makes it handy for previews, thumbnails and scripts.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#options">Options</a> &middot;
  <a href="#how-it-works">How it works</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Usage](#usage)
- [Options](#options)
- [How it works](#how-it-works)

</details>

<a id="usage" name="usage"></a>

## Usage

```sh
xwww screenshot wallpaper.png
xwww screenshot -o HDMI-A-1 wallpaper.png
xwww screenshot -n mynamespace wallpaper.png
```

<a id="options" name="options"></a>

## Options

| Option | Default | Meaning |
|--------|---------|---------|
| `<output-file>` | - | Destination PNG path. |
| `-o, --monitor` | first output | Output to capture. |
| `-n, --namespace` | `""` | Daemon namespace. |

If the daemon has no wallpaper set on the requested output, the command fails
with a clear message.

<a id="how-it-works" name="how-it-works"></a>

## How it works

1. The client sends a `Screenshot` request carrying the output name (or an
   empty string to use the first available output).
2. The daemon finds the matching wallpaper, reads its current canvas buffer and
   normalizes it to standard RGB, dropping any alpha channel.
3. The daemon sends the raw RGB bytes back over the socket through shared
   memory.
4. The client re-encodes them as a PNG with the `image` crate.

Everything is captured as displayed: whatever frame the daemon is showing (a
still image, an animation frame or a scene frame) is what ends up in the file. The
implementation lives in `client/src/screenshot.rs` on the client side and in
the `Screenshot` handler of `daemon/src/main.rs`.

<div align="center">
  <a href="#top">Back to top</a>
</div>
