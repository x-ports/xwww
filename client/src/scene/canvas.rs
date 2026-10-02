//! A small canvas API on top of `tiny-skia`, used by scene scripts.
//!
//! The canvas keeps a paint state (fill color, stroke, gradients, opacity), a transform stack and
//! a current path, mirroring the subset of the HTML canvas 2D API that makes sense for wallpapers.
//! Pixel output is composited over an opaque background so scenes can be handed to the daemon as
//! any `PixelFormat` without alpha ambiguity.

use std::collections::HashMap;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Arc, OnceLock};

use resvg::usvg::{self, fontdb};
use tiny_skia::{
    Color, FillRule, GradientStop, IntSize, LinearGradient, Paint, Path, PathBuilder, Pixmap,
    PixmapPaint, Point, PremultipliedColorU8, RadialGradient, Shader, SpreadMode, Stroke,
    Transform,
};

/// Cache key for scaled assets: canonical path, target size and optional tint.
type ScaledKey = (PathBuf, u32, u32, Option<[u8; 4]>);

/// Cache key for a rendered text run (the effective alpha is baked into the color).
#[derive(Clone, PartialEq, Eq, Hash)]
struct GlyphKey {
    text: String,
    size_bits: u32,
    family: String,
    anchor: String,
    bold: bool,
    color_rgba: [u8; 4],
}

/// A cached text run and where it lands relative to the caller's anchor point.
struct Glyph {
    pixmap: Pixmap,
    offset_x: f32,
    offset_y: f32,
}

