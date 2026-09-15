# Cargo features

The `client` crate exposes build-time feature flags:

| Feature | Default | Description |
|---------|---------|-------------|
| `video` | yes | Video wallpaper support via `ffmpeg-next` (requires FFmpeg dev libraries). |
| `avif`  | no | AVIF decoding via `image/avif-native` (requires `dav1d`). |
| `jxl`   | no | JPEG-XL decoding via `jxl-oxide`. |
| `all-formats` | no | Enables `avif`, `jxl` and `video` together. |

## Examples

```sh
# default build (video on)
cargo build --release

# everything
cargo build --release --all-features

# minimal build, no video/avif/jxl
cargo build --release --no-default-features
```

Note: the FFmpeg dependency (`ffmpeg-sys-next`) generates C bindings at build time, so the `video`
feature additionally requires `libclang`/bindgen and the FFmpeg headers to be present.
