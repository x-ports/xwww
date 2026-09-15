# Image effects

`xwww img` supports two client-side post-processing effects, applied after the image is resized to
the output and before it is sent to the daemon. They only apply to **static** images; animated
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

## Implementation

Both effects live in `client/src/effects.rs` and operate directly on the raw interleaved pixel
buffer produced by the resize pipeline, so they are format-aware (3- or 4-channel) without
depending on the `image` crate's higher-level types.
