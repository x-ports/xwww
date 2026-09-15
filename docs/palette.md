# Palette

`xwww palette` extracts the dominant colors of an image — or of the wallpaper the daemon is
currently displaying — which is useful for generating color schemes that match your wallpaper
(e.g. for theming your shell, terminal or bars).

## Usage

```sh
# from an explicit image
xwww palette path/to/image.png
xwww palette path/to/image.png --count 5

# from the current wallpaper (requires a running daemon)
xwww palette
xwww palette -o HDMI-A-1            # restrict to one output

# machine-readable output
xwww palette --json
```

## Options

| Option | Description |
|--------|-------------|
| `<image>` | Image file. If omitted, the current wallpaper is used. |
| `-c, --count` | Number of colors to output (default `8`). |
| `-o, --output` | When querying the daemon, which output to use. |
| `--json` | Emit a JSON array instead of hex lines. |
| `-n, --namespace` | Daemon namespace (when querying). |

## Output

Plain mode prints one hex color per line, most frequent first:

```
#2E3440
#81A1C1
#BF616A
```

JSON mode prints an array of objects with `hex`, `r`, `g`, `b` fields.

## How it works

The image is downsampled to at most `128x128`, pixels are bucketed into a small histogram (4 bits
per channel), and the top buckets are averaged to produce smooth representative colors. Raster
formats are decoded with the `image` crate; SVG files are rendered with `resvg`. The
implementation lives in `client/src/palette.rs`.
