# Slideshow

`xwww slideshow` cycles through the images of a directory on a timer. It reuses the exact same
request pipeline as `xwww img`, so every resize/filter/transition option keeps working.

## Usage

```sh
xwww slideshow ~/Pictures/wallpapers
xwww slideshow ~/Pictures/wallpapers --interval 30
xwww slideshow ~/Pictures/wallpapers --random --transition-type fade
```

## Options

| Option | Description |
|--------|-------------|
| `<dir>` | Directory to scan (non-recursive). |
| `-i, --interval` | Seconds between wallpapers (default `60`). |
| `-r, --random` | Shuffle instead of alphabetical order. |
| `-t, --transition-type` | Transition between wallpapers (default `any`). |
| `--resize` | Resize strategy (default `crop`). |
| `-n, --namespace` | Daemon namespace. |

The command loops forever (Ctrl-C to stop). A warning is printed to stderr if an individual image
fails to load, and the loop continues.

## Supported extensions

`png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp`, `tiff`, `tif`, `avif`, `svg`, `jxl`, `qoi`, `exr`,
plus the video extensions handled by the `video` feature (see [video](video.md)).

The implementation lives in `client/src/slideshow.rs`.