/// A drawing surface for a single output.
pub struct Canvas {
    pixmap: Pixmap,
    fill: Option<[u8; 4]>,
    stroke: Option<([u8; 4], f32)>,
    shader: Option<Shader<'static>>,
    alpha: f32,
    transform: Transform,
    stack: Vec<Transform>,
    path: PathBuilder,
    /// Decoded assets, keyed by canonical path.
    assets: HashMap<PathBuf, image::RgbaImage>,
    /// Assets already scaled to a draw size (the tint is part of the key).
    scaled: HashMap<ScaledKey, Pixmap>,
    /// Rendered text runs, so repeated strings (glyph rain, cards) are laid out once.
    glyphs: HashMap<GlyphKey, Glyph>,
    /// Directories from which scenes may load images (canonicalized).
    allowed: Vec<PathBuf>,
    /// System font database, shared by every canvas and loaded once per process.
    fonts: Arc<fontdb::Database>,
    /// Previous frame (start of the palette crossfade).
    fade_from: Option<Pixmap>,
    /// New frame (end of the palette crossfade), restored when the fade ends.
    fade_to: Option<Pixmap>,
    /// Known to be fully opaque (last `clear` used alpha 255); enables the fast output path.
    opaque: bool,
    /// Set by every operation that touches pixels; cleared by [`Canvas::take_dirty`].
    dirty: bool,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let pixmap = Pixmap::new(width, height)
            .ok_or_else(|| format!("invalid canvas size {width}x{height}"))?;
        Ok(Self {
            pixmap,
            fill: Some([255, 255, 255, 255]),
            stroke: None,
            shader: None,
            alpha: 1.0,
            transform: Transform::identity(),
            stack: Vec::new(),
            path: PathBuilder::new(),
            assets: HashMap::new(),
            scaled: HashMap::new(),
            glyphs: HashMap::new(),
            allowed: Vec::new(),
            fonts: system_fonts(),
            fade_from: None,
            fade_to: None,
            opaque: false,
            dirty: false,
        })
    }

    /// Returns whether anything was drawn since the last call, resetting the flag.
    ///
    /// The frame loop uses it to skip re-sending identical frames to the daemon.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Stores a copy of the current surface as the start of the crossfade.
    pub fn save_fade_from(&mut self) {
        self.fade_from = Some(self.pixmap.clone());
    }

    /// Stores a copy of the current surface as the end of the crossfade.
    pub fn save_fade_to(&mut self) {
        self.fade_to = Some(self.pixmap.clone());
    }

    /// Rebuilds the surface as the new frame with the old one composited on top at `alpha`
    /// (`1.0` fully visible, `0.0` invisible). Used to fade out the previous palette frame.
    /// No-op without both saved frames.
    pub fn draw_fade(&mut self, alpha: f32) {
        let (Some(from), Some(to)) = (self.fade_from.as_ref(), self.fade_to.as_ref()) else {
            return;
        };
        self.pixmap.data_mut().copy_from_slice(to.data());
        let paint = PixmapPaint {
            opacity: alpha.clamp(0.0, 1.0),
            ..PixmapPaint::default()
        };
        self.pixmap
            .draw_pixmap(0, 0, from.as_ref(), &paint, Transform::identity(), None);
        self.dirty = true;
    }

    /// Restores the clean new frame and drops both saved frames.
    pub fn clear_fade(&mut self) {
        if let Some(to) = &self.fade_to {
            self.pixmap.data_mut().copy_from_slice(to.data());
            self.dirty = true;
        }
        self.fade_from = None;
        self.fade_to = None;
    }

    /// Allows scenes to load images from `dir` (and only from it).
    pub fn allow_asset_dir(&mut self, dir: &FsPath) {
        if let Ok(dir) = dir.canonicalize()
            && !self.allowed.contains(&dir)
        {
            self.allowed.push(dir);
        }
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        self.pixmap
            .fill(Color::from_rgba8(color[0], color[1], color[2], color[3]));
        self.opaque = color[3] == 255;
        self.dirty = true;
    }

    pub fn fill(&mut self, color: Option<[u8; 4]>) {
        self.fill = color;
        if color.is_some() {
            self.shader = None;
        }
    }

    pub fn stroke(&mut self, color: Option<([u8; 4], f32)>) {
        self.stroke = color;
    }

    /// Opacity multiplier in `[0, 1]` applied to every paint from now on.
    pub fn alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, width, height) {
            self.draw_rect(rect);
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, radius: f32) {
        if radius <= 0.0 || !radius.is_finite() {
            return;
        }
        let mut builder = PathBuilder::new();
        builder.push_circle(cx, cy, radius);
        self.draw_builder(builder);
    }

    pub fn round_rect(&mut self, x: f32, y: f32, width: f32, height: f32, radius: f32) {
        let Some(rect) = tiny_skia::Rect::from_xywh(x, y, width, height) else {
            return;
        };
        if radius <= 0.0 {
            self.draw_rect(rect);
            return;
        }

        let r = radius.min(width / 2.0).min(height / 2.0).max(0.0);
        let k = r * 0.552_284_8;
        let (l, t, right, bottom) = (rect.left(), rect.top(), rect.right(), rect.bottom());

        let mut builder = PathBuilder::new();
        builder.move_to(l + r, t);
        builder.line_to(right - r, t);
        builder.cubic_to(right - r + k, t, right, t + r - k, right, t + r);
        builder.line_to(right, bottom - r);
        builder.cubic_to(
            right,
            bottom - r + k,
            right - r + k,
            bottom,
            right - r,
            bottom,
        );
        builder.line_to(l + r, bottom);
        builder.cubic_to(l + r - k, bottom, l, bottom - r + k, l, bottom - r);
        builder.line_to(l, t + r);
        builder.cubic_to(l, t + r - k, l + r - k, t, l + r, t);
        builder.close();
        self.draw_builder(builder);
    }

    pub fn begin_path(&mut self) {
        self.path = PathBuilder::new();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(x, y);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(x, y);
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path.quad_to(cx, cy, x, y);
    }

    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.path.cubic_to(c1x, c1y, c2x, c2y, x, y);
    }

    pub fn close_path(&mut self) {
        self.path.close();
    }

    pub fn fill_path(&mut self) {
        let builder = std::mem::replace(&mut self.path, PathBuilder::new());
        self.draw_builder(builder);
    }

    pub fn stroke_path(&mut self) {
        let builder = std::mem::replace(&mut self.path, PathBuilder::new());
        if let Some(path) = builder.finish() {
            self.stroke_only(&path);
        }
    }

    pub fn linear_gradient(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, colors: &[[u8; 4]]) {
        let Some(shader) = LinearGradient::new(
            Point::from_xy(x0, y0),
            Point::from_xy(x1, y1),
            self.gradient_stops(colors),
            SpreadMode::Pad,
            Transform::identity(),
        ) else {
            return;
        };
        self.shader = Some(shader);
    }

    pub fn radial_gradient(&mut self, cx: f32, cy: f32, radius: f32, colors: &[[u8; 4]]) {
        let Some(shader) = RadialGradient::new(
            Point::from_xy(cx, cy),
            0.0,
            Point::from_xy(cx, cy),
            radius.max(0.0),
            self.gradient_stops(colors),
            SpreadMode::Pad,
            Transform::identity(),
        ) else {
            return;
        };
        self.shader = Some(shader);
    }

    pub fn push(&mut self) {
        self.stack.push(self.transform);
    }

    pub fn pop(&mut self) {
        if let Some(transform) = self.stack.pop() {
            self.transform = transform;
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.transform = self.transform.pre_concat(Transform::from_translate(x, y));
    }

    pub fn rotate(&mut self, degrees: f32) {
        let radians = degrees.to_radians();
        self.transform = self.transform.pre_concat(Transform::from_rotate(radians));
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.transform = self.transform.pre_concat(Transform::from_scale(x, y));
    }

    /// Draws `text` with its baseline at `x, y`, honoring the transform stack.
    ///
    /// The color is passed explicitly. `family` is a font family or a generic name
    /// (`sans-serif`, `monospace`, ...), `anchor` selects the horizontal alignment
    /// (`start`, `middle` or `end`), and `bold` picks the bold face when available.
    /// Text is rasterized through `resvg`/`usvg` with the process-wide font database.
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: [u8; 4],
        family: &str,
        anchor: &str,
        bold: bool,
    ) -> Result<(), String> {
        if text.is_empty() || !size.is_finite() || size <= 0.0 {
            return Ok(());
        }
        let alpha = (f32::from(color[3]) * self.alpha).round().clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            return Ok(());
        }

        let anchor = match anchor {
            "middle" => "middle",
            "end" => "end",
            _ => "start",
        };
        let family = if family.trim().is_empty() {
            "sans-serif"
        } else {
            family
        };
        let key = GlyphKey {
            text: text.to_string(),
            size_bits: size.to_bits(),
            family: family.to_string(),
            anchor: anchor.to_string(),
            bold,
            color_rgba: [color[0], color[1], color[2], 255],
        };

        if !self.glyphs.contains_key(&key) {
            if self.glyphs.len() > 2048 {
                self.glyphs.clear();
            }
            let (width, height) = (self.pixmap.width(), self.pixmap.height());
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\">\
                 <text x=\"0\" y=\"0\" font-family=\"{family}\" font-size=\"{size}\" \
                 font-weight=\"{weight}\" text-anchor=\"{anchor}\" fill=\"#{red:02x}{green:02x}{blue:02x}\">\
                 {content}</text></svg>",
                family = escape_xml(family),
                size = size,
                weight = if bold { "bold" } else { "normal" },
                red = color[0],
                green = color[1],
                blue = color[2],
                content = escape_xml(text),
            );

            let options = usvg::Options {
                fontdb: self.fonts.clone(),
                ..usvg::Options::default()
            };
            let tree = usvg::Tree::from_str(&svg, &options)
                .map_err(|e| format!("failed to lay out text: {e}"))?;
            let Some(node) = tree.root().children().first() else {
                return Ok(());
            };
            let Some(bbox) = node.abs_layer_bounding_box() else {
                return Ok(());
            };
            let glyph_width = bbox.width().ceil().max(1.0) as u32;
            let glyph_height = bbox.height().ceil().max(1.0) as u32;
            let Some(mut pixmap) = Pixmap::new(glyph_width, glyph_height) else {
                return Err("invalid text size".to_string());
            };
            resvg::render_node(node, Transform::identity(), &mut pixmap.as_mut());

            self.glyphs.insert(
                key.clone(),
                Glyph {
                    pixmap,
                    offset_x: bbox.x(),
                    offset_y: bbox.y(),
                },
            );
        }

        let Some(glyph) = self.glyphs.get(&key) else {
            return Ok(());
        };
        let paint = PixmapPaint {
            opacity: f32::from(alpha) / 255.0,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(
            (x + glyph.offset_x).round() as i32,
            (y + glyph.offset_y).round() as i32,
            glyph.pixmap.as_ref(),
            &paint,
            self.transform,
            None,
        );
        self.dirty = true;
        Ok(())
    }

    /// Draws an image asset scaled into the `x, y, width, height` box.
    ///
    /// The path must resolve inside one of the allowed asset directories (the scene's directory,
    /// or an explicit `--asset`). Decoded and scaled versions are cached.
    pub fn draw_image(
        &mut self,
        path: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<(), String> {
        self.draw_image_inner(path, x, y, width, height, None)
    }

    /// Like [`Canvas::draw_image`], but replaces the image colors with `color` while keeping the
    /// image's alpha. Useful for stencils (a monochrome cutout tinted with the palette).
    pub fn draw_image_tinted(
        &mut self,
        path: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [u8; 4],
    ) -> Result<(), String> {
        self.draw_image_inner(path, x, y, width, height, Some(color))
    }

    fn draw_image_inner(
        &mut self,
        path: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        tint: Option<[u8; 4]>,
    ) -> Result<(), String> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Ok(());
        }
        let resolved = self.resolve_asset(path)?;
        let (target_w, target_h) = (width.round() as u32, height.round() as u32);
        let key = (resolved.clone(), target_w, target_h, tint);

        if !self.scaled.contains_key(&key) {
            let source = match self.assets.get(&resolved) {
                Some(image) => image.clone(),
                None => {
                    let image = image::open(&resolved)
                        .map_err(|e| format!("failed to open {}: {e}", resolved.display()))?
                        .to_rgba8();
                    self.assets.insert(resolved.clone(), image.clone());
                    image
                }
            };

            let resized = if source.width() == target_w && source.height() == target_h {
                source
            } else {
                image::imageops::resize(
                    &source,
                    target_w,
                    target_h,
                    image::imageops::FilterType::Lanczos3,
                )
            };

            let mut data = Vec::with_capacity((target_w * target_h * 4) as usize);
            for pixel in resized.pixels() {
                let [r, g, b, a] = pixel.0;
                let (r, g, b, a) = match tint {
                    Some([tr, tg, tb, ta]) => {
                        let alpha = (u32::from(a) * u32::from(ta) / 255) as u8;
                        (tr, tg, tb, alpha)
                    }
                    None => (r, g, b, a),
                };
                data.extend_from_slice(&[
                    premultiply(r, a),
                    premultiply(g, a),
                    premultiply(b, a),
                    a,
                ]);
            }
            let size = IntSize::from_wh(target_w, target_h)
                .ok_or_else(|| format!("invalid image size {target_w}x{target_h}"))?;
            let pixmap =
                Pixmap::from_vec(data, size).ok_or_else(|| "invalid image buffer".to_string())?;
            self.scaled.insert(key.clone(), pixmap);
        }

        let pixmap = self.scaled.get(&key).expect("just inserted");
        let paint = PixmapPaint {
            opacity: self.alpha,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(
            x.round() as i32,
            y.round() as i32,
            pixmap.as_ref(),
            &paint,
            self.transform,
            None,
        );
        self.dirty = true;
        Ok(())
    }

    /// Recolors the given region with a luminance gradient built from `colors`.
    ///
    /// This is the position-specific twin of the image pipeline's `--map-palette`: call it on the
    /// whole canvas, or on any rectangle to let the palette affect only that area. `strength`
    /// blends with the existing pixels.
    pub fn remap(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        colors: &[[u8; 4]],
        strength: f32,
    ) -> Result<(), String> {
        let stops = gradient_stops(colors);
        if stops.is_empty() || strength <= 0.0 {
            return Ok(());
        }

        let canvas_w = self.pixmap.width();
        let canvas_h = self.pixmap.height();
        let x0 = x.max(0.0).floor() as u32;
        let y0 = y.max(0.0).floor() as u32;
        let x1 = ((x + width).min(canvas_w as f32).max(0.0)).floor() as u32;
        let y1 = ((y + height).min(canvas_h as f32).max(0.0)).floor() as u32;
        if x1 <= x0 || y1 <= y0 {
            return Ok(());
        }

        let region_w = x1 - x0;
        let region_h = y1 - y0;
        let mut buffer = Vec::with_capacity((region_w * region_h * 4) as usize);
        {
            let pixels = self.pixmap.pixels();
            for row in y0..y1 {
                for column in x0..x1 {
                    let straight = pixels[(row * canvas_w + column) as usize].demultiply();
                    buffer.extend_from_slice(&[
                        straight.red(),
                        straight.green(),
                        straight.blue(),
                        straight.alpha(),
                    ]);
                }
            }
        }

        crate::effects::palette_map(&mut buffer, 4, &stops, strength);

        let pixels = self.pixmap.pixels_mut();
        for (index, pixel) in buffer.as_chunks::<4>().0.iter().enumerate() {
            let [r, g, b, a] = *pixel;
            let column = x0 + (index as u32 % region_w);
            let row = y0 + (index as u32 / region_w);
            let color = PremultipliedColorU8::from_rgba(
                premultiply(r, a),
                premultiply(g, a),
                premultiply(b, a),
                a,
            )
            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
            pixels[(row * canvas_w + column) as usize] = color;
        }
        self.dirty = true;
        Ok(())
    }

    fn resolve_asset(&self, path: &str) -> Result<PathBuf, String> {
        let requested = FsPath::new(path);
        let mut candidates = Vec::new();
        if requested.is_absolute() {
            candidates.push(requested.to_path_buf());
        } else {
            for root in &self.allowed {
                candidates.push(root.join(requested));
            }
        }

        for candidate in candidates {
            let Ok(canonical) = candidate.canonicalize() else {
                continue;
            };
            if self.allowed.iter().any(|root| canonical.starts_with(root)) && canonical.is_file() {
                return Ok(canonical);
            }
        }

        Err(format!(
            "image '{path}' is not allowed: assets must live next to the scene or in an --asset path"
        ))
    }

    /// Composites the canvas over an opaque background and returns interleaved bytes.
    ///
    /// `channels` must be 3 (RGB) or 4 (RGBA with opaque alpha). When `swap_rb` is true the red
    /// and blue channels are swapped, matching `ipc::PixelFormat`.
    #[must_use]
    pub fn to_flat(&self, channels: u8, swap_rb: bool, background: [u8; 3]) -> Box<[u8]> {
        let channels = usize::from(channels.clamp(3, 4));
        let data = self.pixmap.data();

        // Opaque surfaces need no alpha compositing (premultiplied == straight for alpha 255).
        if self.opaque {
            let mut out = vec![0u8; data.len() / 4 * channels];
            for (pixel, dst) in data
                .as_chunks::<4>()
                .0
                .iter()
                .zip(out.chunks_exact_mut(channels))
            {
                dst[0] = if swap_rb { pixel[2] } else { pixel[0] };
                dst[1] = pixel[1];
                dst[2] = if swap_rb { pixel[0] } else { pixel[2] };
                if channels == 4 {
                    dst[3] = 255;
                }
            }
            return out.into_boxed_slice();
        }

        let mut out = Vec::with_capacity(data.len() / 4 * channels);

        for pixel in data.as_chunks::<4>().0 {
            let alpha = u32::from(pixel[3]);
            let inverse = 255 - alpha;
            let mut rgb = [0u8; 3];
            for channel in 0..3 {
                let background = u32::from(background[channel]) * inverse / 255;
                rgb[channel] = (u32::from(pixel[channel]) + background).min(255) as u8;
            }
            if swap_rb {
                rgb.swap(0, 2);
            }
            out.extend_from_slice(&rgb);
            if channels == 4 {
                out.push(255);
            }
        }

        out.into_boxed_slice()
    }

    fn draw_rect(&mut self, rect: tiny_skia::Rect) {
        let mut builder = PathBuilder::new();
        builder.push_rect(rect);
        self.draw_builder(builder);
    }

    fn draw_builder(&mut self, builder: PathBuilder) {
        let Some(path) = builder.finish() else {
            return;
        };
        self.dirty = true;
        if let Some(color) = self.fill {
            let paint = self.paint(color);
            self.pixmap
                .fill_path(&path, &paint, FillRule::Winding, self.transform, None);
        }
        self.stroke_only(&path);
    }

    fn stroke_only(&mut self, path: &Path) {
        if let Some((color, width)) = self.stroke
            && width > 0.0
        {
            let paint = self.paint(color);
            let stroke = Stroke {
                width,
                ..Stroke::default()
            };
            self.pixmap
                .stroke_path(path, &paint, &stroke, self.transform, None);
        }
    }

    fn paint(&self, color: [u8; 4]) -> Paint<'static> {
        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        if let Some(shader) = &self.shader {
            paint.shader = shader.clone();
        } else {
            let alpha = (f32::from(color[3]) * self.alpha).round().clamp(0.0, 255.0) as u8;
            paint.set_color_rgba8(color[0], color[1], color[2], alpha);
        }
        paint
    }

    fn gradient_stops(&self, colors: &[[u8; 4]]) -> Vec<GradientStop> {
        let last = colors.len().saturating_sub(1).max(1) as f32;
        colors
            .iter()
            .enumerate()
            .map(|(index, color)| {
                let alpha = (f32::from(color[3]) * self.alpha).round().clamp(0.0, 255.0) as u8;
                GradientStop::new(
                    index as f32 / last,
                    Color::from_rgba8(color[0], color[1], color[2], alpha),
                )
            })
            .collect()
    }
}

