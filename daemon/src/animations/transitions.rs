use core::num::NonZeroU8;

use crate::{WaylandObject, wallpaper::WallpaperCell};
use common::ipc::{PixelFormat, Transition, TransitionType};

use super::keyframe::{AnimationSequence, Keyframe, Vector2, functions::BezierCurve};
use waybackend::{Waybackend, objman::ObjectManager};

fn bezier_seq(transition: &Transition, start: f32, end: f32) -> (AnimationSequence, f64) {
    let bezier = BezierCurve::from(
        Vector2 {
            x: transition.bezier.0,
            y: transition.bezier.1,
        },
        Vector2 {
            x: transition.bezier.2,
            y: transition.bezier.3,
        },
    );
    let seq = [
        Keyframe::new(start, 0.0, bezier),
        Keyframe::new(end, transition.duration as f64, bezier),
    ];
    let animation_sequence = AnimationSequence::from(seq);
    (animation_sequence, now_f64())
}

#[inline(always)]
/// This is calculating the following:
/// ```
/// if old.abs_diff(*new) < step.get() {
///     *old = *new;
/// } else if *old > *new {
///     *old -= step.get();
/// } else {
///     *old += step.get();
/// }
/// ```
/// However, it does so with less branches, making it more amenable to being autovectorized.
/// From my tests, this is almost twice as fast as the above code in x86_64, when compiling without
/// any target features. It only loses slightly (5%) in speed when we compile with avx512. However,
/// avx512 is by itself already pretty fast anyway, and thus benefits less from this.
fn change_byte(step: NonZeroU8, old: &mut u8, new: &u8) {
    let min = (*old).min(*new);
    let max = (*old).max(*new);
    let diff = max - min;
    let mut to_add = step.get().min(diff);

    if *old > *new {
        to_add = to_add.wrapping_neg();
    }
    *old = old.wrapping_add(to_add);
}

struct None;
impl None {
    fn run(
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        for w in wallpapers.iter() {
            w.borrow_mut()
                .canvas_change(backend, objman, pixel_format, |canvas| {
                    canvas.copy_from_slice(img);
                });
        }
        true
    }
}

#[allow(private_interfaces)]
pub enum Effect {
    None(None),
    Simple(Simple),
    Fade(Fade),
    Wave(Wave),
    Grow(Grow),
    Outer(Outer),
    Glitch(Glitch),
    Decrypt(Decrypt),
    Dissolve(Dissolve),
    Clock(Clock),
    Zoom(Zoom),
}

