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
    Pixelate(Pixelate),
    Ripple(Ripple),
    Blinds(Blinds),
    Spiral(Spiral),
    Static(Static),
    Parallax(Parallax),
    Melt(Melt),
    Shatter(Shatter),
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
            TransitionType::Pixelate => Self::Pixelate(Pixelate::new(transition, dimensions)),
            TransitionType::Ripple => Self::Ripple(Ripple::new(transition, dimensions)),
            TransitionType::Blinds => Self::Blinds(Blinds::new(transition, dimensions)),
            TransitionType::Spiral => Self::Spiral(Spiral::new(transition, dimensions)),
            TransitionType::Static => Self::Static(Static::new(transition, dimensions)),
            TransitionType::Parallax => {
                Self::Parallax(Parallax::new(transition, ParallaxDirection::Up))
            }
            TransitionType::ParallaxLeft => {
                Self::Parallax(Parallax::new(transition, ParallaxDirection::Left))
            }
            TransitionType::ParallaxRight => {
                Self::Parallax(Parallax::new(transition, ParallaxDirection::Right))
            }
            TransitionType::ParallaxInvert => {
                Self::Parallax(Parallax::new(transition, ParallaxDirection::Invert))
            }
            TransitionType::Melt => Self::Melt(Melt::new(transition, dimensions)),
            TransitionType::Shatter => Self::Shatter(Shatter::new(transition, dimensions)),
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
            Effect::Pixelate(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Ripple(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Blinds(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Spiral(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Static(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Parallax(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Melt(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
            Effect::Shatter(effect) => effect.run(backend, objman, pixel_format, wallpapers, img),
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
                Effect::Pixelate(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Ripple(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Blinds(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Spiral(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Static(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Parallax(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Melt(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
                Effect::Shatter(t) => Effect::Simple(Simple::new(new_nonzero(t.step.get()))),
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

/// Reveals pixels whose precomputed threshold is already passed by `progress` (`0..=255`).
/// Used by the threshold-based effects (ripple, blinds, spiral, static, melt).
fn reveal_by_threshold(
    canvas: &mut [u8],
    img: &[u8],
    thresholds: &[u8],
    channels: usize,
    progress: u8,
) {
    for (i, &threshold) in thresholds.iter().enumerate() {
        if threshold <= progress {
            let off = i * channels;
            canvas[off..off + channels].copy_from_slice(&img[off..off + channels]);
        }
    }
}

/// Pseudo-random float in `[0, 1)`.
fn rand_unit(rng: &mut u64) -> f32 {
    (xorshift(rng) >> 40) as f32 / (1u64 << 24) as f32
}

/// `pixelate` transition: the new image appears as a grid of flat color blocks in raster order,
/// and then the real pixels are filled block by block, so the picture sharpens progressively
/// instead of snapping at the end.
struct Pixelate {
    start: f64,
    duration: f64,
    cols: usize,
    rows: usize,
    block_size: usize,
    averages: Option<Box<[[u8; 4]]>>,
    revealed_blocks: usize,
    sharpened_blocks: usize,
    step: NonZeroU8,
}

/// Fraction of the transition used to lay down the flat blocks; the rest sharpens them.
const PIXELATE_BLOCK_PHASE: f64 = 0.65;

impl Pixelate {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let block_size = 48usize;
        let cols = (dimensions.0 as usize).div_ceil(block_size);
        let rows = (dimensions.1 as usize).div_ceil(block_size);
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            cols,
            rows,
            block_size,
            averages: Option::None,
            revealed_blocks: 0,
            sharpened_blocks: 0,
            step: transition.step,
        }
    }

    fn averages(&mut self, img: &[u8], width: usize, height: usize, channels: usize) {
        if self.averages.is_some() {
            return;
        }
        let block = self.block_size;
        let mut averages = Vec::with_capacity(self.cols * self.rows);
        for by in 0..self.rows {
            for bx in 0..self.cols {
                let x0 = bx * block;
                let y0 = by * block;
                let x1 = (x0 + block).min(width);
                let y1 = (y0 + block).min(height);
                let mut sum = [0u64; 4];
                let mut count = 0u64;
                for y in y0..y1 {
                    let start = y * width * channels + x0 * channels;
                    let end = y * width * channels + x1 * channels;
                    for pixel in img[start..end].chunks_exact(channels) {
                        for (c, value) in pixel.iter().enumerate() {
                            sum[c] += u64::from(*value);
                        }
                        count += 1;
                    }
                }
                let mut average = [0u8; 4];
                let divisor = count.max(1);
                for (c, value) in average.iter_mut().enumerate().take(channels) {
                    *value = (sum[c] / divisor) as u8;
                }
                averages.push(average);
            }
        }
        self.averages = Some(averages.into_boxed_slice());
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
        let channels = pixel_format.channels() as usize;
        let total = self.cols * self.rows;

        // Phase 1: flat color blocks. Phase 2: fill the real pixels block by block.
        let block_target = if t < PIXELATE_BLOCK_PHASE {
            ((t / PIXELATE_BLOCK_PHASE) * total as f64) as usize
        } else {
            total
        };
        let sharpen_target = if t <= PIXELATE_BLOCK_PHASE {
            0
        } else {
            (((t - PIXELATE_BLOCK_PHASE) / (1.0 - PIXELATE_BLOCK_PHASE)) * total as f64) as usize
        };

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            self.averages(img, width, height, channels);

            let averages = self.averages.as_ref().expect("just computed");
            let new_blocks: Vec<usize> = (self.revealed_blocks..block_target).collect();
            let real_blocks: Vec<usize> = (self.sharpened_blocks..sharpen_target).collect();
            let cols = self.cols;
            let block_size = self.block_size;
            let stride = width * channels;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for &block in &new_blocks {
                    let row = block / cols;
                    let col = block % cols;
                    let x0 = col * block_size;
                    let y0 = row * block_size;
                    let x1 = (x0 + block_size).min(width);
                    let y1 = (y0 + block_size).min(height);
                    let color = averages[block];
                    for y in y0..y1 {
                        let off = y * stride + x0 * channels;
                        let len = (x1 - x0) * channels;
                        for pixel in canvas[off..off + len].chunks_exact_mut(channels) {
                            pixel.copy_from_slice(&color[..channels]);
                        }
                    }
                }
                for &block in &real_blocks {
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
        self.revealed_blocks = block_target;
        self.sharpened_blocks = sharpen_target;

        t >= 1.0
    }
}

/// `ripple` transition: concentric wavy rings reveal the new image from `--transition-pos`.
struct Ripple {
    start: f64,
    duration: f64,
    thresholds: Box<[u8]>,
    step: NonZeroU8,
}

impl Ripple {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as usize, dimensions.1 as usize);
        let (cx, cy) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let max_dist = max_corner_distance(cx, cy, width, height);
        let mut thresholds = vec![255u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let normalized = (dist / max_dist).clamp(0.0, 1.0);
                let wave = 0.06 * (dist * 0.05).sin();
                let threshold = (normalized + wave).clamp(0.0, 1.0);
                thresholds[y * width + x] = (threshold * 255.0) as u8;
            }
        }
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            thresholds: thresholds.into_boxed_slice(),
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
        let t = (elapsed(self.start) / self.duration).clamp(0.0, 1.0);
        let progress = (t * 255.0) as u8;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                reveal_by_threshold(canvas, img, &self.thresholds, channels, progress);
            });
        }
        t >= 1.0
    }
}

fn max_corner_distance(cx: f32, cy: f32, width: usize, height: usize) -> f32 {
    let mut max = 0.0f32;
    for (x, y) in [
        (0.0f32, 0.0f32),
        (width as f32, 0.0),
        (0.0, height as f32),
        (width as f32, height as f32),
    ] {
        let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        if d > max {
            max = d;
        }
    }
    max.max(1.0)
}

/// `blinds` transition: horizontal bands open in staggered order, like venetian blinds.
struct Blinds {
    start: f64,
    duration: f64,
    thresholds: Box<[u8]>,
    step: NonZeroU8,
}

impl Blinds {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as usize, dimensions.1 as usize);
        let bands = 24usize;
        let band_height = (height / bands).max(1);
        let mut thresholds = vec![255u8; width * height];
        for y in 0..height {
            let band = y / band_height;
            let within = (y % band_height) as f32 / band_height as f32;
            let within = if band.is_multiple_of(2) {
                within
            } else {
                1.0 - within
            };
            let threshold = ((band as f32 + within * 0.8) / bands as f32).clamp(0.0, 1.0);
            let value = (threshold * 255.0) as u8;
            for x in 0..width {
                thresholds[y * width + x] = value;
            }
        }
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            thresholds: thresholds.into_boxed_slice(),
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
        let t = (elapsed(self.start) / self.duration).clamp(0.0, 1.0);
        let progress = (t * 255.0) as u8;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                reveal_by_threshold(canvas, img, &self.thresholds, channels, progress);
            });
        }
        t >= 1.0
    }
}

/// `spiral` transition: an angular sweep with a radius offset, like a spiral arm unwinding.
struct Spiral {
    start: f64,
    duration: f64,
    thresholds: Box<[u8]>,
    step: NonZeroU8,
}

impl Spiral {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as usize, dimensions.1 as usize);
        let (cx, cy) = transition.pos.to_pixel(dimensions, transition.invert_y);
        let max_dist = max_corner_distance(cx, cy, width, height);
        let mut thresholds = vec![255u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let mut angle = dy.atan2(dx);
                if angle < 0.0 {
                    angle += core::f32::consts::TAU;
                }
                let threshold = ((angle / core::f32::consts::TAU) + (dist / max_dist) * 2.0)
                    .fract()
                    .clamp(0.0, 1.0);
                thresholds[y * width + x] = (threshold * 255.0) as u8;
            }
        }
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            thresholds: thresholds.into_boxed_slice(),
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
        let t = (elapsed(self.start) / self.duration).clamp(0.0, 1.0);
        let progress = (t * 255.0) as u8;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                reveal_by_threshold(canvas, img, &self.thresholds, channels, progress);
            });
        }
        t >= 1.0
    }
}

