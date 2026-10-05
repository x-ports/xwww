<a id="top" name="top"></a>

# Slideshow and random

`xwww slideshow` cycles through the images of a directory on a timer, and
`xwww random` sets one random image and exits. Both reuse the same request
pipeline as `xwww img`, so resize strategies, filters and transitions keep
working.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#slideshow">Slideshow</a> &middot;
  <a href="#random">Random</a> &middot;
  <a href="#supported-files">Supported files</a> &middot;
  <a href="#how-it-works">How it works</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Slideshow](#slideshow)
- [Random](#random)
- [Supported files](#supported-files)
- [How it works](#how-it-works)

</details>

<a id="slideshow" name="slideshow"></a>

## Slideshow

```sh
xwww slideshow ~/Pictures/wallpapers
xwww slideshow ~/Pictures/wallpapers --interval 30
xwww slideshow ~/Pictures/wallpapers --random --transition-type fade
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<dir>` | - | Directory to scan (non-recursive). |
| `-i, --interval` | `60` | Seconds between wallpapers. |
| `-r, --random` | off | Shuffle the order instead of sorting alphabetically. |
| `-t, --transition-type` | `any` | Transition between wallpapers. |
| `--resize` | `crop` | Resize strategy for each image. |
| `-n, --namespace` | `""` | Daemon namespace (the first one is used). |

The command loops forever; press Ctrl-C to stop it. When an individual image
fails to load, a warning is printed to stderr and the loop continues with the
next file. The slideshow always uses the `Lanczos3` filter and targets all
outputs; use `xwww img` directly when you need per-output control.

<a id="random" name="random"></a>

## Random

```sh
xwww random ~/Pictures/wallpapers
xwww random ~/Pictures/wallpapers --transition-type glitch
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<dir>` | - | Directory to choose from (non-recursive). |
| `-t, --transition-type` | `any` | Transition to use. |
| `--resize` | `crop` | Resize strategy. |
| `-n, --namespace` | `""` | Daemon namespace. |

Unlike `slideshow`, this command sets one image and exits.

<a id="supported-files" name="supported-files"></a>

## Supported files

Both commands scan the top level of the directory and select files by extension:

`png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp`, `tiff`, `tif`, `avif`, `svg`,
`jxl`, `qoi`, `exr`.

The `avif` and `jxl` decoders are optional build features. Video files are not
picked up by these commands: pass them explicitly to `xwww img` (see
[Video](video.md)).

<a id="how-it-works" name="how-it-works"></a>

## How it works

The implementation lives in `client/src/slideshow.rs`. For every file it builds
a complete `xwww img` request and forwards it through the same orchestration
used by the regular command, which keeps the behavior consistent and makes the
slideshow inherit future image handling for free.

<div align="center">
  <a href="#top">Back to top</a>
</div>