impl Effect {
    pub fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        match transition.transition_type {
            TransitionType::Simple => Self::Simple(Simple::new(transition.step)),
            TransitionType::Fade => Self::Fade(Fade::new(transition)),
            TransitionType::Outer => Self::Outer(Outer::new(transition, dimensions)),
            TransitionType::Grow => Self::Grow(Grow::new(transition, dimensions)),
            TransitionType::Wipe | TransitionType::Wave => {
                Self::Wave(Wave::new(transition, dimensions))
            }
            TransitionType::Glitch => Self::Glitch(Glitch::new(transition, dimensions)),
            TransitionType::Decrypt => Self::Decrypt(Decrypt::new(transition, dimensions)),
            TransitionType::Dissolve => Self::Dissolve(Dissolve::new(transition, dimensions)),
            TransitionType::Clock => Self::Clock(Clock::new(transition, dimensions)),
            TransitionType::Zoom => Self::Zoom(Zoom::new(transition, dimensions)),
            TransitionType::None => Self::None(None),
        }
    }

    pub fn execute(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let done = match self {
            Effect::None(_) => None::run(backend, objman, pixel_format, wallpapers, img),
            Effect::Simple(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Fade(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Wave(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Grow(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Outer(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Glitch(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Decrypt(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Dissolve(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Clock(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Zoom(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
        };
        // we only finish for real if we are doing a None or a Simple transition
        if done {
            #[inline(always)]
            const fn new_nonzero(step: u8) -> NonZeroU8 {
                NonZeroU8::new(step / 4 + 4).unwrap()
            }
            *self = match self {
                Effect::None(_) | Effect::Simple(_) => return true,
                Effect::Fade(t) => Effect::Simple(Simple::new(new_nonzero(t.step as u8))),
                Effect::Wave(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Grow(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Outer(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Glitch(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Decrypt(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Dissolve(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Clock(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Zoom(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
            };
            return false;
        }
        done
    }
}

struct Simple {
    step: NonZeroU8,
}

impl Simple {
    fn new(step: NonZeroU8) -> Self {
        Self { step }
    }
    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let step = self.step;
        let mut done = true;
        for wallpaper in wallpapers.iter() {
            wallpaper
                .borrow_mut()
                .canvas_change(backend, objman, pixel_format, |canvas| {
                    for (old, new) in canvas.iter_mut().zip(img) {
                        change_byte(step, old, new);
                    }
                    done = done && canvas == img;
                });
        }
        done
    }
}

struct Fade {
    start: f64,
    seq: AnimationSequence,
    step: u16,
}

impl Fade {
    fn new(transition: &Transition) -> Self {
        let (seq, start) = bezier_seq(transition, 0.0, 1.0);
        let step = 0;
        Self { start, seq, step }
    }
    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        for wallpaper in wallpapers.iter() {
            wallpaper
                .borrow_mut()
                .canvas_change(backend, objman, pixel_format, |canvas| {
                    for (old, new) in canvas.iter_mut().zip(img) {
                        let x = *old as u16 * (256 - self.step);
                        let y = *new as u16 * self.step;
                        *old = ((x + y) >> 8) as u8;
                    }
                });
        }
        self.step = (256.0 * self.seq.now() as f64).trunc() as u16;
        self.seq.advance_to(elapsed(self.start));
        self.seq.finished()
    }
}

struct Wave {
    start: f64,
    seq: AnimationSequence,
    center: (u32, u32),
    sin: f64,
    cos: f64,
    scale_x: f64,
    scale_y: f64,
    circle_radius: f64,
    a: f64,
    b: f64,
    step: NonZeroU8,
}

impl Wave {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let width = dimensions.0;
        let height = dimensions.1;
        let center = (width / 2, height / 2);
        let screen_diag = ((width.pow(2) + height.pow(2)) as f64).sqrt();

        let angle = transition.angle.to_radians();
        let (sin, cos) = angle.sin_cos();
        let (scale_x, scale_y) = (transition.wave.0 as f64, transition.wave.1 as f64);

        let circle_radius = screen_diag / 2.0;

        let offset = (sin.abs() * width as f64 + cos.abs() * height as f64) * 2.0;
        let a = circle_radius * cos;
        let b = circle_radius * sin;
        let max_offset = circle_radius.powi(2) * 2.0;

        let (seq, start) = bezier_seq(transition, offset as f32, max_offset as f32);

        let step = transition.step;
        Self {
            start,
            seq,
            center,
            sin,
            cos,
            scale_x,
            scale_y,
            circle_radius,
            a,
            b,
            step,
        }
    }
    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let Self {
            center,
            sin,
            cos,
            scale_x,
            scale_y,
            circle_radius,
            a,
            b,
            step,
            ..
        } = *self;
        // graph: https://www.desmos.com/calculator/wunde042es
        //
        // checks if a pixel is to the left or right of the line
        let is_low = |x: f64, y: f64, offset: f64| {
            let x = x - center.0 as f64;
            let y = y - center.1 as f64;

            let lhs = y * sin - x * cos;

            let f = ((x * sin + y * cos) / scale_x).sin() * scale_y;
            let rhs = f - circle_radius + offset / circle_radius;
            lhs <= rhs
        };

        let channels = pixel_format.channels() as usize;
        let offset = self.seq.now() as f64;
        self.seq.advance_to(elapsed(self.start));

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let stride = width * pixel_format.channels() as usize;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                // divide in 3 sections: the one we know will not be drawn to, the one we know
                // WILL be drawn to, and the one we need to do a more expensive check on.
                // We do this by creating 2 lines: the first tangential to the wave's peaks,
                // the second to its valeys. In-between is where we have to do the more
                // expensive checks
                for line in 0..height {
                    let y = ((height - line) as f64 - center.1 as f64 - scale_y * sin) * b;
                    let x =
                        (circle_radius.powi(2) - y - offset) / a + center.0 as f64 + scale_y * cos;
                    let x = x.min(width as f64);
                    let (col_begin, col_end) = if a.is_sign_negative() {
                        (0usize, x as usize * channels)
                    } else {
                        (x as usize * channels, stride)
                    };
                    for col in col_begin..col_end {
                        let old = unsafe { canvas.get_unchecked_mut(line * stride + col) };
                        let new = unsafe { img.get_unchecked(line * stride + col) };
                        change_byte(step, old, new);
                    }
                    let old_x = x;
                    let y = ((height - line) as f64 - center.1 as f64 + scale_y * sin) * b;
                    let x =
                        (circle_radius.powi(2) - y - offset) / a + center.0 as f64 - scale_y * cos;
                    let x = x.min(width as f64);
                    let (col_begin, col_end) = if old_x < x {
                        (old_x as usize, x as usize)
                    } else {
                        (x as usize, old_x as usize)
                    };
                    for col in col_begin..col_end {
                        if is_low(col as f64, line as f64, offset) {
                            let i = line * stride + col * channels;
                            for j in 0..channels {
                                let old = unsafe { canvas.get_unchecked_mut(i + j) };
                                let new = unsafe { img.get_unchecked(i + j) };
                                change_byte(step, old, new);
                            }
                        }
                    }
                }
            });
        }

        self.seq.finished()
    }
}

struct Grow {
    start: f64,
    seq: AnimationSequence,
    center_x: usize,
    center_y: usize,
    dist_center: f32,
    step: NonZeroU8,
}

impl Grow {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as f32, dimensions.1 as f32);
        let (center_x, center_y) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let dist_center: f32 = 0.0;
        let dist_end: f32 = {
            let mut x = center_x;
            let mut y = center_y;
            if x < width / 2.0 {
                x = width - 1.0 - x;
            }
            if y < height / 2.0 {
                y = height - 1.0 - y;
            }
            f32::sqrt(x.powi(2) + y.powi(2))
        };

        let (center_x, center_y) = (center_x as usize, center_y as usize);

        let step = transition.step;
        let (seq, start) = bezier_seq(transition, 0.0, dist_end);
        Self {
            start,
            seq,
            center_x,
            center_y,
            dist_center,
            step,
        }
    }
    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let Self {
            center_x,
            center_y,
            dist_center,
            step,
            ..
        } = *self;
        let channels = pixel_format.channels() as usize;

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let stride = width * pixel_format.channels() as usize;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                let line_begin = center_y.saturating_sub(dist_center as usize);
                let line_end = height.min(center_y + dist_center as usize);

                // to plot half a circle with radius r, we do sqrt(r^2 - x^2)
                for line in line_begin..line_end {
                    let offset = (dist_center.powi(2) - (center_y as f32 - line as f32).powi(2))
                        .sqrt() as usize;
                    let col_begin = center_x.saturating_sub(offset) * channels;
                    let col_end = width.min(center_x + offset) * channels;
                    for col in col_begin..col_end {
                        let old = unsafe { canvas.get_unchecked_mut(line * stride + col) };
                        let new = unsafe { img.get_unchecked(line * stride + col) };
                        change_byte(step, old, new);
                    }
                }
            });
        }

        self.dist_center = self.seq.now();
        self.seq.advance_to(elapsed(self.start));
        self.seq.finished()
    }
}

struct Outer {
    start: f64,
    seq: AnimationSequence,
    center_x: usize,
    center_y: usize,
    dist_center: f32,
    step: NonZeroU8,
}

impl Outer {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as f32, dimensions.1 as f32);
        let (center_x, center_y) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let dist_center = {
            let mut x = center_x;
            let mut y = center_y;
            if x < width / 2.0 {
                x = width - 1.0 - x;
            }
            if y < height / 2.0 {
                y = height - 1.0 - y;
            }
            f32::sqrt(x.powi(2) + y.powi(2))
        };
        let (center_x, center_y) = (center_x as usize, center_y as usize);

        let step = transition.step;
        let (seq, start) = bezier_seq(transition, dist_center, 0.0);
        Self {
            start,
            seq,
            center_x,
            center_y,
            dist_center,
            step,
        }
    }
    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let Self {
            center_x,
            center_y,
            dist_center,
            step,
            ..
        } = *self;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let stride = width * pixel_format.channels() as usize;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                // to plot half a circle with radius r, we do sqrt(r^2 - x^2)
                for line in 0..height {
                    let offset = (dist_center.powi(2) - (center_y as f32 - line as f32).powi(2))
                        .sqrt() as usize;
                    let col_begin = center_x.saturating_sub(offset) * channels;
                    let col_end = width.min(center_x + offset) * channels;
                    for col in 0..col_begin {
                        let old = unsafe { canvas.get_unchecked_mut(line * stride + col) };
                        let new = unsafe { img.get_unchecked(line * stride + col) };
                        change_byte(step, old, new);
                    }
                    for col in col_end..stride {
                        let old = unsafe { canvas.get_unchecked_mut(line * stride + col) };
                        let new = unsafe { img.get_unchecked(line * stride + col) };
                        change_byte(step, old, new);
                    }
                }
            });
        }
        self.dist_center = self.seq.now();
        self.seq.advance_to(elapsed(self.start));
        self.seq.finished()
    }
}

