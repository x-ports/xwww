<a id="top" name="top"></a>

# Video wallpapers

`xwww` can use a video file as a wallpaper. The video is decoded into RGB
frames, delta-compressed and streamed to the daemon exactly like an animated
GIF, so it reuses the existing animation pipeline.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#supported-formats">Supported formats</a> &middot;
  <a href="#build-requirements">Build requirements</a> &middot;
  <a href="#how-it-works">How it works</a> &middot;
  <a href="#limitations">Limitations</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Usage](#usage)
- [Supported formats](#supported-formats)
- [Build requirements](#build-requirements)
- [How it works](#how-it-works)
- [Limitations](#limitations)

</details>

<a id="usage" name="usage"></a>

## Usage

```sh
xwww img wallpaper.mp4
xwww img wallpaper.mp4 --resize crop --filter Lanczos3
```

<a id="supported-formats" name="supported-formats"></a>

## Supported formats

Videos are detected by file extension: `mp4`, `mkv`, `webm`, `mov`, `avi`,
`m4v`, `mpg`, `mpeg`, `flv`, `wmv`, `ts` and `m2ts`. The codec support itself
comes from the FFmpeg libraries used at build time.

Note that `xwww slideshow` and `xwww random` only scan for image extensions, so
videos must be passed to `xwww img` explicitly.

<a id="build-requirements" name="build-requirements"></a>

## Build requirements

Video support is implemented with
[`ffmpeg-next`](https://crates.io/crates/ffmpeg-next) and lives behind the
`video` cargo feature, which is **on by default**. Building it requires the
FFmpeg development libraries (libavcodec, libavformat, libavutil, libswscale),
a C toolchain and `libclang` for bindgen.

To avoid depending on the host FFmpeg installation, use `video-static`, which
fetches and builds FFmpeg from source and links it statically. The published
release archives use this feature.

```sh
cargo build --release --features video-static   # self-contained FFmpeg
cargo build --release --no-default-features     # no video support
```

<a id="how-it-works" name="how-it-works"></a>

## How it works

`client/src/video.rs` opens the file with `ffmpeg-next`, picks the best video
stream and decodes every frame to `RGB24` at native resolution using
`libswscale`. The shared compression path then mirrors what GIF handling does:
each frame is resized to the output dimensions and delta-compressed against the
previous frame, producing an `Animation` that the daemon renders frame by
frame. Frame timing is taken from the stream's average frame rate.

<a id="limitations" name="limitations"></a>

## Limitations

- Only the best video stream is used; audio and subtitles are ignored.
- Frames are decoded and held in memory for the duration of the request, the
  same trade-off as animated GIFs. Very long or high-resolution videos can use
  a lot of memory.
- Frame timing comes from the average frame rate, so variable-frame-rate
  content may drift slightly.
- `--blur`, `--dim` and `--map-palette` are not applied to video frames.

<div align="center">
  <a href="#top">Back to top</a>
</div>
