# Changelog

All notable changes to xwww are documented here.
Dates use YYYY-MM-DD.

## [2026-10-02] - 0.13.1

### Fixed

- **Scene frames are actually presented.** The daemon commits a transition whose effect already
  finished (instant scene frames) without waiting for its frame budget, and the client sends those
  instant frames at 144 fps. Previously each one waited a full second and the next frame replaced
  its animator before it was drawn, so animated scenes looked frozen on screen and palette changes
  only appeared after re-applying the scene, while `xwww screenshot` already showed the new frames.
- **Palette crossfade on animated scenes.** The crossfade target is refreshed with every new frame,
  so the scene keeps moving while it fades, and the clean frame is restored when the fade ends
  instead of leaving a blend with the previous palette.
- **Palette crossfade with static scenes.** The crossfade restores its clean target before the
  scene paints, so scenes that only redraw when their inputs change (card blocks, dashboards) pick
  up the new palette while fading instead of keeping the old colors until the scene is re-applied.

### Changed

- Scene text runs are cached per glyph, size, family, anchor and color, so scenes that redraw many
  strings (glyph rain, cards) lay them out once instead of going through the font stack every
  frame.
- Output conversion for fully opaque canvases skips alpha compositing.

## [2026-10-02] - 0.13.0

### Added

- **Palette effects**: `xwww img --map-palette <source>` recolors a static wallpaper through a
  palette read from the xwww palette file, the equisdots desktop, a file or a command, with
  `--map-strength` controlling the blend.
- **Scene engine** (behind the `scene` feature): `xwww scene check|render|run` executes a
  JavaScript wallpaper on a tiny-skia canvas with a sandboxed QuickJS runtime (memory and time
  limits), palette providers and image assets behind an allowlist.
- **Canvas text**: `canvas.text(str, x, y, size, color, options)` renders with the system fonts
  through resvg/usvg; options cover family, anchor and bold.
- **Tinted images**: `canvas.image_tinted(path, x, y, w, h, color)` draws an asset with a flat
  color while keeping its alpha, for stencils and masks.
- **Eleven reveal transitions**: `pixelate`, `ripple`, `blinds`, `spiral`, `static`, `parallax`,
  `parallax-left`, `parallax-right`, `parallax-invert`, `melt` and `shatter`.
- **Entry transition for scenes**: the first frame of `xwww scene run` accepts the same transition
  flags as `xwww img`; every frame after that is instant.
- **Palette crossfade for scenes**: `--palette-fade <ms>` (default 800) blends the previous frame
  out when the active palette changes.

### Changed

- Scene frames are only pushed when the canvas actually changes, so idle scenes cost almost no CPU.
- Scene paths are canonicalized before deriving asset directories, so relative script paths work.
- The `xwww` client and `xwww-daemon` share the transition types over IPC; the new effects are
  serialized after the existing ones, keeping older clients compatible.

### Fixed

- Palette crossfade: the fade is rebuilt from the clean new frame and restores it when it ends,
  instead of keeping a blend with the previous palette.
- Scene entry transitions are timed from the moment the first frame is sent, so rendering the
  scene does not eat the animation and cut it short.
- `shatter`, `parallax` and `pixelate` finish inside their duration (tile animations, exact final
  frame and progressive sharpening).
- The daemon compiles with the current `rustix` (the stdio calls are now `unsafe`).
