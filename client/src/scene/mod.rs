//! The scene engine: renders a JavaScript wallpaper in the client process.
//!
//! A [`SceneEngine`] owns a QuickJS runtime, its canvas and a [`PaletteProvider`]. Callers wire
//! the resulting frames to the daemon: `main.rs` sends them through the normal image IPC, while
//! the `render` subcommand writes PNGs for debugging.

pub mod canvas;
pub mod providers;
pub mod runtime;

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub use canvas::Canvas;
pub use providers::PaletteProvider;
pub use runtime::SceneRuntime;

use crate::palette_source::ScenePalette;

/// A loaded scene, ready to render frames.
pub struct SceneEngine {
    runtime: SceneRuntime,
    palette: PaletteProvider,
    setup_done: bool,
    /// Duration of the crossfade when the active palette changes (zero disables it).
    palette_fade: Duration,
    /// Digest of the palette used by the latest frame.
    last_palette: Option<u64>,
    /// When the running palette crossfade ends.
    fade_deadline: Option<Instant>,
}

fn palette_digest(palette: &ScenePalette) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    palette.slug.hash(&mut hasher);
    palette.name.hash(&mut hasher);
    palette.background.to_array().hash(&mut hasher);
    palette.foreground.to_array().hash(&mut hasher);
    for color in &palette.colors {
        color.to_array().hash(&mut hasher);
    }
    for (name, color) in &palette.roles {
        name.hash(&mut hasher);
        color.to_array().hash(&mut hasher);
    }
    hasher.finish()
}

impl SceneEngine {
    /// Loads and compiles a scene script without rendering anything.
    pub fn check(script_path: &Path) -> Result<Self, String> {
        Self::load(script_path, 64, 64, Duration::from_millis(100), None)
    }

    /// Loads a scene for an output of `width`x`height` physical pixels.
    pub fn load(
        script_path: &Path,
        width: u32,
        height: u32,
        timeout: Duration,
        palette_spec: Option<&str>,
    ) -> Result<Self, String> {
        Self::load_with_assets(script_path, width, height, timeout, palette_spec, &[])
    }

    /// Like [`SceneEngine::load`], additionally allowing image assets from `--asset` paths.
    /// The scene's own directory is always allowed.
    pub fn load_with_assets(
        script_path: &Path,
        width: u32,
        height: u32,
        timeout: Duration,
        palette_spec: Option<&str>,
        assets: &[PathBuf],
    ) -> Result<Self, String> {
        let script_path = script_path.canonicalize().map_err(|e| {
            format!("failed to resolve scene {}: {e}", script_path.display())
        })?;
        let script = std::fs::read_to_string(&script_path)
            .map_err(|e| format!("failed to read scene {}: {e}", script_path.display()))?;

        let mut allowed = Vec::new();
        if let Some(parent) = script_path.parent() {
            allowed.push(parent.to_path_buf());
        }
        for asset in assets {
            if asset.is_dir() {
                allowed.push(asset.clone());
            } else if let Some(parent) = asset.parent() {
                allowed.push(parent.to_path_buf());
            }
        }

        let runtime = SceneRuntime::new(&script, width, height, timeout, &allowed)
            .map_err(|e| format!("{}: {e}", script_path.display()))?;
        let palette = PaletteProvider::new(palette_spec)?;
        let last_palette = Some(palette_digest(palette.cached()));

        Ok(Self {
            runtime,
            palette,
            setup_done: false,
            palette_fade: Duration::ZERO,
            last_palette,
            fade_deadline: None,
        })
    }

    /// Enables a crossfade of `duration` when the active palette changes.
    #[must_use]
    pub fn with_palette_fade(mut self, duration: Duration) -> Self {
        self.palette_fade = duration;
        self
    }

