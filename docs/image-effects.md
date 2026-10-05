<a id="top" name="top"></a>

# Image effects

`xwww img` supports three client-side post-processing effects. They are applied
after the image is decoded and resized to an output and before the pixels are
sent to the daemon, per output. They only apply to **static** images: animated
images (GIF, WebP, APNG) and videos are sent untouched.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#blur">blur</a> &middot;
  <a href="#dim">dim</a> &middot;
  <a href="#map-palette">map-palette</a> &middot;
  <a href="#implementation">Implementation</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [blur](#blur)
- [dim](#dim)
- [map-palette](#map-palette)
- [Implementation](#implementation)

</details>

<a id="blur" name="blur"></a>

## `--blur <radius>`

Applies an approximate Gaussian blur with the given radius in pixels. The
implementation runs three iterations of a separable box blur (a standard, cheap
Gaussian approximation), each with a horizontal pass followed by a vertical
pass over a sliding window, so the cost is `O(width * height * channels)` per
pass regardless of the radius.

```sh
xwww img wallpaper.png --blur 20
```

The default `--blur 0` disables the effect.

<a id="dim" name="dim"></a>

## `--dim <factor>`

Darkens the RGB channels by a factor in `[0.0, 1.0]`. `1.0` (the default)
leaves the image unchanged and `0.0` turns it fully black. The alpha channel is
never modified.

```sh
xwww img wallpaper.png --dim 0.5
```

<a id="map-palette" name="map-palette"></a>

## `--map-palette <spec>` and `--map-strength <factor>`

Recolors the wallpaper with a palette. Every pixel's Rec. 709 luminance is
mapped through a gradient built from the palette colors plus the palette
background and foreground, sorted by luminance and deduplicated, so dark areas
take the darkest color and bright areas the brightest. `--map-strength`
(default `1.0`) blends the recolored result with the original image, where
`0.0` is a no-op and `1.0` fully replaces the colors.

```sh
xwww img wallpaper.png --map-palette xwww
xwww img wallpaper.png --map-palette xwww:~/.config/xwww/other.json --map-strength 0.7
xwww img wallpaper.png --map-palette equisdots:london
xwww img wallpaper.png --map-palette file:~/.cache/wal/colors.json
xwww img wallpaper.png --map-palette command:'pywal -c'
```

| Spec | Meaning |
|------|---------|
| `xwww` / `xwww:<path>` | Reads the xwww palette file (`$XDG_CONFIG_HOME/xwww/palette.json`, or `~/.config/xwww/palette.json`), which the desktop is expected to write. |
| `equisdots` / `equisdots:<slug>` | Reads the equisdots desktop palette (explicit opt-in): the active slug comes from `~/.config/hypr/settings.json` (`bar.palette`, legacy `dock.palette`, fallback `x`), and files live under `~/.config/hypr/scripts/quickshell/dock/palettes` (including one level of subdirectories such as `community/`). |
| `file:<path>` (or a bare path) | Palette JSON (base16 schema) or a text file with one `#rrggbb` per line. Lines may also use `key=#rrggbb`, and comments start with `//` or `;`. |
| `command:<cmd>` | Runs the command with `sh -c` and parses its stdout like `file`. |

Palette JSON must contain a `base16` object with `color0` through `color15`;
`background` (default `color0`), `foreground` (default `color7`), `slug`,
`name` and `roles` are optional. See the [Palette file contract](scene.md#palette-file-contract)
for the full schema.

<a id="implementation" name="implementation"></a>

## Implementation

The effects live in `client/src/effects.rs` and operate directly on the raw
interleaved pixel buffer produced by the resize pipeline, so they are
format-aware (3 or 4 channels) without depending on the `image` crate's
higher-level types. Palette loading and normalization live in
`client/src/palette_source.rs` and produce a single `ScenePalette` type shared
with the [Scene engine](scene.md).

<div align="center">
  <a href="#top">Back to top</a>
</div>