fn now_f64() -> f64 {
    let t = crate::clock::get();
    t.tv_sec as f64 + t.tv_nsec as f64 / 1_000_000_000.0
}

fn elapsed(start: f64) -> f64 {
    now_f64() - start
}

/// Tiny xorshift64 pseudo-random generator, seeded deterministically. We avoid pulling in a full
/// random-number crate just for the glitch and decrypt effects, since they only need cheap,
/// repeatable noise.
fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Returns a pseudo-random value in `[0, n)`.
fn rand_range(rng: &mut u64, n: usize) -> usize {
    if n == 0 {
        0
    } else {
        (xorshift(rng) as usize) % n
    }
}

/// Builds a seed derived from the current time and the output dimensions, so each transition has a
/// distinct, non-deterministic sequence of glitches/reveals.
fn seed(dimensions: (u32, u32)) -> u64 {
    let mut s = now_f64().to_bits() ^ ((dimensions.0 as u64) << 32) ^ dimensions.1 as u64;
    if s == 0 {
        s = 0x9E37_79B9_7F4A_7C15;
    }
    s
}

/// `glitch` transition: reveals the new image with a digital corruption effect. Horizontal bands
/// are randomly torn and shifted (with wrap-around) and random noise is injected. The amount of
/// corruption decreases over the duration until the image settles cleanly.
struct Glitch {
    start: f64,
    duration: f64,
    step: NonZeroU8,
    rng: u64,
}