/// `static` transition: the new image dissolves through television-like static. The reveal follows
/// a random per-pixel threshold, and a shrinking amount of random noise is sprinkled on top.
struct Static {
    start: f64,
    duration: f64,
    thresholds: Box<[u8]>,
    rng: u64,
    step: NonZeroU8,
}

impl Static {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let total = (dimensions.0 as usize) * (dimensions.1 as usize);
        let mut rng = seed(dimensions);
        let thresholds: Vec<u8> = (0..total).map(|_| (xorshift(&mut rng) & 0xff) as u8).collect();
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            thresholds: thresholds.into_boxed_slice(),
            rng: seed(dimensions),
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
        let elapsed = elapsed(self.start);
        let t = (elapsed / self.duration).clamp(0.0, 1.0);
        let progress = (t * 255.0) as u8;
        let intensity = 1.0 - t;
        let rng = &mut self.rng;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                reveal_by_threshold(canvas, img, &self.thresholds, channels, progress);
                if intensity > 0.0 {
                    let count = (intensity * canvas.len() as f64 / 1200.0) as usize;
                    for _ in 0..count {
                        let i = rand_range(rng, canvas.len());
                        canvas[i] = (xorshift(rng) & 0xff) as u8;
                    }
                }
            });
        }
        t >= 1.0
    }
}

