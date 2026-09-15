//! `xwww` video support.
//!
//! Decodes a video file into RGB frames using `ffmpeg-next`, so that `xwww img` can treat a video
//! exactly like an animated GIF: the frames are resized, delta-compressed, and streamed to the
//! daemon through the regular animation pipeline.
//!
//! This module is compiled only when the `video` feature is enabled (it is on by default).

use std::path::Path;

use common::ipc::Nanos;

use ffmpeg_next::Rational;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{Context as Scaler, Flags};

/// The list of file extensions treated as video.
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "avi", "m4v", "mpg", "mpeg", "flv", "wmv", "ts", "m2ts",
];

pub struct DecodedVideo {
    pub width: u32,
    pub height: u32,
    /// RGB (3 bytes per pixel) frames paired with their display duration in nanoseconds.
    pub frames: Vec<(Box<[u8]>, Nanos)>,
}

/// Returns whether the given path has a video file extension.
pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| VIDEO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Decodes a video file into RGB frames.
pub fn decode(path: &Path) -> Result<DecodedVideo, String> {
    ffmpeg_next::init().map_err(|e| format!("failed to initialize ffmpeg: {e}"))?;

    let mut ictx =
        ffmpeg_next::format::input(path).map_err(|e| format!("failed to open video: {e}"))?;
    let stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or_else(|| "no video stream found".to_string())?;
    let stream_index = stream.index();

    let interval_nanos = frame_interval_nanos(&stream.avg_frame_rate());
    let nanos = Nanos::from_nanos(interval_nanos);

    let context = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|e| format!("failed to create codec context: {e}"))?;
    let mut decoder = context
        .decoder()
        .video()
        .map_err(|e| format!("failed to create video decoder: {e}"))?;

    let (width, height) = (decoder.width(), decoder.height());
    let src_fmt = decoder.format();

    let mut scaler = Scaler::get(
        src_fmt,
        width,
        height,
        Pixel::RGB24,
        width,
        height,
        Flags::BILINEAR,
    )
    .map_err(|e| format!("failed to create scaler: {e}"))?;

    let mut rgb = VideoFrame::empty();
    let mut frames: Vec<(Box<[u8]>, Nanos)> = Vec::new();

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }
        decoder
            .send_packet(&packet)
            .map_err(|e| format!("failed to send packet: {e}"))?;

        let mut decoded = VideoFrame::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            push_frame(&mut frames, &mut scaler, &decoded, &mut rgb, nanos)?;
            decoded = VideoFrame::empty();
        }
    }

    // flush any frames still buffered in the decoder
    decoder
        .send_eof()
        .map_err(|e| format!("failed to flush decoder: {e}"))?;
    let mut decoded = VideoFrame::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        push_frame(&mut frames, &mut scaler, &decoded, &mut rgb, nanos)?;
        decoded = VideoFrame::empty();
    }

    if frames.is_empty() {
        return Err("video contained no decodable frames".to_string());
    }

    Ok(DecodedVideo {
        width,
        height,
        frames,
    })
}

/// Scales a decoded frame to RGB24 and appends it to `frames`.
fn push_frame(
    frames: &mut Vec<(Box<[u8]>, Nanos)>,
    scaler: &mut Scaler,
    decoded: &VideoFrame,
    rgb: &mut VideoFrame,
    nanos: Nanos,
) -> Result<(), String> {
    scaler
        .run(decoded, rgb)
        .map_err(|e| format!("failed to scale frame: {e}"))?;

    let plane = rgb.data(0);
    let stride = rgb.stride(0);
    let h = rgb.height() as usize;
    let row_bytes = rgb.width() as usize * 3;

    let mut data = Vec::with_capacity(row_bytes * h);
    for y in 0..h {
        data.extend_from_slice(&plane[y * stride..y * stride + row_bytes]);
    }

    frames.push((data.into_boxed_slice(), nanos));
    Ok(())
}