impl Glitch {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            step: transition.step,
            rng: seed(dimensions),
        }
    }

    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let elapsed = elapsed(self.start);
        let t = (elapsed / self.duration).clamp(0.0, 1.0);
        let intensity = 1.0 - t;
        let rng = &mut self.rng;

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let channels = pixel_format.channels() as usize;
            let stride = width * channels;

            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                // start from the final image, then corrupt a shrinking fraction of it
                canvas.copy_from_slice(img);
                if intensity <= 0.0 {
                    return;
                }

                // horizontal tearing: shift whole rows with wrap-around
                let band_count = ((intensity * height as f64) / 8.0) as usize;
                for _ in 0..band_count {
                    let y = rand_range(rng, height);
                    let band_h = rand_range(rng, 16).max(1);
                    let shift = rand_range(rng, stride).max(1);
                    for line in y..(y + band_h).min(height) {
                        let start = line * stride;
                        let row = &mut canvas[start..start + stride];
                        if xorshift(rng) & 1 == 0 {
                            row.rotate_left(shift);
                        } else {
                            row.rotate_right(shift);
                        }
                    }
                }

                // sprinkle random noise
                let noise = ((intensity * stride as f64 * height as f64) / 2000.0) as usize;
                for _ in 0..noise {
                    let i = rand_range(rng, canvas.len());
                    canvas[i] = (xorshift(rng) & 0xff) as u8;
                }
            });
        }

        t >= 1.0
    }
}

