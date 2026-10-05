<a id="top" name="top"></a>

# xwww documentation

`xwww` is a Wayland wallpaper daemon forked from
[`awww`](https://codeberg.org/LGFae/awww), which is itself a fork of
[`swww`](https://github.com/LGFae/swww). It runs in the background, draws
wallpapers through the `wlr-layer-shell` protocol, and is controlled at runtime
by the `xwww` client. See the [project README](../README.md) for installation
and quick-start instructions.

<div align="center">
  <a href="../README.md">Project README</a> &middot;
  <a href="#user-guides">User guides</a> &middot;
  <a href="#reference">Reference</a> &middot;
  <a href="#project">Project</a> &middot;
  <a href="#quick-start">Quick start</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [User guides](#user-guides)
- [Reference](#reference)
- [Project](#project)
- [Quick start](#quick-start)

</details>

<a id="user-guides" name="user-guides"></a>

## User guides

| Document | Contents |
|----------|----------|
| [Commands](commands.md) | Every `xwww` subcommand and its options. |
| [Transitions](transitions.md) | All 30 transition effects and their parameters. |
| [Image effects](image-effects.md) | `--blur`, `--dim` and `--map-palette`. |
| [Palette](palette.md) | Extracting dominant colors from an image or wallpaper. |
| [Scene engine](scene.md) | JavaScript wallpapers: canvas API, palettes and runtime rules. |
| [Slideshow](slideshow.md) | Cycling wallpapers on a timer, and one-shot random images. |
| [Screenshot](screenshot.md) | Capturing the daemon's current wallpaper. |
| [Video](video.md) | Video wallpapers and their build requirements. |

<a id="reference" name="reference"></a>

## Reference

| Document | Contents |
|----------|----------|
| [Architecture](architecture.md) | Workspace layout and how the client, daemon and shared library fit together. |
| [IPC protocol](ipc-protocol.md) | The socket plus shared-memory message format. |
| [Cargo features](cargo-features.md) | Build-time feature flags and combinations. |

<a id="project" name="project"></a>

## Project

| Document | Contents |
|----------|----------|
| [Changelog](../CHANGELOG.md) | Notable changes per release. |
| [Future ideas](future-ideas.md) | A wishlist of features and effects. |
| [Example scripts](../example_scripts/README.md) | Shell scripting recipes. |
| [License](../LICENSE) | GPL-3.0. |

<a id="quick-start" name="quick-start"></a>

## Quick start

```sh
# build (video support is on by default)
cargo build --release

# start the daemon
./target/release/xwww-daemon &

# set a wallpaper
./target/release/xwww img path/to/image.png

# set a wallpaper with a transition
./target/release/xwww img path/to/image.png --transition-type glitch

# query outputs and stop the daemon
./target/release/xwww query
./target/release/xwww kill
```

<div align="center">
  <a href="#top">Back to top</a>
</div>
