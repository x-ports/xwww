<a id="top" name="top"></a>

# Transitions

Transitions are selected with `xwww img -t <type>` (or
`--transition-type <type>`), and also by `xwww scene run` for its entry frame.
They are implemented by the daemon on top of the raw pixel buffers, so they
work for static images, animated images and scenes alike.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#overview">Overview</a> &middot;
  <a href="#reference">Reference</a> &middot;
  <a href="#parameters">Parameters</a> &middot;
  <a href="#implementation-notes">Implementation notes</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Overview](#overview)
- [Reference](#reference)
- [Parameters](#parameters)
- [Implementation notes](#implementation-notes)

</details>

<a id="overview" name="overview"></a>

## Overview

The `simple` and `none` transitions work by stepping each pixel channel toward
the target value, so they do not have a fixed duration. Every other effect is
time-based: it runs for `--transition-duration` seconds at
`--transition-fps` frames per second, using the shape parameters
`--transition-angle`, `--transition-pos`, `--transition-bezier` and
`--transition-wave` where relevant. Every effect ends by handing off to a
`simple` pass so the final frame is pixel-perfect.

```sh
xwww img image.png --transition-type grow --transition-pos 0.5,0.5 --transition-duration 2
xwww img image.png --transition-type wipe --transition-angle 30
xwww img image.png --transition-type random
xwww img image.png --transition-type none
```

<a id="reference" name="reference"></a>

## Reference

| Type | Description |
|------|-------------|
| `none` | Switches to the new image instantly (`simple` with step 255). |
| `simple` | Default. Fades by stepping each channel toward the target. |
| `fade` | Like `simple`, driven by a `--transition-bezier` curve. |
| `left` | Directional wipe entering from the left. |
| `right` | Directional wipe entering from the right. |
| `top` | Directional wipe entering from the top. |
| `bottom` | Directional wipe entering from the bottom. |
| `wipe` | Wipe at an arbitrary `--transition-angle`. |
| `wave` | Wavy wipe; `--transition-wave` controls the wave size. |
| `grow` | A circle grows from `--transition-pos`. |
| `center` | `grow` anchored at the center of the output. |
| `outer` | A circle shrinks toward `--transition-pos`. |
| `any` | Randomly picks `grow` or `outer` at a random position. |
| `random` | Picks one of `fade`, `wave`, `outer`, `grow`, `glitch`, `decrypt`, `dissolve`, `clock` or `zoom` with random shape parameters. |
| `glitch` | Digital corruption: torn bands, channel shifts and noise that decay over time. |
| `decrypt` | Reveals the image in `32x32` blocks in pseudo-random order. |
| `dissolve` | Reveals the image pixel by pixel in pseudo-random order. |
| `clock` | Sweeps the image in angularly, like a clock hand around `--transition-pos`. |
| `zoom` | Zooms out from `--transition-pos` (default center). |
| `pixelate` | Flat `48x48` color blocks appear in raster order, then sharpen. |
| `ripple` | Concentric wavy rings reveal the image from `--transition-pos`. |
| `blinds` | Horizontal venetian blinds open in staggered order. |
| `spiral` | An angular sweep with a radius offset unwinds from `--transition-pos`. |
| `static` | The image fades in through television-like static. |
| `parallax` | Vertical push: the new image enters from the bottom while the old one drifts up slower. |
| `parallax-left` | Horizontal push from the left, old image drifting right slower. |
| `parallax-right` | Horizontal push from the right, old image drifting left slower. |
| `parallax-invert` | Horizontal push where the old image follows the same direction as the new one. |
| `melt` | Ragged vertical drips reveal the image from the top, each column at its own speed. |
| `shatter` | Random tiles fly in with a scale and rotation animation. |

### Effect notes

**`glitch`** - horizontal bands are randomly torn and shifted with wrap-around,
and random noise is injected. The corruption intensity decays over
`--transition-duration` until the image settles cleanly.

**`decrypt`** - the image is divided into a grid of `32x32` blocks; each block
snaps into place in a pseudo-random order, like a cipher being decoded.

**`dissolve`** - a shuffled list of every pixel index is generated up front, so
each pixel snaps in exactly once and the image is complete at the end.

**`clock`** - pixels are revealed in angular order around `--transition-pos`,
like a clock hand sweeping the output.

**`zoom`** - pixels are sampled from the new image with a per-frame scale factor
using nearest-neighbor sampling, which keeps the effect cheap on large outputs.

**`pixelate`** - blocks are drawn with the average color of their region; once
the whole image is blocked in, the effect hands off to a `simple` pass that
sharpens it to the final frame.

**`ripple`, `blinds`, `spiral`, `melt`** - these precompute a per-pixel
threshold buffer and reveal pixels as the transition progress passes each
threshold. `blinds` sweeps odd bands in the opposite direction; `ripple` uses
slightly wavy rings; `spiral` combines the polar angle with the distance from
`--transition-pos`; `melt` gives each column its own speed.

**`static`** - a random per-pixel threshold reveals the new image while noise is
sprinkled on top; the noise decays over `--transition-duration`.

**`parallax` family** - the new image pushes the old one out while both move at
different speeds. `parallax` pushes vertically from the bottom;
`parallax-left` and `parallax-right` push horizontally; `parallax-invert` keeps
the horizontal push but makes the old image follow the same direction instead
of lagging behind.

**`shatter`** - the image is divided into `64x64` tiles that appear in random
order, each flying into place with a short scale and rotation animation.

<a id="parameters" name="parameters"></a>

## Parameters

| Option | Used by | Meaning |
|--------|---------|---------|
| `--transition-duration <s>` | all time-based | Total duration in seconds; ignored by `simple`/`none`. |
| `--transition-fps <n>` | all time-based | Frames per second generated by the daemon. |
| `--transition-step <1-255>` | `simple`, `none` | How far each channel moves per frame; 255 is instant. |
| `--transition-angle <deg>` | `wipe`, `wave` | Wipe direction, where `0` is right-to-left, `90` top-to-bottom and `270` bottom-to-top. |
| `--transition-pos <x,y>` | `grow`, `outer`, `zoom`, `clock`, radial effects | Center of the effect. Percent values are floats (`0.5,0.5`), pixels are integers (`200,400`), and the keywords `center`, `top`, `bottom`, `left`, `right` and the four corners are accepted. |
| `--transition-bezier <a,b,c,d>` | `fade`, circle effects | Timing curve; see [cubic-bezier.com](https://cubic-bezier.com). Default `.54,0,.34,.99`. |
| `--transition-wave <w,h>` | `wave` | Wave width and height. |
| `--invert-y` | all | Flips the y coordinate given in `--transition-pos`. |

`--transition-pos` accepts both percent and pixel values because floats parse
as percentages and integers as pixels. Out-of-range percent values print a
warning but are used as given.

<a id="implementation-notes" name="implementation-notes"></a>

## Implementation notes

All effects live in `daemon/src/animations/transitions.rs`. The random-order
effects (`glitch`, `decrypt`, `dissolve`, `static`, `shatter`, `melt`) use a
small embedded xorshift PRNG instead of a random-number crate. Pixel-level
effects precompute threshold or order buffers once and then reveal pixels as
the transition progress passes each threshold; `parallax` builds each frame
from a snapshot of the old image. The same file also contains the shared
keyframe machinery in `daemon/src/animations/keyframe.rs` used for easing.

<div align="center">
  <a href="#top">Back to top</a>
</div>
