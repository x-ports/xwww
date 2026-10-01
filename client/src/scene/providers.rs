//! Context providers for scenes: palette and clock.
//!
//! F1 keeps this deliberately simple: the palette is re-read at most once per second, and a
//! failed reload keeps the last good palette. Compositor events (Hyprland's event socket) and
//! system sources land in F2/F3.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::palette_source::ScenePalette;

const RELOAD_INTERVAL: Duration = Duration::from_secs(1);

/// Keeps the active palette fresh without re-reading (or re-running) the source every frame.
pub struct PaletteProvider {
    spec: Option<String>,
    palette: ScenePalette,
    last_reload: Instant,
}

impl PaletteProvider {
    /// Loads the palette immediately. Without a spec it uses the default chain (xwww palette
    /// file, then equisdots, then the neutral fallback).
    pub fn new(spec: Option<&str>) -> Result<Self, String> {
        let palette = match spec {
            Some(spec) => ScenePalette::load(spec)?,
            None => ScenePalette::load_default(),
        };
        Ok(Self {
            spec: spec.map(str::to_owned),
            palette,
            last_reload: Instant::now(),
        })
    }

    /// Returns the current palette, re-reading the source if the interval elapsed.
    pub fn current(&mut self) -> &ScenePalette {
        if self.last_reload.elapsed() >= RELOAD_INTERVAL {
            self.last_reload = Instant::now();
            if let Some(spec) = &self.spec
                && let Ok(palette) = ScenePalette::load(spec)
            {
                self.palette = palette;
            }
        }
        &self.palette
    }

    #[must_use]
    pub fn cached(&self) -> &ScenePalette {
        &self.palette
    }
}

/// Wall clock in milliseconds since the Unix epoch.
#[must_use]
pub fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64() * 1000.0)
}
