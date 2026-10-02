//! `xwww slideshow` — cycles through the images of a directory on a timer.
//!
//! The command reuses the exact same request pipeline as `xwww img`, building a full `Img` request
//! for each file and forwarding it to `process_xwww_args`, so every resize/filter/transition option
//! keeps working.

use std::path::PathBuf;
use std::time::Duration;

use crate::cli::{self, Random, Slideshow, Xwww};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "avif", "svg", "jxl", "qoi", "exr",
];

pub fn run(slideshow: &Slideshow) -> Result<(), String> {
    let mut images = list_images(&slideshow.dir)?;
    if images.is_empty() {
        return Err(format!(
            "no supported images found in directory: {}",
            slideshow.dir.display()
        ));
    }

    if slideshow.random {
        fastrand::shuffle(&mut images);
    }

    let namespace = slideshow.namespace.first().cloned().unwrap_or_default();
    let interval = Duration::from_secs(slideshow.interval.max(1));

    loop {
        for path in &images {
            let img = make_img(
                path.clone(),
                slideshow.transition_type.clone(),
                slideshow.resize,
                namespace.clone(),
            );
            if let Err(e) = crate::process_xwww_args(&Xwww::Img(img), &namespace) {
                eprintln!("WARNING: failed to set wallpaper {}: {e}", path.display());
            }
            std::thread::sleep(interval);
        }
    }
}

/// Sets a single random image from a directory, then exits.
pub fn run_random(random: &Random) -> Result<(), String> {
    let images = list_images(&random.dir)?;
    if images.is_empty() {
        return Err(format!(
            "no supported images found in directory: {}",
            random.dir.display()
        ));
    }

    let path = &images[fastrand::usize(..images.len())];
    let namespace = random.namespace.first().cloned().unwrap_or_default();
    let img = make_img(
        path.clone(),
        random.transition_type.clone(),
        random.resize,
        namespace.clone(),
    );
    crate::process_xwww_args(&Xwww::Img(img), &namespace)?;
    Ok(())
}

/// Collects the supported image files of a directory, in alphabetical order.
fn list_images(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut images = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("failed to read directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        {
            images.push(path);
        }
    }

    images.sort();
    Ok(images)
}

/// Builds a complete `Img` request for a single file with sensible slideshow defaults.
fn make_img(
    path: PathBuf,
    transition: cli::TransitionType,
    resize: cli::ResizeStrategy,
    namespace: String,
) -> cli::Img {
    cli::Img {
        all: false,
        image: cli::parse_image(path.to_string_lossy().as_ref()).unwrap(),
        outputs: String::new(),
        namespace: vec![namespace],
        no_cache: false,
        #[allow(deprecated)]
        no_resize: false,
        resize,
        crop_gravity: None,
        fill_color: [0, 0, 0, 255],
        filter: cli::Filter::Lanczos3,
        blur: 0,
        dim: 1.0,
        map_palette: None,
        map_strength: 1.0,
        transition_type: transition,
        transition_step: std::num::NonZeroU8::new(90).unwrap(),
        transition_duration: 1.0,
        transition_fps: 30,
        transition_angle: 0.0,
        transition_pos: cli::CliPosition {
            x: cli::CliCoord::Percent(0.5),
            y: cli::CliCoord::Percent(0.5),
        },
        invert_y: false,
        transition_bezier: (0.54, 0.0, 0.34, 0.99),
        transition_wave: (20.0, 20.0),
    }
}