#[derive(Clone, Copy)]
enum ParallaxDirection {
    /// New image pushes from the bottom, the old one drifts up slower.
    Up,
    /// New image pushes from the left, the old one drifts right slower.
    Left,
    /// New image pushes from the right, the old one drifts left slower.
    Right,
    /// Horizontal push where the old image follows the new direction instead of lagging.
    Invert,
}

/// `parallax*` transitions: the new image slides in while the old one moves at a different speed,
/// giving a depth effect during the push.
struct Parallax {
    start: f64,
    seq: AnimationSequence,
    dir: ParallaxDirection,
    old: Option<Box<[u8]>>,
    step: NonZeroU8,
}

impl Parallax {
    fn new(transition: &Transition, dir: ParallaxDirection) -> Self {
        let (seq, start) = bezier_seq(transition, 0.0, 1.0);
        Self {
            start,
            seq,
            dir,
            old: Option::None,
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
        self.seq.advance_to(elapsed(self.start));
        let p = self.seq.now().min(1.0);
        let dir = self.dir;
        let old = &mut self.old;
        let channels = pixel_format.channels() as usize;

        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let stride = width * channels;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                if old.as_ref().is_none_or(|o| o.len() != canvas.len()) {
                    *old = Some(canvas.to_vec().into_boxed_slice());
                }
                let previous = old.as_ref().expect("just filled");

                match dir {
                    ParallaxDirection::Up => {
                        let split = ((height as f32) * (1.0 - p)).clamp(0.0, height as f32) as usize;
                        let shift = (height as f32 * 0.35 * p) as usize;
                        for y in 0..split {
                            let src_y = (y + shift).min(height - 1);
                            let src = src_y * stride;
                            let dst = y * stride;
                            canvas[dst..dst + stride]
                                .copy_from_slice(&previous[src..src + stride]);
                        }
                        for y in split..height {
                            let src_y = y - split;
                            let src = src_y * stride;
                            let dst = y * stride;
                            canvas[dst..dst + stride].copy_from_slice(&img[src..src + stride]);
                        }
                    }
                    ParallaxDirection::Right | ParallaxDirection::Invert => {
                        let split = ((width as f32) * (1.0 - p)).clamp(0.0, width as f32) as usize;
                        let shift = (width as f32 * 0.35 * p) as usize;
                        for y in 0..height {
                            let row = y * stride;
                            for x in 0..split {
                                let src_x = if matches!(dir, ParallaxDirection::Right) {
                                    (x + shift).min(width - 1)
                                } else {
                                    x.saturating_sub(shift)
                                };
                                let src = row + src_x * channels;
                                let dst = row + x * channels;
                                canvas[dst..dst + channels]
                                    .copy_from_slice(&previous[src..src + channels]);
                            }
                            for x in split..width {
                                let src = row + (x - split) * channels;
                                let dst = row + x * channels;
                                canvas[dst..dst + channels].copy_from_slice(&img[src..src + channels]);
                            }
                        }
                    }
                    ParallaxDirection::Left => {
                        let split = ((width as f32) * p).clamp(0.0, width as f32) as usize;
                        let shift = (width as f32 * 0.35 * p) as usize;
                        for y in 0..height {
                            let row = y * stride;
                            for x in 0..split {
                                let src_x = x + (width - split);
                                let src = row + src_x * channels;
                                let dst = row + x * channels;
                                canvas[dst..dst + channels].copy_from_slice(&img[src..src + channels]);
                            }
                            for x in split..width {
                                let src_x = x.saturating_sub(shift);
                                let src = row + src_x * channels;
                                let dst = row + x * channels;
                                canvas[dst..dst + channels]
                                    .copy_from_slice(&previous[src..src + channels]);
                            }
                        }
                    }
                }
            });
        }

        self.seq.finished()
    }
}

