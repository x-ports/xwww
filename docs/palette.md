<a id="top" name="top"></a>

# Palette

`xwww palette` extracts the dominant colors of an image, or of the wallpaper
the daemon is currently displaying. This is useful for generating color schemes
that match the wallpaper, for example to theme a shell, terminal or status bar.
The same palette sources can be consumed by `xwww img --map-palette` and by
JavaScript scenes.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#options">Options</a> &middot;
  <a href="#output">Output</a> &middot;
  <a href="#how-it-works">How it works</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Usage](#usage)
- [Options](#options)
- [Output](#output)
- [How it works](#how-it-works)

</details>

<a id="usage" name="usage"></a>

## Usage

```sh
# from an explicit image
xwww palette path/to/image.png
xwww palette path/to/image.png --count 5

# from the current wallpaper (requires a running daemon)
xwww palette
xwww palette -o HDMI-A-1

# machine-readable output
xwww palette --json
```

<a id="options" name="options"></a>

## Options

| Option | Default | Meaning |
|--------|---------|---------|
| `<image>` | current wallpaper | Image file to analyze. If omitted, the daemon is queried for the wallpaper on the selected output. |
| `-c, --count` | `8` | Number of colors to output. |
| `--json` | off | Emit a JSON array instead of plain hex lines. |
| `-o, --output` | first output | When querying the daemon, which output to use. |
| `-n, --namespace` | `""` | Daemon namespace (when querying). |

If the current wallpaper on the selected output is a solid color rather than an
image, the command fails with a message containing the color.

<a id="output" name="output"></a>

## Output

Plain mode prints one uppercase hex color per line, most frequent first:

```
#2E3440
#81A1C1
#BF616A
```

JSON mode prints an array of objects with `hex`, `r`, `g` and `b` fields:

```json
[
    {
        "hex": "#2E3440",
        "r": 46,
        "g": 52,
        "b": 64
    }
]
```

<a id="how-it-works" name="how-it-works"></a>

## How it works

The image is downsampled to at most `128x128`, pixels are bucketed into a
coarse histogram (4 bits per channel), and the most frequent buckets are
averaged to produce smooth representative colors, ordered from most to least
frequent. Raster formats are decoded with the `image` crate; SVG files are
rendered with `resvg`. The implementation lives in `client/src/palette.rs`.

<div align="center">
  <a href="#top">Back to top</a>
</div>