fn premultiply(channel: u8, alpha: u8) -> u8 {
    ((u32::from(channel) * u32::from(alpha)) / 255) as u8
}

/// Loads the system fonts once per process and maps the generic families to real ones.
fn system_fonts() -> Arc<fontdb::Database> {
    static FONTS: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();

            let families: Vec<String> = db
                .faces()
                .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
                .collect();
            let has = |candidate: &str| families.iter().any(|name| name == candidate);

            for candidate in [
                "Inter",
                "Noto Sans",
                "Open Sans",
                "DejaVu Sans",
                "Liberation Sans",
                "Cantarell",
                "Adwaita Sans",
                "Arial",
            ] {
                if has(candidate) {
                    db.set_sans_serif_family(candidate);
                    break;
                }
            }
            for candidate in [
                "JetBrains Mono",
                "Hack Nerd Font",
                "Fira Code",
                "Cascadia Code",
                "DejaVu Sans Mono",
                "Liberation Mono",
            ] {
                if has(candidate) {
                    db.set_monospace_family(candidate);
                    break;
                }
            }

            Arc::new(db)
        })
        .clone()
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn luma(color: [u8; 3]) -> f32 {
    f32::from(color[0]) * 0.2126 + f32::from(color[1]) * 0.7152 + f32::from(color[2]) * 0.0722
}

