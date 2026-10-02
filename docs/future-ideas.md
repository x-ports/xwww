# Future ideas

A running wishlist of features and effects that would fit `xwww`. The ones marked *(client)* can
be implemented entirely in the client; *(daemon)* require changes to the daemon and IPC.

## Effects

- **Ken Burns** — slow pan/zoom over static images *(daemon)*.
- **Pixelate / mosaic** — a blocky reveal transition *(daemon, easy — similar to `decrypt` but in
  raster order)*.
- **Scanline / CRT** — a cathode-ray style reveal *(daemon)*.
- **Gradient / blur reveal** — start blurred and sharpen *(client, build on `--blur`)*.
- **Kaleidoscope** — mirror-symmetric reveal *(daemon)*.

## Features

- **Smart wallpapers** — `xwww scene` (JS scenes driven by palettes) is implemented behind the
  `scene` feature; Hyprland events and streaming are still pending. See [scene.md](scene.md)
  *(client)*.
- **Color scheme generator** — extend `palette` to emit a full theme (background, foreground,
  accent) and integrate with external themers *(client)*.
- **`xwww watch`** — an event stream (wallpaper set, output hotplug) for reactive shells instead
  of polling `query` *(daemon + client)*.
- **Per-workspace wallpapers** — a helper built on `--namespace`, compositor-specific *(client)*.
- **Config file** — a TOML config (`~/.config/xwww/config.toml`) as an alternative to env vars and
  long flag lists *(client)*.
- **Remote/URL sources** — `xwww img https://...` *(client)*.
- **Slideshow filters** — recursive directory scan, include/exclude globs, one-shot mode
  *(client)*.

## Video

- **Audio** — optionally sync audio playback (heavyweight; probably out of scope).
- **Seek/loop** — seek to a position, or loop a sub-range of a video *(daemon + client)*.
- **Hardware decode** — offload decode via VAAPI *(client)*.

## Robustness

- **Streaming video decode** — decode and compress frames incrementally instead of holding them in
  memory *(client)*.
- **Screenshot to clipboard** — pipe the screenshot into `wl-copy` *(client)*.