/// `melt` transition: ragged vertical drips reveal the new image from the top, each column moving
/// at its own speed.
struct Melt {
    start: f64,
    duration: f64,
    thresholds: Box<[u8]>,
    step: NonZeroU8,
}

impl Melt {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let (width, height) = (dimensions.0 as usize, dimensions.1 as usize);
        let mut rng = seed(dimensions);
        let speeds: Vec<f32> = (0..width).map(|_| 0.7 + rand_unit(&mut rng) * 0.7).collect();
        let mut thresholds = vec![255u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let normalized = y as f32 / height.max(1) as f32;
                let threshold = (normalized / speeds[x]).clamp(0.0, 1.0);
                thresholds[y * width + x] = (threshold * 255.0) as u8;
            }
        }
        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            thresholds: thresholds.into_boxed_slice(),
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
        let t = (elapsed(self.start) / self.duration).clamp(0.0, 1.0);
        let progress = (t * 255.0) as u8;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                reveal_by_threshold(canvas, img, &self.thresholds, channels, progress);
            });
        }
        t >= 1.0
    }
}

/// `shatter` transition: random tiles fly in with a scale/rotation animation until the whole image
/// is in place, like broken glass reassembling.
struct Shatter {
    start: f64,
    duration: f64,
    tile_size: usize,
    cols: usize,
    rows: usize,
    order: Box<[u32]>,
    angles: Box<[f32]>,
    anim: Box<[f32]>,
    done: Box<[bool]>,
    revealed: usize,
    last: f64,
    step: NonZeroU8,
}

const SHATTER_ANIM: f64 = 0.35;

