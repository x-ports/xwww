<a id="top" name="top"></a>

# Future ideas

A running wishlist of features and effects that could fit `xwww`. Items marked
*(client)* can be implemented entirely in the client; items marked *(daemon)*
require changes to the daemon and the IPC protocol.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#effects">Effects</a> &middot;
  <a href="#features">Features</a> &middot;
  <a href="#video">Video</a> &middot;
  <a href="#robustness">Robustness</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Effects](#effects)
- [Features](#features)
- [Video](#video)
- [Robustness](#robustness)

</details>

<a id="effects" name="effects"></a>

## Effects

- **Ken Burns** - slow pan and zoom over static images *(daemon)*.
- **Scanline / CRT** - a cathode-ray style reveal *(daemon)*.
- **Gradient / blur reveal** - start blurred and sharpen, reusing `--blur`
  *(client)*.
- **Kaleidoscope** - mirror-symmetric reveal *(daemon)*.

<a id="features" name="features"></a>

## Features

- **Compositor events for scenes** - a Hyprland event provider (workspaces,
  active window, monitors) plus event streaming. The clock and palette
  providers are already implemented; see [Scene engine](scene.md) *(client)*.
- **Color scheme generator** - extend `palette` to emit a full theme
  (background, foreground, accent) and integrate with external themers
  *(client)*.
- **`xwww watch`** - an event stream (wallpaper set, output hotplug) for
  reactive shells instead of polling `query` *(daemon + client)*.
- **Per-workspace wallpapers** - a helper built on `--namespace`,
  compositor-specific *(client)*.
- **Config file** - a TOML file (`~/.config/xwww/config.toml`) as an
  alternative to environment variables and long flag lists *(client)*.
- **Remote and URL sources** - `xwww img https://...` *(client)*.
- **Slideshow filters** - recursive scanning, include/exclude globs and a
  one-shot mode *(client)*.
- **Scene screenshots cache** - persist the last scene frame on exit so the
  lock screen keeps showing it *(client)*.

<a id="video" name="video"></a>

## Video

- **Audio** - optionally sync audio playback (heavyweight; probably out of
  scope).
- **Seek/loop** - seek to a position, or loop a sub-range of a video
  *(daemon + client)*.
- **Hardware decode** - offload decoding through VAAPI *(client)*.
- **Streaming decode** - decode and compress frames incrementally instead of
  holding them all in memory *(client)*.

<a id="robustness" name="robustness"></a>

## Robustness

- **Screenshot to clipboard** - pipe the screenshot into `wl-copy` *(client)*.
- **Automated end-to-end coverage** - exercise every option with different
  output scaling and layouts *(tests)*.
- **Compiled integration tests without a session** - a headless Wayland
  compositor in CI to run the ignored tests on every commit *(tests)*.

<div align="center">
  <a href="#top">Back to top</a>
</div>