    /// Renders one frame at time `t` (seconds since the scene started).
    ///
    /// When the palette changed since the previous frame and a crossfade is configured, the
    /// previous frame is blended over the new one while it fades out.
    pub fn render(&mut self, t: f64) -> Result<(), String> {
        let palette = self.palette.current().clone();
        let digest = palette_digest(&palette);
        if self.last_palette != Some(digest) {
            self.last_palette = Some(digest);
            if !self.palette_fade.is_zero() {
                self.runtime.save_snapshot();
                self.fade_deadline = Some(Instant::now() + self.palette_fade);
            }
        }

        let setup_done = &mut self.setup_done;
        self.runtime.render(t, &palette, setup_done)?;

        if let Some(deadline) = self.fade_deadline {
            let now = Instant::now();
            if now >= deadline {
                self.runtime.clear_snapshot();
                self.fade_deadline = None;
            } else {
                let total = self.palette_fade.as_secs_f32().max(f32::EPSILON);
                let remaining = deadline.duration_since(now).as_secs_f32() / total;
                self.runtime.draw_snapshot(remaining);
            }
        }
        Ok(())
    }

    pub fn with_canvas<R>(&self, f: impl FnOnce(&Canvas) -> R) -> R {
        self.runtime.with_canvas(f)
    }

    /// Whether the scene painted something since the last call; used to avoid
    /// re-sending identical frames to the daemon.
    pub fn take_dirty(&self) -> bool {
        self.runtime.take_dirty()
    }

    #[must_use]
    pub fn palette(&self) -> &ScenePalette {
        self.palette.cached()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scene_file(name: &str, source: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("xwww-scene-unit-{name}.js"));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(source.as_bytes()).unwrap();
        path
    }

