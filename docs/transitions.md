# Transitions

Transitions are selected with `xwww img -t <type>` (or `--transition-type`). The `simple` and
`none` transitions work by stepping each pixel channel toward the target; the rest are
time-based effects controlled by `--transition-duration`, `--transition-fps`, and (where relevant)
`--transition-angle`, `--transition-pos`, `--transition-bezier` and `--transition-wave`.

| Type      | Description |
|-----------|-------------|
| `simple`  | Default. Fades toward the new image by per-channel stepping. |
| `fade`    | Like `simple` but driven by a `--transition-bezier` curve. |
| `left`/`right`/`top`/`bottom` | Directional wipes. |
| `wipe`    | A wipe at an arbitrary `--transition-angle`. |
| `wave`    | A wavy wipe (`--transition-wave` controls wave size). |
| `grow`    | A circle grows from `--transition-pos`. |
| `center`  | Alias of `grow` anchored at the center. |
| `outer`   | A circle shrinks toward `--transition-pos`. |
| `any`     | Randomly picks `grow` or `outer` at a random position. |
| `random`  | Picks a random effect (including `glitch`, `decrypt`, `dissolve`, `clock`, `zoom`, `pixelate`, `ripple`, `blinds`, `spiral`, `static`, `melt` and `shatter`). |
| `none`    | Instantly switches to the new image. |
| `pixelate`| The new image appears as flat color blocks in raster order, then the final `simple` pass sharpens them. |
| `ripple`  | Concentric wavy rings reveal the new image from `--transition-pos`. |
| `blinds`  | Horizontal venetian blinds open in staggered order. |
| `spiral`  | An angular sweep with a radius offset, like a spiral arm unwinding. |
| `static`  | The new image dissolves through television-like static (`--transition-duration` controls how long the noise lasts). |
| `parallax`| Vertical push: the new image enters from the bottom while the old one drifts up slower. |
| `parallax-left` | Horizontal push from the left, old image drifting right slower. |
| `parallax-right` | Horizontal push from the right, old image drifting left slower. |
| `parallax-invert` | Horizontal push from the right where the old image follows the same direction (inverted depth). |
| `melt`    | Ragged vertical drips reveal the new image from the top, each column at its own speed. |
| `shatter` | Random tiles fly in with a scale/rotation animation until the whole image is in place. |

## `glitch`

Reveals the new image with a digital-corruption effect: horizontal bands are randomly torn and
shifted (with wrap-around) and random noise is injected. The amount of corruption decays over the
transition duration until the image settles cleanly.

## `decrypt`

Reveals the new image block by block in a pseudo-random order — like a cipher being decoded. The
image is divided into a grid of `32x32` blocks and each block "snaps" into place in a random order
until the whole image is shown.

## `dissolve`

Reveals the new image pixel by pixel in a pseudo-random order. A shuffled list of every pixel index
is generated up front, so each pixel snaps in exactly once and the image is complete at the end of
the transition.

## `clock`

Sweeps the new image in like a clock hand rotating around `--transition-pos`. Pixels are revealed
in angular order around the center.

## `zoom`

Zooms out from `--transition-pos` (default: center) until the new image fills the output. Pixels
are sampled from the new image with a per-frame scale factor using nearest-neighbor sampling, so it
stays cheap even on large outputs.

## `pixelate`

Reveals the new image as a grid of `48x48` flat color blocks in raster order: each block appears
with the average color of its region and, once every block is in, the effect hands off to a
`simple` pass that sharpens the result.

## `ripple`

Concentric rings expand from `--transition-pos`, revealing the new image. The rings are slightly
wavy, so the reveal edge moves like water.

## `blinds`

Horizontal bands open in staggered order like venetian blinds; odd bands sweep in the opposite
direction.

## `spiral`

The reveal threshold combines the polar angle with the distance from `--transition-pos`, so the new
image unwinds like a spiral arm.

## `static`

The new image dissolves through a random per-pixel threshold while random noise is sprinkled on
top. The noise decays over `--transition-duration`, like a television losing signal.

## `parallax` family

The new image pushes the old one out while both move at different speeds, giving depth to the
switch. `parallax` pushes vertically from the bottom; `parallax-left` and `parallax-right` push
horizontally; `parallax-invert` keeps the horizontal push but makes the old image follow the same
direction instead of lagging behind.

## `melt`

The new image is revealed from the top with a ragged edge: each column has a random speed, so the
boundary drips down unevenly.

## `shatter`

The image is divided into `64x64` tiles that appear in random order, each one flying into place
with a short scale and rotation animation, like broken glass reassembling.

All of them are implemented in `daemon/src/animations/transitions.rs`. The random-order effects
(`glitch`, `decrypt`, `dissolve`, `static`, `shatter`, `melt`) use a tiny embedded xorshift PRNG
rather than a full random-number dependency. `ripple`, `blinds`, `spiral` and `melt` precompute a
per-pixel threshold buffer and reveal pixels as the transition progress passes each threshold;
`pixelate` uses a block schedule and `parallax` builds each frame from a snapshot of the old image.
Every effect finishes by handing off to a `simple` transition so the final frame is pixel-perfect.
