<a id="top" name="top"></a>

# Cargo features

The `client` crate (`xwww`) exposes build-time feature flags. The `daemon` and
`common` crates have no features.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#features">Features</a> &middot;
  <a href="#examples">Examples</a> &middot;
  <a href="#notes">Notes</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Features](#features)
- [Examples](#examples)
- [Notes](#notes)

</details>

<a id="features" name="features"></a>

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `video` | yes | Video wallpaper support through `ffmpeg-next`. Requires FFmpeg development libraries and a C toolchain for bindgen. |
| `video-static` | no | Implies `video`, but fetches and builds FFmpeg from source and links it statically, so no FFmpeg installation is needed at build or run time. |
| `avif` | no | AVIF decoding through `image/avif-native`. Requires `dav1d`. |
| `jxl` | no | JPEG XL decoding through `jxl-oxide`. |
| `scene` | no | JavaScript scene wallpapers (`xwww scene`). Embeds QuickJS through `rquickjs` and uses `tiny-skia` as the canvas. |
| `all-formats` | no | Enables `avif`, `jxl`, `video` and `scene` together. |

<a id="examples" name="examples"></a>

## Examples

```sh
# default build (video enabled)
cargo build --release

# every optional format and the scene engine
cargo build --release --all-features

# minimal build: no video, AVIF, JXL or scenes
cargo build --release --no-default-features

# with JavaScript scenes only
cargo build --release --features scene

# release archives use an FFmpeg build independent of the host
cargo build --release --features video-static,scene
```

<a id="notes" name="notes"></a>

## Notes

- `ffmpeg-sys-next` generates C bindings at build time, so the plain `video`
  feature needs `libclang`/bindgen and the FFmpeg headers present on the
  machine. `video-static` avoids that host dependency by building FFmpeg as
  part of the crate.
- The release workflow and the published tarballs use
  `--features video-static,scene`, which is the recommended combination for
  binary distributions.
- Optional image formats are registered at decode time; a build without `avif`
  or `jxl` fails with an unrecognized-format error for those files. See the
  [project README](../README.md#troubleshooting).

<div align="center">
  <a href="#top">Back to top</a>
</div>