/// `decrypt` transition: reveals the new image block by block, in a pseudo-random order, like a
/// cipher being decoded. Each block snaps into place as it is "decrypted" until the whole image is
/// shown.
struct Decrypt {
    start: f64,
    duration: f64,
    step: NonZeroU8,
    blocks: Box<[u32]>,
    cols: usize,
    block_size: usize,
    revealed: usize,
}

impl Decrypt {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (w, h) = (dimensions.0 as usize, dimensions.1 as usize);
        let block_size = 32usize;
        let cols = w.div_ceil(block_size);
        let rows = h.div_ceil(block_size);
        let total = cols * rows;

        let mut blocks: Vec<u32> = (0..total as u32).collect();
        let mut rng = seed(dimensions);
        for i in (1..total).rev() {
            let j = rand_range(&mut rng, i + 1);
            blocks.swap(i, j);
        }

        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            step: transition.step,
            blocks: blocks.into_boxed_slice(),
            cols,
            block_size,
            revealed: 0,
        }
    }

    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let elapsed = elapsed(self.start);
        let t = (elapsed / self.duration).clamp(0.0, 1.0);
        let target = (t * self.blocks.len() as f64) as usize;

        // blocks that still need to be revealed this frame
        let new_blocks: Vec<usize> = (self.revealed..target)
            .map(|i| self.blocks[i] as usize)
            .collect();
        self.revealed = target;

        // Always touch the canvas, even with no new blocks: skipping the draw
        // lets the buffer pool drain (all buffers released) and the next
        // attach_buffer_and_damage_surface panics on an empty pool.
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let channels = pixel_format.channels() as usize;
            let stride = width * channels;
            let cols = self.cols;
            let block_size = self.block_size;

            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for &block in &new_blocks {
                    let row = block / cols;
                    let col = block % cols;
                    let x0 = col * block_size;
                    let y0 = row * block_size;
                    let x1 = (x0 + block_size).min(width);
                    let y1 = (y0 + block_size).min(height);
                    for y in y0..y1 {
                        let off = y * stride + x0 * channels;
                        let len = (x1 - x0) * channels;
                        canvas[off..off + len].copy_from_slice(&img[off..off + len]);
                    }
                }
            });
        }

        t >= 1.0
    }
}

/// `dissolve` transition: reveals the new image pixel by pixel, in a pseudo-random order. A
/// shuffled list of every pixel index is generated up front, and pixels are revealed from it as the
/// transition progresses, so every pixel snaps in exactly once.
struct Dissolve {
    start: f64,
    duration: f64,
    step: NonZeroU8,
    order: Box<[u32]>,
    revealed: usize,
}

impl Dissolve {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let total = (dimensions.0 as usize) * (dimensions.1 as usize);
        let mut order: Vec<u32> = (0..total as u32).collect();

        let mut rng = seed(dimensions);
        for i in (1..total).rev() {
            let j = rand_range(&mut rng, i + 1);
            order.swap(i, j);
        }

        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            step: transition.step,
            order: order.into_boxed_slice(),
            revealed: 0,
        }
    }

    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let elapsed = elapsed(self.start);
        let t = (elapsed / self.duration).clamp(0.0, 1.0);
        let target = (t * self.order.len() as f64) as usize;

        let new: Vec<usize> = (self.revealed..target)
            .map(|i| self.order[i] as usize)
            .collect();
        self.revealed = target;

        // Always touch the canvas (see Decrypt): an empty reveal batch must not
        // skip the draw, or the drained pool crashes the next attach.
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let channels = pixel_format.channels() as usize;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for &pixel in &new {
                    let off = pixel * channels;
                    canvas[off..off + channels].copy_from_slice(&img[off..off + channels]);
                }
            });
        }

        t >= 1.0
    }
}

