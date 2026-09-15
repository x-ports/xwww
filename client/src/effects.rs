//! Client-side image effects applied to a decoded wallpaper before it is sent to the daemon.
//!
//! The effects operate directly on the raw interleaved pixel buffer (`Box<[u8]>`) that the resize
//! pipeline already produced, so they know the target dimensions and pixel format up front.

/// Applies an approximate gaussian blur to an interleaved pixel buffer.
///
/// The implementation uses three iterations of a separable box blur (a well-known approximation of
/// a gaussian blur that is both cheap and stable). Each iteration runs a horizontal pass followed
/// by a vertical pass, both using a sliding window so the cost is `O(width * height * channels)`
/// per pass regardless of the radius.
///
/// # Arguments
///
/// * `bytes` - mutable interleaved pixel buffer (`width * height * channels` bytes long).
/// * `width`, `height` - image dimensions in pixels.
/// * `channels` - number of bytes per pixel (3 for RGB, 4 for ARGB).
/// * `radius` - blur radius in pixels. `0` is a no-op.
pub fn blur(bytes: &mut [u8], width: u32, height: u32, channels: u8, radius: u32) {
    let radius = radius as usize;
    let channels = channels as usize;
    let width = width as usize;
    let height = height as usize;

    if radius == 0 || width == 0 || height == 0 {
        return;
    }
    debug_assert_eq!(bytes.len(), width * height * channels);

    let mut buf_a = bytes.to_vec();
    let mut buf_b = vec![0u8; bytes.len()];

    for _ in 0..3 {
        horizontal_pass(&buf_a, &mut buf_b, width, height, channels, radius);
        vertical_pass(&buf_b, &mut buf_a, width, height, channels, radius);
    }

    bytes.copy_from_slice(&buf_a);
}

/// Darkens the RGB channels of an interleaved pixel buffer.
///
/// The alpha channel (when present) is left untouched. `factor` should be in `[0.0, 1.0]`, where
/// `1.0` leaves the image unchanged and `0.0` turns it fully black. Values outside that range are
/// clamped.
pub fn dim(bytes: &mut [u8], channels: u8, factor: f32) {
    let channels = channels as usize;
    let factor = factor.clamp(0.0, 1.0);

    if factor >= 1.0 {
        return;
    }

    // only scale the color channels, never the alpha channel
    let n = channels.min(3);
    for pixel in bytes.chunks_exact_mut(channels) {
        for value in &mut pixel[..n] {
            *value = (*value as f32 * factor) as u8;
        }
    }
}

/// Blurs every row of `src` into `dst` using a clamped sliding window of size `2 * radius + 1`.
fn horizontal_pass(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize,
    channels: usize,
    radius: usize,
) {
    for y in 0..height {
        let base = y * width * channels;
        for c in 0..channels {
            let right = radius.min(width - 1);
            let mut sum: u64 = 0;
            for x in 0..=right {
                sum += u64::from(src[base + x * channels + c]);
            }
            let mut win = right + 1;
            for x in 0..width {
                dst[base + x * channels + c] = (sum / win as u64) as u8;

                let add = x + radius + 1;
                if add < width {
                    sum += u64::from(src[base + add * channels + c]);
                    win += 1;
                }
                if x >= radius {
                    sum -= u64::from(src[base + (x - radius) * channels + c]);
                    win -= 1;
                }
            }
        }
    }
}

/// Blurs every column of `src` into `dst` using a clamped sliding window of size `2 * radius + 1`.
fn vertical_pass(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize,
    channels: usize,
    radius: usize,
) {
    for x in 0..width {
        let col = x * channels;
        for c in 0..channels {
            let bottom = radius.min(height - 1);
            let mut sum: u64 = 0;
            for y in 0..=bottom {
                sum += u64::from(src[y * width * channels + col + c]);
            }
            let mut win = bottom + 1;
            for y in 0..height {
                dst[y * width * channels + col + c] = (sum / win as u64) as u8;

                let add = y + radius + 1;
                if add < height {
                    sum += u64::from(src[add * width * channels + col + c]);
                    win += 1;
                }
                if y >= radius {
                    sum -= u64::from(src[(y - radius) * width * channels + col + c]);
                    win -= 1;
                }
            }
        }
    }
}
