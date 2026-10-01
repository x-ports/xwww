# Video wallpapers

`xwww` supports using a video file as a wallpaper. The video is decoded into frames, delta-compressed,
and streamed to the daemon exactly like an animated GIF, so it reuses the existing animation
rendering pipeline.

## Usage

```sh
xwww img wallpaper.mp4
xwww img wallpaper.mp4 --resize crop --filter Lanczos3
xwww slideshow ~/Videos --interval 120   # videos are picked up too
```

Videos are detected by extension: `mp4`, `mkv`, `webm`, `mov`, `avi`, `m4v`, `mpg`, `mpeg`,
`flv`, `wmv`, `ts`, `m2ts`.

## Build requirement

Video support is implemented with [`ffmpeg-next`](https://crates.io/crates/ffmpeg-next) and is
behind the `video` cargo feature, which is **on by default**. Building it requires the FFmpeg
development libraries (libavcodec, libavformat, libavutil, libswscale) plus a C toolchain for
bindgen. Disable it with:

```sh
cargo build --no-default-features
```

## How it works

`client/src/video.rs` opens the file with `ffmpeg-next`, finds the video stream, and decodes every
frame to `RGB24` at native resolution (using `libswscale`). `compress_to_animation` then mirrors
what `imgproc::compress_frames` does for GIFs: each frame is resized to the output dimensions and
delta-compressed against the previous frame, producing an `Animation` that the daemon renders
frame by frame.

## Limitations

- Only the **first video stream** is used; audio and subtitles are ignored.
- Frames are decoded and held in memory for the duration of the request (same trade-off as animated
  GIFs).
- Frame timing is derived from the stream's average frame rate; variable-frame-rate content may
  drift slightly.
- `--blur`, `--dim` and `--map-palette` are not applied to video frames.