fn gradient_stops(colors: &[[u8; 4]]) -> Vec<[u8; 3]> {
    let mut stops: Vec<[u8; 3]> = colors.iter().map(|c| [c[0], c[1], c[2]]).collect();
    stops.sort_by(|a, b| luma(*a).total_cmp(&luma(*b)));
    stops.dedup_by(|a, b| luma(*a).total_cmp(&luma(*b)) == std::cmp::Ordering::Equal);
    stops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixels(canvas: &Canvas) -> Vec<[u8; 3]> {
        let flat = canvas.to_flat(3, false, [0, 0, 0]);
        flat.as_chunks::<3>().0.to_vec()
    }

    #[test]
    fn clear_fills_the_surface() {
        let mut canvas = Canvas::new(4, 2).unwrap();
        canvas.clear([255, 0, 0, 255]);
        assert!(pixels(&canvas).iter().all(|p| *p == [255, 0, 0]));
    }

    #[test]
    fn rect_draws_at_the_requested_position() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas.fill(Some([0, 255, 0, 255]));
        canvas.rect(1.0, 1.0, 2.0, 2.0);
        let p = pixels(&canvas);
        assert_eq!(p[0], [0, 0, 0]);
        assert_eq!(p[5], [0, 255, 0]);
        assert_eq!(p[10], [0, 255, 0]);
    }

    #[test]
    fn circle_respects_radius() {
        let mut canvas = Canvas::new(5, 5).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas.fill(Some([255, 255, 255, 255]));
        canvas.circle(2.5, 2.5, 1.0);
        let p = pixels(&canvas);
        assert_eq!(p[12], [255, 255, 255]);
        assert_eq!(p[0], [0, 0, 0]);
    }

    #[test]
    fn translate_moves_subsequent_shapes() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas.fill(Some([0, 0, 255, 255]));
        canvas.translate(2.0, 0.0);
        canvas.rect(0.0, 0.0, 2.0, 4.0);
        let p = pixels(&canvas);
        assert_eq!(p[0], [0, 0, 0]);
        assert_eq!(p[2], [0, 0, 255]);
    }

    #[test]
    fn alpha_blends_into_the_background() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.clear([0, 0, 0, 0]);
        canvas.fill(Some([255, 255, 255, 128]));
        canvas.rect(0.0, 0.0, 1.0, 1.0);
        // Premultiplied white at 50% composites to mid gray over black.
        let p = pixels(&canvas);
        assert!(p[0][0] > 120 && p[0][0] < 135, "{:?}", p[0]);
    }

    #[test]
    fn gradient_varies_along_the_axis() {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas.linear_gradient(0.0, 0.0, 4.0, 0.0, &[[0, 0, 0, 255], [255, 255, 255, 255]]);
        canvas.rect(0.0, 0.0, 4.0, 1.0);
        let p = pixels(&canvas);
        assert!(p[0][0] < p[3][0], "{p:?}");
    }

    #[test]
    fn swap_rb_flips_channels() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.clear([10, 20, 30, 255]);
        assert_eq!(&canvas.to_flat(3, true, [0, 0, 0])[..], &[30, 20, 10]);
        assert_eq!(&canvas.to_flat(4, false, [0, 0, 0])[..], &[10, 20, 30, 255]);
    }

    #[test]
    fn transparent_areas_show_the_background() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        canvas.clear([0, 0, 0, 0]);
        assert_eq!(&canvas.to_flat(3, false, [7, 8, 9])[..], &[7, 8, 9]);
    }

    #[test]
    fn text_renders_glyphs() {
        if system_fonts().is_empty() {
            return;
        }
        let mut canvas = Canvas::new(64, 32).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas
            .text(
                "Ag",
                2.0,
                24.0,
                24.0,
                [255, 255, 255, 255],
                "sans-serif",
                "start",
                false,
            )
            .unwrap();
        assert!(
            pixels(&canvas).iter().any(|pixel| pixel[0] > 200),
            "no glyph pixels were drawn"
        );
    }

    #[test]
    fn empty_text_is_a_noop() {
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.clear([0, 0, 0, 255]);
        canvas
            .text("", 0.0, 0.0, 10.0, [255, 255, 255, 255], "", "", false)
            .unwrap();
        assert!(pixels(&canvas).iter().all(|p| *p == [0, 0, 0]));
    }

    fn write_test_image(name: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("xwww-canvas-assets-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("asset.png");
        let mut img = image::RgbaImage::new(2, 1);
        img.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
        img.put_pixel(1, 0, image::Rgba([0, 0, 0, 255]));
        img.save(&path).unwrap();
        (dir, path)
    }

    #[test]
    fn draws_an_allowed_asset() {
        let (dir, _) = write_test_image("draw");
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.allow_asset_dir(&dir);
        canvas.draw_image("asset.png", 0.0, 0.0, 2.0, 1.0).unwrap();
        let flat = canvas.to_flat(3, false, [0, 0, 0]);
        assert_eq!(&flat[0..3], &[255, 255, 255]);
        assert_eq!(&flat[3..6], &[0, 0, 0]);
    }

    #[test]
    fn draws_a_tinted_asset() {
        let (dir, _) = write_test_image("tint");
        let mut canvas = Canvas::new(2, 1).unwrap();
        canvas.allow_asset_dir(&dir);
        canvas
            .draw_image_tinted("asset.png", 0.0, 0.0, 2.0, 1.0, [255, 0, 0, 255])
            .unwrap();
        let flat = canvas.to_flat(3, false, [0, 0, 0]);
        assert_eq!(&flat[0..3], &[255, 0, 0]);
        assert_eq!(&flat[3..6], &[255, 0, 0]);
    }

    #[test]
    fn rejects_assets_outside_the_allowlist() {
        let (_, path) = write_test_image("reject");
        let mut canvas = Canvas::new(2, 1).unwrap();
        let error = canvas
            .draw_image(path.to_str().unwrap(), 0.0, 0.0, 2.0, 1.0)
            .unwrap_err();
        assert!(error.contains("not allowed"), "{error}");
    }

    #[test]
    fn remap_only_touches_the_requested_region() {
        let mut canvas = Canvas::new(4, 1).unwrap();
        canvas.clear([128, 128, 128, 255]);
        canvas
            .remap(
                0.0,
                0.0,
                2.0,
                1.0,
                &[[0, 0, 255, 255], [255, 0, 0, 255]],
                1.0,
            )
            .unwrap();
        let flat = canvas.to_flat(3, false, [0, 0, 0]);
        assert!(flat[0] != 128 || flat[2] != 128, "{flat:?}");
        assert_eq!(&flat[6..9], &[128, 128, 128]);
        assert_eq!(&flat[9..12], &[128, 128, 128]);
    }
}
