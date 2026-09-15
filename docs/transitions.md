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
| `random`  | Picks a random effect (now includes `glitch`, `decrypt`, `dissolve`, `clock` and `zoom`). |
| `none`    | Instantly switches to the new image. |

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

`glitch`, `decrypt`, `dissolve`, `clock` and `zoom` are all implemented in
`daemon/src/animations/transitions.rs`. The random-order effects (`glitch`, `decrypt`, `dissolve`)
use a tiny embedded xorshift PRNG rather than a full random-number dependency, and every effect
finishes by handing off to a `simple` transition so the final frame is pixel-perfect.