impl Shatter {
    fn new(transition: &Transition, dimensions: (u32, u32)) -> Self {
        let tile_size = 64usize;
        let cols = (dimensions.0 as usize).div_ceil(tile_size);
        let rows = (dimensions.1 as usize).div_ceil(tile_size);
        let total = cols * rows;

        let mut rng = seed(dimensions);
        let mut order: Vec<u32> = (0..total as u32).collect();
        for i in (1..total).rev() {
            let j = rand_range(&mut rng, i + 1);
            order.swap(i, j);
        }
        let angles: Vec<f32> = (0..total).map(|_| (rand_unit(&mut rng) - 0.5) * 1.2).collect();

        Self {
            start: now_f64(),
            duration: f64::from(transition.duration).max(0.001),
            tile_size,
            cols,
            rows,
            order: order.into_boxed_slice(),
            angles: angles.into_boxed_slice(),
            anim: vec![-1.0f32; total].into_boxed_slice(),
            done: vec![false; total].into_boxed_slice(),
            revealed: 0,
            last: now_f64(),
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
        let elapsed = elapsed(self.start);
        let t = (elapsed / self.duration).clamp(0.0, 1.0);
        // Reveal the tiles early enough that their own animation also fits in the duration.
        let reveal_window = (self.duration - SHATTER_ANIM).max(0.05);
        let reveal_t = (elapsed / reveal_window).clamp(0.0, 1.0);
        let target = (reveal_t * self.order.len() as f64) as usize;

        let now = now_f64();
        let dt = (now - self.last).clamp(0.0, 0.1);
        self.last = now;

        for i in self.revealed..target {
            let tile = self.order[i] as usize;
            self.anim[tile] = 0.0;
        }
        self.revealed = target;

        // Advance the per-tile animation once, then draw every wallpaper with the same state.
        let mut finished: Vec<usize> = Vec::new();
        for tile in 0..self.anim.len() {
            if self.anim[tile] >= 0.0 && self.anim[tile] < 1.0 {
                self.anim[tile] += (dt / SHATTER_ANIM) as f32;
                if self.anim[tile] >= 1.0 {
                    self.anim[tile] = 1.0;
                    finished.push(tile);
                }
            }
        }

        let tile_size = self.tile_size;
        let cols = self.cols;
        let rows = self.rows;
        let channels = pixel_format.channels() as usize;
        for wallpaper in wallpapers.iter() {
            let mut wallpaper = wallpaper.borrow_mut();
            let dim = wallpaper.get_dimensions();
            let width = dim.0 as usize;
            let height = dim.1 as usize;
            let stride = width * channels;
            let anim = &self.anim;
            let done = &self.done;
            let angles = &self.angles;
            wallpaper.canvas_change(backend, objman, pixel_format, |canvas| {
                for tile in 0..(cols * rows) {
                    if done[tile] {
                        continue;
                    }
                    let a = anim[tile];
                    if a < 0.0 {
                        continue;
                    }
                    let tx = tile % cols;
                    let ty = tile / cols;
                    let x0 = tx * tile_size;
                    let y0 = ty * tile_size;
                    let x1 = (x0 + tile_size).min(width);
                    let y1 = (y0 + tile_size).min(height);

                    if a >= 1.0 {
                        for y in y0..y1 {
                            let off = y * stride + x0 * channels;
                            let len = (x1 - x0) * channels;
                            canvas[off..off + len].copy_from_slice(&img[off..off + len]);
                        }
                        continue;
                    }

                    let scale = 0.55 + 0.45 * a;
                    let (sin, cos) = (-angles[tile] * (1.0 - a)).sin_cos();
                    let cx = (x0 + x1) as f32 / 2.0;
                    let cy = (y0 + y1) as f32 / 2.0;
                    let half = tile_size as f32 / 2.0;
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let dx = x as f32 - cx;
                            let dy = y as f32 - cy;
                            // inverse rotate + scale to find the source pixel
                            let rx = (dx * cos + dy * sin) / scale + cx;
                            let ry = (-dx * sin + dy * cos) / scale + cy;
                            if rx < x0 as f32 || rx >= x1 as f32 || ry < y0 as f32 || ry >= y1 as f32
                            {
                                continue;
                            }
                            if (rx - cx).abs() > half || (ry - cy).abs() > half {
                                continue;
                            }
                            let src = ry as usize * stride + rx as usize * channels;
                            let dst = y * stride + x * channels;
                            canvas[dst..dst + channels].copy_from_slice(&img[src..src + channels]);
                        }
                    }
                }
            });
        }

        for tile in finished {
            self.done[tile] = true;
        }

        // Hold the effect until every tile finished its animation, so the last ones
        // do not snap when the transition is replaced.
        let animating = self.anim.iter().any(|a| (0.0..1.0).contains(a));
        t >= 1.0 && !animating
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
