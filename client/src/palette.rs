//! `xwww palette` — extracts the dominant colors of an image, or of the wallpaper the daemon is
//! currently displaying.
//!
//! The extraction pipeline is deliberately cheap: the image is downsampled to at most 128x128, the
//! pixels are bucketed into a small histogram (4 bits per channel), and the top buckets are
//! averaged to produce smooth representative colors.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use common::ipc::{Answer, BgImg, IpcSocket, RequestSend};

use crate::cli::Palette;

pub fn run(palette: &Palette) -> Result<(), String> {
    let path = match &palette.image {
        Some(p) => p.clone(),
        None => current_wallpaper(&palette.namespace, palette.output.as_deref())?,
    };

    let img = load_rgb(&path)?;
    let colors = dominant_colors(&img, palette.count);

    if palette.json {
        print_json(&colors);
    } else {
        for c in colors {
            println!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        }
    }

    Ok(())
}

/// Asks the daemon which image is currently displayed on the requested output and returns its path.
fn current_wallpaper(namespaces: &[String], output: Option<&str>) -> Result<PathBuf, String> {
    let namespace = namespaces.first().map(String::as_str).unwrap_or("");
    let socket = IpcSocket::client(namespace).map_err(|e| e.to_string())?;

    RequestSend::Query
        .send(&socket)
        .map_err(|e| e.to_string())?;
    let bytes = socket.recv().map_err(|e| e.to_string())?;
    let Answer::Info(infos) = Answer::receive(bytes) else {
        return Err("Daemon did not return Info, as expected".to_string());
    };

    let info = infos
        .iter()
        .find(|i| output.is_none_or(|o| i.name.as_ref() == o))
        .ok_or_else(|| "no such output".to_string())?;

    match &info.img {
        BgImg::Img(path) => Ok(PathBuf::from(path.as_ref())),
        BgImg::Color(color) => Err(format!(
            "the current wallpaper is a solid color (#{:02X}{:02X}{:02X}), not an image",
            color[0], color[1], color[2]
        )),
    }
}

/// Decodes an image file (raster via the `image` crate, SVG via `resvg`) into an RGB image.
fn load_rgb(path: &Path) -> Result<image::RgbImage, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read image: {e}"))?;

    if let Ok(img) = image::load_from_memory(&bytes) {
        return Ok(img.to_rgb8());
    }

    // Fall back to SVG rendering.
    let tree =
        resvg::usvg::Tree::from_data(&bytes, &resvg::usvg::Options::default()).map_err(|e| {
            format!("failed to decode image (not a supported raster format or SVG): {e}")
        })?;

    let size = tree.size();
    let scale = (256.0 / size.width()).clamp(0.0, 1.0);
    let width = (size.width() * scale).max(1.0) as u32;
    let height = (size.height() * scale).max(1.0) as u32;

    let mut buf = vec![0; (width * height * 4) as usize];
    let mut pixmap = resvg::tiny_skia::PixmapMut::from_bytes(&mut buf, width, height)
        .ok_or_else(|| "failed to allocate pixmap for SVG".to_string())?;
    resvg::render(
        &tree,
        resvg::usvg::Transform::from_scale(scale, scale),
        &mut pixmap,
    );

    let rgb: Vec<u8> = buf
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();

    image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "failed to build RGB image".to_string())
}

/// Computes the dominant colors of an image via a downsampled histogram and returns the top `count`
/// colors, ordered from most to least frequent.
fn dominant_colors(img: &image::RgbImage, count: usize) -> Vec<[u8; 3]> {
    let count = count.max(1);
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }

    let scale = (128.0 / w as f32).min(128.0 / h as f32).min(1.0);
    let (tw, th) = (
        (w as f32 * scale).max(1.0) as u32,
        (h as f32 * scale).max(1.0) as u32,
    );

    // bucket key = (r >> 4) << 8 | (g >> 4) << 4 | (b >> 4)
    let mut hist: HashMap<u32, ([u64; 3], u32)> = HashMap::new();

    for y in 0..th {
        for x in 0..tw {
            let sx = ((x as f32 / scale).min((w - 1) as f32)) as u32;
            let sy = ((y as f32 / scale).min((h - 1) as f32)) as u32;
            let p = img.get_pixel(sx, sy).0;
            let key =
                (((p[0] >> 4) as u32) << 8) | (((p[1] >> 4) as u32) << 4) | (p[2] >> 4) as u32;
            let entry = hist.entry(key).or_insert(([0; 3], 0));
            entry.0[0] += u64::from(p[0]);
            entry.0[1] += u64::from(p[1]);
            entry.0[2] += u64::from(p[2]);
            entry.1 += 1;
        }
    }

    let mut entries: Vec<(u32, ([u64; 3], u32))> = hist.into_iter().collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.1.1));
    entries.truncate(count);

    entries
        .into_iter()
        .map(|(_, (sum, n))| {
            let n = u64::from(n);
            [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8]
        })
        .collect()
}

fn print_json(colors: &[[u8; 3]]) {
    use jzon::{JsonValue, object, stringify_pretty};

    let mut arr = JsonValue::new_array();
    for c in colors {
        _ = arr.push(object! {
            hex: format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]),
            r: c[0],
            g: c[1],
            b: c[2],
        });
    }
    println!("{}", stringify_pretty(arr, 4));
}
