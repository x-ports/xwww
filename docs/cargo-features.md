# Cargo features

The `client` crate exposes build-time feature flags:

| Feature | Default | Description |
|---------|---------|-------------|
| `video` | yes | Video wallpaper support via `ffmpeg-next` (requires FFmpeg dev libraries). |
| `video-static` | no | Like `video`, but compiles and statically links FFmpeg from source (no runtime FFmpeg dependency). |
| `avif`  | no | AVIF decoding via `image/avif-native` (requires `dav1d`). |
| `jxl`   | no | JPEG-XL decoding via `jxl-oxide`. |
| `scene` | no | JavaScript scene wallpapers (`xwww scene`): embeds QuickJS (`rquickjs`) and uses `tiny-skia`. |
| `all-formats` | no | Enables `avif`, `jxl`, `video` and `scene` together. |

## Examples

```sh
# default build (video on)
cargo build --release

# everything
cargo build --release --all-features

# minimal build, no video/avif/jxl
cargo build --release --no-default-features

# with JavaScript scenes
cargo build --release --features scene
```

Note: the FFmpeg dependency (`ffmpeg-sys-next`) generates C bindings at build time, so the `video`
feature additionally requires `libclang`/bindgen and the FFmpeg headers to be present. The
`video-static` feature instead fetches and builds FFmpeg from source and links it statically, so no
FFmpeg installation is needed at build or runtime (this is what the release binaries use).