/// Computes the duration of a single frame from a frame-rate rational, defaulting to 30 fps when
/// the stream does not advertise a valid rate.
fn frame_interval_nanos(rate: &Rational) -> u64 {
    let (num, den) = (rate.numerator(), rate.denominator());
    if num <= 0 || den <= 0 {
        return 33_333_333; // ~30 fps
    }
    (den as u64 * 1_000_000_000 / num as u64).max(1)
}

/// Resizes every decoded frame to `dim`, delta-compresses them into an animation, and returns the
/// first frame together with that animation. This mirrors what `imgproc::compress_frames` does for
/// animated GIFs, so the daemon renders videos exactly like any other animation.
pub fn compress_to_animation(
    video: &DecodedVideo,
    dim: (u32, u32),
    resize: crate::cli::ResizeStrategy,
    filter: crate::cli::Filter,
    fill_color: [u8; 4],
    pixel_format: common::ipc::PixelFormat,
) -> Result<(Box<[u8]>, common::ipc::Animation), String> {
    use common::compression::{BitPack, Compressor};
    use common::ipc::Nanos;

    let filter_type = crate::imgproc::make_filter(filter);
    let mut compressor = Compressor::new();

    let mut first: Option<Box<[u8]>> = None;
    let mut prev: Option<Box<[u8]>> = None;
    let mut first_nanos = Nanos::from_nanos(0);
    let mut deltas: Vec<(BitPack, Nanos)> = Vec::new();

    for (data, nanos) in &video.frames {
        let img =
            crate::imgproc::Image::from_rgb(video.width, video.height, data.clone(), pixel_format);
        let resized = match resize {
            crate::cli::ResizeStrategy::No => crate::imgproc::img_pad(&img, dim, fill_color),
            crate::cli::ResizeStrategy::Crop => {
                crate::imgproc::img_resize_crop(&img, dim, filter_type, None)?
            }
            crate::cli::ResizeStrategy::Fit => {
                crate::imgproc::img_resize_fit(&img, dim, filter_type, fill_color)?
            }
            crate::cli::ResizeStrategy::Stretch => {
                crate::imgproc::img_resize_stretch(&img, dim, filter_type)?
            }
        };

        if first.is_none() {
            first = Some(resized.clone());
            first_nanos = *nanos;
        } else if let Some(p) = prev.as_ref() {
            match compressor.compress(p, &resized, pixel_format) {
                Some(bp) => deltas.push((bp, *nanos)),
                None => match deltas.last_mut() {
                    Some(last) => last.1 += *nanos,
                    None => first_nanos += *nanos,
                },
            }
        }
        prev = Some(resized);
    }

    // wrap-around: delta from the last frame back to the first
    if let (Some(p), Some(f)) = (prev.as_ref(), first.as_ref()) {
        match compressor.compress(p, f, pixel_format) {
            Some(bp) => deltas.push((bp, first_nanos)),
            None => {
                if let Some(last) = deltas.last_mut() {
                    last.1 += first_nanos;
                }
            }
        }
    }

    let animation = common::ipc::Animation {
        animation: deltas.into_boxed_slice(),
    };
    let first_frame = first.ok_or_else(|| "no frames decoded".to_string())?;

    Ok((first_frame, animation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_interval_is_sane() {
        // 30 fps
        let rate = Rational::new(30, 1);
        let nanos = frame_interval_nanos(&rate);
        assert_eq!(nanos, 33_333_333);
    }

    /// Decodes a test video if one is present. Skipped silently otherwise.
    #[test]
    fn decodes_test_video() {
        let path = std::path::Path::new("/tmp/opencode/test.mp4");
        if !path.exists() {
            return;
        }
        let video = decode(path).expect("failed to decode test video");
        assert!(video.width > 0 && video.height > 0);
        assert!(!video.frames.is_empty());

        let expected = (video.width * video.height * 3) as usize;
        for (data, _) in &video.frames {
            assert_eq!(data.len(), expected, "frame has unexpected size");
        }
    }
}
