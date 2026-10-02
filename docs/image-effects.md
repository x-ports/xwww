# Image effects

`xwww img` supports three client-side post-processing effects, applied after the image is resized
to the output and before it is sent to the daemon. They only apply to **static** images; animated
images and videos are left untouched.

## `--blur <radius>`

Applies an approximate Gaussian blur. The implementation uses three iterations of a separable box
blur (a standard, cheap Gaussian approximation), each running a horizontal pass followed by a
vertical pass with a sliding window — so the cost is `O(width * height * channels)` per pass,
independent of the radius.

```sh
xwww img wallpaper.png --blur 20
```

Set `--blur 0` (the default) to disable.

## `--dim <factor>`

Darkens the RGB channels by a factor in `[0.0, 1.0]` (`1.0` leaves the image unchanged, `0.0` turns
it black). The alpha channel is never modified.

```sh
xwww img wallpaper.png --dim 0.5
```

## `--map-palette <spec>` / `--map-strength <factor>`

Recolors the wallpaper with a color palette: every pixel's luminance (Rec. 709) is mapped through
a gradient built from the palette colors (plus background and foreground, sorted by luminance), so
dark areas take the darkest color and bright areas the brightest.

```sh
xwww img wallpaper.png --map-palette xwww
xwww img wallpaper.png --map-palette xwww:~/.config/xwww/other.json --map-strength 0.7
xwww img wallpaper.png --map-palette equisdots:london
xwww img wallpaper.png --map-palette file:~/.cache/wal/colors.json
xwww img wallpaper.png --map-palette command:'pywal -c'
```

| Spec | Meaning |
|------|---------|
| `xwww` / `xwww:<path>` | xwww palette file (`~/.config/xwww/palette.json` by default), written by the desktop. |
| `equisdots` / `equisdots:<slug>` | equisdots desktop palette (explicit opt-in): slug from `~/.config/hypr/settings.json` (`bar.palette`, fallback `x`), files under `dock/palettes`. |
| `file:<path>` (or a bare path) | Palette JSON (base16 schema) or a text file with one `#rrggbb` per line. |
| `command:<cmd>` | Parses the stdout of a command the same way as `file`. |

`--map-strength` (default `1.0`) blends the result with the original image (`0.0` is a no-op, `1.0`
fully replaces it).

## Implementation

The effects live in `client/src/effects.rs` and operate directly on the raw interleaved pixel
buffer produced by the resize pipeline, so they are format-aware (3- or 4-channel) without
depending on the `image` crate's higher-level types. Palette loading and normalization live in
`client/src/palette_source.rs`.