/// `clock` transition: sweeps the new image in like a clock hand rotating around
/// `--transition-pos`. Pixels are revealed in angular order around the center.
struct Clock {
    start: f64,
    seq: AnimationSequence,
    center_x: f32,
    center_y: f32,
    step: NonZeroU8,
}

impl Clock {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (center_x, center_y) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let (seq, start) = bezier_seq(transition, 0.0, core::f32::consts::TAU);
        Self {
            start,
            seq,
            center_x,
            center_y,
            step: transition.step,
        }
    }

    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        let center_x = self.center_x;
        let center_y = self.center_y;
        let sweep = self.seq.now();
        self.seq.advance_to(elapsed(self.start));

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let channels = pixel_format.channels() as usize;
            let stride = width * channels;

            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for y in 0..height {
                    let dy = y as f32 - center_y;
                    for x in 0..width {
                        let dx = x as f32 - center_x;
                        let mut angle = dy.atan2(dx);
                        if angle < 0.0 {
                            angle += core::f32::consts::TAU;
                        }
                        if angle < sweep {
                            let off = y * stride + x * channels;
                            canvas[off..off + channels].copy_from_slice(&img[off..off + channels]);
                        }
                    }
                }
            });
        }

        self.seq.finished()
    }
}

/// `zoom` transition: the new image zooms out from `--transition-pos` (default: center) until it
/// fills the output. Pixels are sampled from the new image with a per-frame scale factor using
/// nearest-neighbor sampling, so it is cheap even on large outputs.
struct Zoom {
    start: f64,
    seq: AnimationSequence,
    center_x: f32,
    center_y: f32,
    step: NonZeroU8,
}

impl Zoom {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (center_x, center_y) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let (seq, start) = bezier_seq(transition, 0.0, 1.0);
        Self {
            start,
            seq,
            center_x,
            center_y,
            step: transition.step,
        }
    }

    fn run(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
        wallpapers: &mut [WallpaperCell],
        img: &[u8],
    ) -> bool {
        // scale goes from START_SCALE (zoomed in) down to 1.0 (full image)
        const START_SCALE: f32 = 2.0;
        let t = self.seq.now();
        let scale = START_SCALE + (1.0 - START_SCALE) * t;
        self.seq.advance_to(elapsed(self.start));

        let center_x = self.center_x;
        let center_y = self.center_y;

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let channels = pixel_format.channels() as usize;
            let stride = width * channels;

            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for y in 0..height {
                    let sy = (y as f32 - center_y) / scale + center_y;
                    if sy < 0.0 || sy >= height as f32 {
                        continue;
                    }
                    let sy = sy as usize;
                    for x in 0..width {
                        let sx = (x as f32 - center_x) / scale + center_x;
                        if sx < 0.0 || sx >= width as f32 {
                            continue;
                        }
                        let sx = sx as usize;
                        let src = sy * stride + sx * channels;
                        let dst = y * stride + x * channels;
                        canvas[dst..dst + channels].copy_from_slice(&img[src..src + channels]);
                    }
                }
            });
        }

        self.seq.finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_byte() {
        fn expected(step: NonZeroU8, old: &mut u8, new: &u8) {
            if old.abs_diff(*new) < step.get() {
                *old = *new;
            } else if *old > *new {
                *old -= step.get();
            } else {
                *old += step.get();
            }
        }

        for old in 0..=255 {
            for new in 0..=255 {
                for step in 1..=255 {
                    let step = NonZeroU8::new(step).unwrap();
                    let mut a = old;
                    let mut b = old;
                    expected(step, &mut a, &new);
                    super::change_byte(step, &mut b, &new);
                    assert_eq!(a, b, "old: {old}, new: {new}, step: {step}");
                }
            }
        }
    }
}
