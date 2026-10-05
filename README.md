<a id="top" name="top"></a>

# xwww

**An efficient animated wallpaper daemon for Wayland, controlled at runtime.**

`xwww` draws wallpapers on the `wlr-layer-shell` protocol and can be reconfigured
while it runs: images, animated GIF/WebP/APNG files, videos, solid colors,
transitions, palettes and JavaScript scenes. It is a fork of
[`awww`](https://codeberg.org/LGFae/awww) ("An Answer to your Wayland Wallpaper
Woes"), which is itself a fork of
[`swww`](https://github.com/LGFae/swww). Licensed GPL-3.0.

<div align="center">
  <a href="#overview">Overview</a> &middot;
  <a href="#previews">Previews</a> &middot;
  <a href="#features">Features</a> &middot;
  <a href="#requirements">Requirements</a> &middot;
  <a href="#installation">Installation</a> &middot;
  <a href="#quick-start">Quick start</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#configuration">Configuration</a> &middot;
  <a href="#cache">Cache</a> &middot;
  <a href="#troubleshooting">Troubleshooting</a> &middot;
  <a href="#development">Development</a> &middot;
  <a href="#documentation">Documentation</a> &middot;
  <a href="#alternatives">Alternatives</a> &middot;
  <a href="#license">License</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Overview](#overview)
- [Previews](#previews)
- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
  - [From source](#from-source)
  - [Man pages](#man-pages)
  - [Nix](#nix)
  - [Prebuilt release archives](#prebuilt-release-archives)
  - [systemd user service](#systemd-user-service)
- [Quick start](#quick-start)
- [Usage](#usage)
  - [Images and colors](#images-and-colors)
  - [Outputs and namespaces](#outputs-and-namespaces)
  - [Transitions](#transitions)
  - [Image effects](#image-effects)
  - [JavaScript scenes](#javascript-scenes)
  - [Palette](#palette)
  - [Slideshow and random](#slideshow-and-random)
  - [Screenshot](#screenshot)
  - [Video wallpapers](#video-wallpapers)
  - [Daemon control](#daemon-control)
- [Configuration](#configuration)
- [Cache](#cache)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Documentation](#documentation)
- [Alternatives](#alternatives)
- [Acknowledgments](#acknowledgments)
- [License](#license)

</details>

<a id="overview" name="overview"></a>

## Overview

`xwww` is split into two programs:

- `xwww-daemon` connects to the Wayland compositor, creates one layer surface
  per output, and owns the visible pixels.
- `xwww` is the control client. It decodes and prepares the wallpaper, then sends
  the pixels and an optional transition to the daemon over a UNIX socket.

The daemon never restarts to change a wallpaper, which makes `xwww` a good
building block for shell scripts and desktop integrations. Scheduling logic
(time-of-day wallpapers, per-workspace images, and so on) is expected to live in
scripts that call `xwww`, not inside the daemon.

Because `xwww` relies on `wlr-layer-shell`, it works on compositors that
implement that protocol (Hyprland, Sway, niri, River, Wayfire, ...) and does
**not** work on GNOME, which does not implement it.

<a id="previews" name="previews"></a>

## Previews

<div align="center">
  <img src="https://equisdots.github.io/web/previews/clips/gif/tour-v1-part02.gif" alt="xwww wallpaper preview (tour v1, part 02)" width="100%">
  <p><em>Palette-aware wallpapers. Change the color palette while a dynamic wallpaper is running and it recolors itself to match, without restarting anything.</em></p>
</div>

<div align="center">
  <img src="https://equisdots.github.io/web/previews/clips/gif/tour-v1-part05.gif" alt="xwww wallpaper preview (tour v1, part 05)" width="100%">
  <p><em>One channel for every stack. Rust, QML or shell scripts: if your desktop can communicate with xwww, the wallpaper can change.</em></p>
</div>

<a id="features" name="features"></a>

## Features

- Displays static images in the following formats:
  `avif` (optional feature), `bmp`, `dds`, `exr`, `farbfeld`, `gif`, `hdr`,
  `ico`, `jpeg`, `jpeg-xl` (optional feature), `png`, `pnm`, `qoi`, `svg`, `tga`,
  `tiff` and `webp`.
- Displays animated `gif`, `webp` and `apng` wallpapers.
- Displays videos as wallpapers (optional feature, on by default).
- Fills outputs with an arbitrary `rrggbb` color.
- Applies client-side effects before sending: Gaussian `--blur`, `--dim` and
  palette recoloring with `--map-palette`.
- Uses 30 transition effects, including wipes, circles, glitch, decrypt,
  pixelate, ripple, blinds, spiral, static, parallax, melt and shatter.
- Extracts a dominant-color palette from an image or from the current wallpaper
  with `xwww palette`.
- Cycles wallpapers with `xwww slideshow`, or picks one with `xwww random`.
- Captures the current wallpaper to a PNG with `xwww screenshot`.
- Renders procedural, palette-aware JavaScript wallpapers with `xwww scene`
  (optional feature).
- Targets individual outputs, all outputs, or independent daemon namespaces.
- Works from scripts: `xwww img -` reads an image from standard input.

<a id="requirements" name="requirements"></a>

## Requirements

Runtime:

- A compositor that implements the `wlr-layer-shell` protocol, typically
  wlroots-based.
- [`lz4`](https://github.com/lz4/lz4) for compressing animation frames.
- A running Wayland session with `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` set.

Build-time (in addition to a Rust toolchain):

- `wayland-client` development files and the
  `wayland-protocols` XML files, discoverable through `pkg-config`.
- `liblz4` development files (`pkg-config` must find `liblz4 >= 1.8`).
- A C toolchain and FFmpeg development headers when the `video` feature is
  enabled (it is by default). The `video-static` feature builds and links FFmpeg
  statically instead.
- `scdoc` to generate the man pages.

The MSRV is Rust **1.89.0** (edition 2024).

<a id="installation" name="installation"></a>

## Installation

<a id="from-source" name="from-source"></a>

### From source

```sh
git clone https://github.com/x-ports/xwww
cd xwww
cargo build --release
```

The binaries are written to `target/release/xwww` and
`target/release/xwww-daemon`. Put both on your `PATH`.

Shell completions for bash, zsh, fish and elvish are generated during the build
into the `completions/` directory.

To build a minimal binary without video or the scene engine:

```sh
cargo build --release --no-default-features
```

See [Cargo features](docs/cargo-features.md) for every flag and combination.

<a id="man-pages" name="man-pages"></a>

### Man pages

With `scdoc` installed:

```sh
./doc/gen.sh
```

The pages are written to `doc/generated`. Install them where `manpath` says, or
copy them into `<prefix>/share/man/man1`.

<a id="nix" name="nix"></a>

### Nix

The repository is also a flake. Add it to your `flake.nix`:

```nix
inputs.xwww.url = "git+https://github.com/x-ports/xwww";
```

Pass the inputs to your modules with `specialArgs` and install the package:

```nix
environment.systemPackages = [
  inputs.xwww.packages.${pkgs.stdenv.hostPlatform.system}.xwww
];
```

<a id="prebuilt-release-archives" name="prebuilt-release-archives"></a>

### Prebuilt release archives

Every tagged release publishes tarballs on the
[GitHub releases page](https://github.com/x-ports/xwww/releases) for
`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. They are built with
`--features video-static,scene` and contain both binaries, the shell
completions, the generated man pages and the systemd unit. Verify the download
with the `.sha256` file published next to each tarball.

<a id="systemd-user-service" name="systemd-user-service"></a>

### systemd user service

A service unit is available at
[`contrib/systemd/xwww-daemon.service`](contrib/systemd/xwww-daemon.service).
It runs the daemon as part of `graphical-session.target` and is the recommended
way to start it automatically.

<a id="quick-start" name="quick-start"></a>

## Quick start

Start the daemon once:

```sh
xwww-daemon &
```

Then set wallpapers from another terminal:

```sh
# static image
xwww img ~/Pictures/wallpaper.png

# animated gif with a grow transition
xwww img ~/Pictures/animation.gif \
  --transition-type grow --transition-pos 0.5,0.5 --transition-duration 2

# solid color
xwww img 0x1e1e2eff

# list outputs and the current wallpaper
xwww query

# stop the daemon
xwww kill
```

<a id="usage" name="usage"></a>

## Usage

<a id="images-and-colors" name="images-and-colors"></a>

### Images and colors

```sh
xwww img path/to/image.png
xwww img - < image.png                # read from standard input
xwww img 0xff0000ff                   # solid rrggbbaa color
xwww img image.png --resize fit --filter Lanczos3
xwww img image.png --resize no --fill-color 101010ff
```

`--resize` accepts `crop` (default), `fit`, `stretch` and `no` (center and pad
with `--fill-color`). When cropping, `--crop-gravity` selects the anchored
portion (`center`, `top-left`, `top`, `top-right`, `left`, `right`,
`bottom-left`, `bottom`, `bottom-right`). `--filter` accepts `Nearest`,
`Bilinear`, `CatmullRom`, `Mitchell` and `Lanczos3`.

<a id="outputs-and-namespaces" name="outputs-and-namespaces"></a>

### Outputs and namespaces

```sh
xwww img -o HDMI-A-1,DP-2 wallpaper.jpg   # specific outputs
xwww query                                # valid output names
xwww query --json                         # machine-readable output
xwww-daemon --namespace secondary         # start a second, independent daemon
xwww img -n secondary other.png           # talk to that daemon
xwww kill --all                           # every namespace
```

A daemon namespace is appended to the socket name, so several daemons can run
side by side on the same Wayland display. Most commands accept
`-n/--namespace` (repeatable) and `-a/--all`; run `xwww <command> --help` for
the exact option set.

<a id="transitions" name="transitions"></a>

### Transitions

```sh
xwww img image.png --transition-type center
xwww img image.png --transition-type wipe --transition-angle 30
xwww img image.png --transition-type random --transition-duration 2
```

The full list, with descriptions and shape parameters, is in
[Transitions](docs/transitions.md). Common options:

| Option | Meaning |
|--------|---------|
| `-t, --transition-type` | Effect name (default `simple`). |
| `--transition-step` | How far each pixel moves per frame (default `2` for `simple`, `90` otherwise). |
| `--transition-duration` | Duration in seconds for time-based effects (default `3`). |
| `--transition-fps` | Frame rate of the transition (default `30`). |
| `--transition-angle` | Angle for `wipe` and `wave`, in degrees. |
| `--transition-pos` | Center for circular/radial effects (percent, pixels or a keyword such as `center`). |
| `--transition-bezier` | Bezier curve for `fade` and circle effects. |
| `--transition-wave` | Wave size for `wave`. |
| `--invert-y` | Flips the y coordinate of `--transition-pos`. |

<a id="image-effects" name="image-effects"></a>

### Image effects

```sh
xwww img image.png --blur 20
xwww img image.png --dim 0.5
xwww img image.png --map-palette xwww --map-strength 0.7
```

`--map-palette` recolors the image through a gradient built from a palette
source: `xwww[:<path>]`, `equisdots[:<slug>]`, `file:<path>` or `command:<cmd>`.
Effects apply to static images only. See
[Image effects](docs/image-effects.md).

<a id="javascript-scenes" name="javascript-scenes"></a>

### JavaScript scenes

Requires a build with the `scene` feature:

```sh
xwww scene check ~/scenes/clock.js
xwww scene render ~/scenes/clock.js -o clock.png --size 2560x1440
xwww scene run ~/scenes/clock.js --fps 10 --palette xwww
```

A scene is a single JavaScript file with optional `setup(ctx)` and
`render(t, ctx)` functions that draw on a small canvas API and read the active
color palette. See the [Scene engine](docs/scene.md) guide.

<a id="palette" name="palette"></a>

### Palette

```sh
xwww palette path/to/image.png --count 5
xwww palette                 # colors of the current wallpaper
xwww palette --json
```

Prints the dominant colors of an image, useful for theming the rest of the
desktop around the wallpaper. See [Palette](docs/palette.md).

<a id="slideshow-and-random" name="slideshow-and-random"></a>

### Slideshow and random

```sh
xwww slideshow ~/Pictures/wallpapers --interval 300 --random
xwww random ~/Pictures/wallpapers --transition-type glitch
```

`slideshow` cycles through a directory forever; `random` sets one random image
and exits. See [Slideshow](docs/slideshow.md).

<a id="screenshot" name="screenshot"></a>

### Screenshot

```sh
xwww screenshot wallpaper.png
xwww screenshot -o HDMI-A-1 wallpaper.png
```

Captures the wallpaper currently displayed by the daemon (not the whole
desktop). See [Screenshot](docs/screenshot.md).

<a id="video-wallpapers" name="video-wallpapers"></a>

### Video wallpapers

```sh
xwww img wallpaper.mp4
```

Videos are decoded into frames and streamed to the daemon like an animated GIF.
Video support is a default feature; see [Video](docs/video.md) for build
requirements and limitations.

<a id="daemon-control" name="daemon-control"></a>

### Daemon control

| Command | Effect |
|---------|--------|
| `xwww pause` | Freezes animations on the last rendered frame. |
| `xwww unpause` | Resumes animations. |
| `xwww toggle` | Flips the paused state. |
| `xwww restore` | Reloads the cached wallpaper per output. |
| `xwww clear [color]` | Fills outputs with a color. |
| `xwww clear-cache` | Deletes the whole cache directory. |
| `xwww kill` | Stops the daemon and removes its socket. |

<a id="configuration" name="configuration"></a>

## Configuration

`xwww` has no configuration file; behavior is controlled through command-line
flags and environment variables. The flags listed below also accept an
environment variable as a fallback:

| Variable | Equivalent flag |
|----------|-----------------|
| `XWWW_TRANSITION` | `--transition-type` |
| `XWWW_TRANSITION_STEP` | `--transition-step` |
| `XWWW_TRANSITION_DURATION` | `--transition-duration` |
| `XWWW_TRANSITION_FPS` | `--transition-fps` |
| `XWWW_TRANSITION_ANGLE` | `--transition-angle` |
| `XWWW_TRANSITION_POS` | `--transition-pos` |
| `XWWW_TRANSITION_BEZIER` | `--transition-bezier` |
| `XWWW_TRANSITION_WAVE` | `--transition-wave` |
| `INVERT_Y` | `--invert-y` |
| `XWWW_PALETTE_FADE` | `xwww scene run --palette-fade` |

The daemon also accepts `-f/--format`, `-l/--layer`, `-n/--namespace`,
`--no-cache` and `-q/--quiet`; run `xwww-daemon --help` for details.

<a id="cache" name="cache"></a>

## Cache

The client remembers the last wallpaper sent to each output under
`$XDG_CACHE_HOME/xwww/<version>` (or `~/.cache/xwww/<version>`), together with
the resize/filter settings used, and preprocessed animation frames. When an
output is (re)connected, the daemon reloads that cached wallpaper
automatically; `xwww restore` does it on demand. Old versions are cleaned up
when the cache is updated.

Pass `--no-cache` to `xwww img` to skip updating the cache, or to
`xwww-daemon` to disable cache loading on startup. Run `xwww clear-cache` to
delete everything.

<a id="troubleshooting" name="troubleshooting"></a>

## Troubleshooting

**The daemon starts but the client cannot connect.**
Check that `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR` are set in the environment
that runs both programs, and that the compositor implements `wlr-layer-shell`.
The socket lives at
`$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY-xwww-daemon[.<namespace>].sock`.

**"the scene feature is not enabled".**
`xwww scene` is compiled in only with the `scene` feature:
`cargo build --release --features scene`. Release archives already include it.

**AVIF or JPEG XL images fail to load.**
Those decoders are opt-in. Rebuild with `--features avif` or `--features jxl`
(`--features all-formats` enables every optional format). AVIF also needs
`dav1d` installed.

**High CPU usage while caching an animated wallpaper.**
Caching resizes every frame, which is expensive on very large animations.
Resize the source first (for example with
[`gifsicle`](https://github.com/kohler/gifsicle)) and the daemon will cache and
play it much more cheaply.

**The wallpaper is blank on one output after a change.**
Run `xwww query` to confirm the daemon sees the output, then `xwww restore` to
reload the cached image for it.

**Wayland protocol error mentioning `wl_output` version 4.**
The compositor must expose version 4 or newer of `wl_output`.

**Transitions look abrupt on high refresh rate monitors.**
`--transition-fps` defaults to `30`; raise it to match the monitor. Frame rate
does not affect `--transition-step`, which controls how far each pixel moves per
frame.

<a id="development" name="development"></a>

## Development

```sh
cargo fmt --all
cargo clippy --workspace --locked --tests
cargo test --workspace
```

The integration tests in `tests/integration.rs` drive a live daemon and are
ignored by default. Run them inside a Wayland session with:

```sh
cargo test --workspace -- --include-ignored
```

Documentation and man pages are spell-checked with
[`codespell`](https://github.com/codespell-project/codespell) and
[`typos`](https://github.com/crate-ci/typos); see `tests/spell_check.rs`.
`./doc/gen.sh` regenerates the man pages and requires `scdoc`.

<a id="documentation" name="documentation"></a>

## Documentation

Full documentation lives in [`docs/`](docs/README.md):

| Document | Contents |
|----------|----------|
| [Architecture](docs/architecture.md) | Crate layout and how the client and daemon interact. |
| [Commands](docs/commands.md) | Reference for every `xwww` subcommand. |
| [Transitions](docs/transitions.md) | All transition effects. |
| [Image effects](docs/image-effects.md) | `--blur`, `--dim` and `--map-palette`. |
| [Palette](docs/palette.md) | Dominant-color extraction. |
| [Scene engine](docs/scene.md) | JavaScript scenes, palettes and the canvas API. |
| [Slideshow](docs/slideshow.md) | Cycling wallpapers on a timer. |
| [Screenshot](docs/screenshot.md) | Capturing the current wallpaper. |
| [Video](docs/video.md) | Video wallpapers. |
| [IPC protocol](docs/ipc-protocol.md) | Socket and shared-memory message format. |
| [Cargo features](docs/cargo-features.md) | Build-time feature flags. |
| [Future ideas](docs/future-ideas.md) | Planned and proposed work. |

<a id="alternatives" name="alternatives"></a>

## Alternatives

`xwww` is not the smallest wallpaper program around. If you want something
simpler, see the
[awesome-wayland list of wallpaper programs](https://github.com/natpen/awesome-wayland#wallpaper).
In particular:

- [`wbg`](https://codeberg.org/dnkl/wbg) - probably the simplest of them all;
  a good fit for a single static image.
- [`swaybg`](https://github.com/swaywm/swaybg) - made by the wlroots
  developers.
- [`mpvpaper`](https://github.com/GhostNaN/mpvpaper) - if you want videos as
  wallpapers.
- [`swww`](https://github.com/LGFae/swww) and
  [`awww`](https://codeberg.org/LGFae/awww) - the upstream projects this fork
  is based on.
- [`kitty`](https://sw.kovidgoyal.net/kitty/) - use its
  [panel](https://sw.kovidgoyal.net/kitty/kittens/panel/) kitten to show an
  arbitrary TUI program as the wallpaper.

<a id="acknowledgments" name="acknowledgments"></a>

## Acknowledgments

Thanks to everyone involved in the [Smithay](https://github.com/Smithay)
project. The first versions of this program were adapted from the
[layer shell example in the client-toolkit](https://github.com/Smithay/client-toolkit/blob/master/examples/layer_shell.rs).
Thanks as well to the `swww` and `awww` authors and contributors, whose work
this fork builds on.

<a id="license" name="license"></a>

## License

GPL-3.0. See [LICENSE](LICENSE).

<div align="center">
  <a href="#top">Back to top</a>
</div>