    fn palette_file(name: &str, source: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("xwww-palette-unit-{name}.json"));
        std::fs::write(&path, source).unwrap();
        path
    }

    fn pixel(engine: &SceneEngine, index: usize) -> [u8; 3] {
        engine.with_canvas(|canvas| {
            let flat = canvas.to_flat(3, false, [0, 0, 0]);
            [flat[index * 3], flat[index * 3 + 1], flat[index * 3 + 2]]
        })
    }

    #[test]
    fn renders_shapes_into_the_canvas() {
        let path = scene_file(
            "shapes",
            r##"function render(t, ctx) {
                canvas.clear("#000000");
                canvas.fill("#c8283c");
                canvas.rect(0, 0, 8, 8);
            }"##,
        );
        let mut engine = SceneEngine::load(&path, 8, 8, Duration::from_millis(200), None).unwrap();
        engine.render(0.0).unwrap();
        assert_eq!(pixel(&engine, 0), [200, 40, 60]);
    }

    #[test]
    fn injected_palette_reaches_the_scene() {
        let path = scene_file(
            "palette",
            r##"function render(t, ctx) {
                canvas.clear(ctx.palette.background.hex);
                canvas.fill(ctx.palette.foreground.hex);
                canvas.circle(ctx.width / 2, ctx.height / 2, ctx.width / 3);
            }"##,
        );
        let palette = palette_file(
            "injected",
            r##"{"name":"T","slug":"t","background":"#0a141e","foreground":"#f0f0f0","base16":{"color0":"#0a141e","color1":"#c8283c","color2":"#3cc85a","color3":"#000000","color4":"#000000","color5":"#000000","color6":"#000000","color7":"#f0f0f0","color8":"#000000","color9":"#000000","color10":"#000000","color11":"#000000","color12":"#000000","color13":"#000000","color14":"#000000","color15":"#f0f0f0"}}"##,
        );
        let spec = format!("file:{}", palette.display());
        let mut engine =
            SceneEngine::load(&path, 4, 4, Duration::from_millis(200), Some(&spec)).unwrap();
        engine.render(0.0).unwrap();
        assert_eq!(pixel(&engine, 0), [10, 20, 30]);
        assert!(pixel(&engine, 2 * 4 + 2)[0] > 150);
    }

    #[test]
    fn setup_receives_the_context() {
        let path = scene_file(
            "setup-ctx",
            r##"let ready = false;
            function setup(ctx) { ready = ctx.width === 8 && ctx.height === 6; }
            function render(t, ctx) {
                canvas.clear(ready ? "#00ff00" : "#ff0000");
            }"##,
        );
        let mut engine = SceneEngine::load(&path, 8, 6, Duration::from_millis(200), None).unwrap();
        engine.render(0.0).unwrap();
        assert_eq!(pixel(&engine, 0), [0, 255, 0]);
    }

    #[test]
    fn scene_can_draw_and_remap_an_asset() {
        let asset_path = std::env::temp_dir().join("xwww-scene-unit-asset.png");
        let mut asset = image::RgbaImage::new(4, 1);
        for (index, color) in [[0u8, 0, 0], [0, 0, 0], [255, 255, 255], [255, 255, 255]]
            .iter()
            .enumerate()
        {
            asset.put_pixel(
                index as u32,
                0,
                image::Rgba([color[0], color[1], color[2], 255]),
            );
        }
        asset.save(&asset_path).unwrap();

        let path = scene_file(
            "asset",
            r##"function render(t, ctx) {
                canvas.clear("#000000");
                canvas.image("xwww-scene-unit-asset.png", 0, 0, ctx.width, ctx.height);
                canvas.remap(0, 0, ctx.width / 2, ctx.height, ["#ff0000", "#00ff00"], 1.0);
            }"##,
        );
        let mut engine = SceneEngine::load(&path, 4, 1, Duration::from_millis(400), None).unwrap();
        engine.render(0.0).unwrap();

        let flat = engine.with_canvas(|canvas| canvas.to_flat(3, false, [0, 0, 0]));
        // The right half was not remapped and stays white.
        assert_eq!(&flat[9..12], &[255, 255, 255]);
        // The left half (dark) took palette-ish colors from the gradient map.
        assert!(flat[0] > 0 || flat[1] > 0, "{flat:?}");
    }

    #[test]
    fn timeout_is_reported() {
        let path = scene_file("timeout", "function render(t, ctx) { while (true) {} }");
        let mut engine = SceneEngine::load(&path, 2, 2, Duration::from_millis(50), None).unwrap();
        let error = engine.render(0.0).unwrap_err();
        assert!(error.contains("exceeded"), "{error}");
    }

    #[test]
    fn check_rejects_invalid_scripts() {
        let path = scene_file("syntax", "function render( {");
        assert!(SceneEngine::check(&path).is_err());
    }

    fn full_palette(name: &str, background: &str) -> String {
        let mut colors = Vec::new();
        for i in 0..16 {
            let value = if i == 0 { background } else { "#101010" };
            colors.push(format!(r#""color{i}": "{value}""#));
        }
        format!(
            r##"{{"name":"{name}","slug":"{name}","background":"{background}","foreground":"#ffffff","base16":{{{}}}}}"##,
            colors.join(",")
        )
    }

    #[test]
    fn palette_change_crossfades() {
        let path = scene_file(
            "fade",
            r##"function render(t, ctx) {
                canvas.clear(ctx.palette.background.hex);
            }"##,
        );
        let palette = palette_file("fade", &full_palette("a", "#000000"));
        let spec = format!("file:{}", palette.display());
        let mut engine = SceneEngine::load(&path, 4, 4, Duration::from_millis(200), Some(&spec))
            .unwrap()
            .with_palette_fade(Duration::from_millis(600));

        engine.render(0.0).unwrap();
        assert_eq!(pixel(&engine, 0), [0, 0, 0]);

        // Switch the palette; the provider polls the source once per second.
        std::fs::write(&palette, full_palette("a", "#ffffff")).unwrap();
        std::thread::sleep(Duration::from_millis(1100));
        engine.render(0.1).unwrap();
        assert!(
            pixel(&engine, 0)[0] < 60,
            "the old frame should still be dominant right after the change"
        );

        // Halfway through the fade the frame is a blend of both palettes.
        std::thread::sleep(Duration::from_millis(300));
        engine.render(0.4).unwrap();
        let blended = pixel(&engine, 0)[0];
        assert!((60..=200).contains(&blended), "expected a blend, got {blended}");

        // After the fade the new palette is exact.
        std::thread::sleep(Duration::from_millis(400));
        engine.render(0.8).unwrap();
        assert_eq!(pixel(&engine, 0), [255, 255, 255]);
    }
}
