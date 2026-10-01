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

/// Recolors an interleaved pixel buffer with a gradient map built from palette stops.
///
/// Every pixel's luminance is mapped onto the gradient defined by `stops` (which must be sorted
/// by ascending luminance, as produced by `ScenePalette::gradient_stops`) and blended with the
/// original color according to `strength` (`0.0` leaves the image untouched, `1.0` fully
/// replaces it). The alpha channel, when present, is never modified.
///
/// # Arguments
///
/// * `bytes` - mutable interleaved pixel buffer (`width * height * channels` bytes long).
/// * `channels` - number of bytes per pixel (3 for RGB, 4 for ARGB).
/// * `stops` - gradient stops as `[r, g, b]`, sorted by ascending luminance, length >= 1.
/// * `strength` - blend factor in `[0.0, 1.0]`.
pub fn palette_map(bytes: &mut [u8], channels: u8, stops: &[[u8; 3]], strength: f32) {
    let channels = channels as usize;
    let strength = strength.clamp(0.0, 1.0);

    if stops.is_empty() || strength <= 0.0 || channels < 3 {
        return;
    }
    debug_assert_eq!(bytes.len() % channels, 0);

    let lumas: Vec<f32> = stops.iter().map(|stop| luma(*stop)).collect();
    let first = lumas[0];
    let last = *lumas.last().unwrap();

    for pixel in bytes.chunks_exact_mut(channels) {
        let original = [pixel[0], pixel[1], pixel[2]];
        let mapped = sample_gradient(stops, &lumas, first, last, luma(original));

        for channel in 0..3 {
            pixel[channel] = mix(original[channel], mapped[channel], strength);
        }
    }
}

/// Rec. 709 relative luminance, in `[0.0, 255.0]`.
fn luma(pixel: [u8; 3]) -> f32 {
    f32::from(pixel[0]) * 0.2126 + f32::from(pixel[1]) * 0.7152 + f32::from(pixel[2]) * 0.0722
}

/// Samples the gradient at the given luminance. `lumas` must be sorted and strictly increasing.
fn sample_gradient(stops: &[[u8; 3]], lumas: &[f32], first: f32, last: f32, value: f32) -> [u8; 3] {
    if value <= first {
        return stops[0];
    }
    if value >= last {
        return stops[stops.len() - 1];
    }

    let index = lumas
        .partition_point(|&stop| stop <= value)
        .saturating_sub(1)
        .min(stops.len() - 2);
    let (low, high) = (stops[index], stops[index + 1]);
    let (low_luma, high_luma) = (lumas[index], lumas[index + 1]);
    let t = ((value - low_luma) / (high_luma - low_luma)).clamp(0.0, 1.0);

    [
        mix(low[0], high[0], t),
        mix(low[1], high[1], t),
        mix(low[2], high[2], t),
    ]
}

fn mix(from: u8, to: u8, t: f32) -> u8 {
    (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
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

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK_WHITE: &[[u8; 3]] = &[[0, 0, 0], [255, 255, 255]];

    #[test]
    fn palette_map_preserves_neutral_gray_with_black_white_stops() {
        let mut pixel = [128u8, 128, 128];
        palette_map(&mut pixel, 3, BLACK_WHITE, 1.0);
        assert_eq!(pixel, [128, 128, 128]);
    }

    #[test]
    fn palette_map_clamps_at_gradient_ends() {
        let mut pixels = [0u8, 0, 0, 255, 255, 255];
        palette_map(&mut pixels, 3, BLACK_WHITE, 1.0);
        assert_eq!(&pixels[0..3], &[0, 0, 0]);
        assert_eq!(&pixels[3..6], &[255, 255, 255]);
    }

    #[test]
    fn palette_map_preserves_alpha() {
        let mut pixel = [10u8, 20, 30, 42];
        palette_map(&mut pixel, 4, BLACK_WHITE, 1.0);
        assert_eq!(pixel[3], 42);
    }

    #[test]
    fn palette_map_strength_zero_is_a_no_op() {
        let mut pixel = [10u8, 20, 30, 42];
        palette_map(&mut pixel, 4, BLACK_WHITE, 0.0);
        assert_eq!(pixel, [10, 20, 30, 42]);
    }

    #[test]
    fn palette_map_strength_half_blends_towards_gradient() {
        let mut pixel = [100u8, 100, 100];
        palette_map(&mut pixel, 3, BLACK_WHITE, 0.5);
        assert_eq!(pixel, [100, 100, 100]);
        let mut pixel = [0u8, 0, 0, 255];
        palette_map(&mut pixel, 4, &[[255, 255, 255]], 0.5);
        assert_eq!(pixel, [128, 128, 128, 255]);
    }

    #[test]
    fn palette_map_single_stop_fills_with_it() {
        let mut pixels = [0u8, 0, 0, 12, 34, 56];
        palette_map(&mut pixels, 3, &[[10, 20, 30]], 1.0);
        assert_eq!(pixels, [10, 20, 30, 10, 20, 30]);
    }

    #[test]
    fn palette_map_empty_stops_is_a_no_op() {
        let mut pixels = [10u8, 20, 30];
        palette_map(&mut pixels, 3, &[], 1.0);
        assert_eq!(pixels, [10, 20, 30]);
    }

    #[test]
    fn palette_map_interpolates_between_stops() {
        let stops = [[0u8, 0, 0], [255, 0, 0]];
        let mut pixels = [128u8, 128, 128];
        palette_map(&mut pixels, 3, &stops, 1.0);
        assert!(pixels[0] > pixels[1]);
        assert_eq!(pixels[1], pixels[2]);
    }
}
