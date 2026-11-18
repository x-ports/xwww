use common::log::error;
use smallvec::SmallVec;
use waybackend::{Waybackend, objman::ObjectManager};

use rustix::time::Timespec;

use common::{
    compression::Decompressor,
    ipc::{self, BgImg, ImgReq, Nanos, PixelFormat},
    mmap::MmappedBytes,
};

use crate::{WaylandObject, wallpaper::WallpaperCell};

mod keyframe;
mod transitions;
use transitions::Effect;

pub struct Animator {
    pub wallpapers: SmallVec<[WallpaperCell; 2]>,
    now: Timespec,
    animator: AnimatorKind,
}

enum AnimatorKind {
    Transition(Transition),
    Animation(Animation),
}

fn sanitize_transition(transition: &ipc::Transition) -> ipc::Transition {
    let mut transition = transition.clone();
    if transition.fps == 0 {
        transition.fps = 30;
    }
    if !transition.duration.is_finite() || transition.duration < 0.0 {
        transition.duration = 3.0;
    }
    if !transition.angle.is_finite() {
        transition.angle = 0.0;
    }
    if !transition.wave.0.is_finite() || !transition.wave.1.is_finite() {
        transition.wave = (20.0, 20.0);
    }
    if !transition.bezier.0.is_finite()
        || !transition.bezier.1.is_finite()
        || !transition.bezier.2.is_finite()
        || !transition.bezier.3.is_finite()
    {
        transition.bezier = (0.54, 0.0, 0.34, 0.99);
    }
    let finite = |coord: &ipc::Coord| match coord {
        ipc::Coord::Pixel(value) | ipc::Coord::Percent(value) => value.is_finite(),
    };
    if !finite(&transition.pos.x) || !finite(&transition.pos.y) {
        transition.pos = ipc::Position::new(ipc::Coord::Percent(0.5), ipc::Coord::Percent(0.5));
    }
    transition
}

impl Animator {
    pub fn new(
        mut wallpapers: SmallVec<[WallpaperCell; 2]>,
        transition: &ipc::Transition,
        img_req: ImgReq,
        animation: Option<ipc::Animation>,
    ) -> Option<Self> {
        let transition = sanitize_transition(transition);
        let ImgReq { img, path, dim, .. } = img_req;
        if wallpapers.is_empty() {
            return None;
        }
        for w in &mut wallpapers {
            w.borrow_mut().set_img_info(BgImg::Img(path.str().into()));
        }

        let expect = wallpapers[0].borrow().get_dimensions();
        if dim != expect {
            error!(
                "image has wrong dimensions! Expect {}x{}, actual {}x{}",
                expect.0, expect.1, dim.0, dim.1
            );
            return None;
        }
        let effect = Some(Box::new(Effect::new(&transition, dim)));
        Some(Self {
            wallpapers,
            now: crate::clock::get(),
            animator: AnimatorKind::Transition(Transition {
                effect,
                fps_nanos: Nanos::from_nanos(1_000_000_000 / transition.fps as u64),
                img,
                animation,
            }),
        })
    }

    pub fn time_to_draw(&self) -> Timespec {
        match &self.animator {
            AnimatorKind::Transition(transition) => transition.time_to_draw(&self.now),
            AnimatorKind::Animation(animation) => animation.time_to_draw(&self.now),
        }
    }

    pub fn updt_time(&mut self) {
        self.now = crate::clock::get();
    }

    pub fn frame(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        pixel_format: PixelFormat,
    ) -> bool {
        let Self {
            wallpapers,
            animator,
            ..
        } = self;
        match animator {
            AnimatorKind::Transition(transition) => {
                if !transition.frame(backend, objman, wallpapers.as_mut_slice(), pixel_format) {
                    return false;
                }
                // Note: it needs to have more than a single frame, otherwise there is no point in
                // animating it
                if let Some(animation) = transition.animation.take()
                    && animation.animation.len() > 1
                {
                    *animator = AnimatorKind::Animation(Animation {
                        animation,
                        decompressor: Decompressor::new(),
                        i: 0,
                    });
                    return false;
                }
                true
            }
            AnimatorKind::Animation(animation) => {
                animation.frame(backend, objman, wallpapers, pixel_format);
                false
            }
        }
    }
}

struct Transition {
    fps_nanos: Nanos,
    effect: Option<Box<Effect>>,
    img: MmappedBytes,
    animation: Option<ipc::Animation>,
}

impl Transition {
    fn time_to_draw(&self, start: &Timespec) -> Timespec {
        // An already-finished effect (e.g. instant transitions) only needs the pending buffer
        // to be attached and committed, so there is nothing to wait for.
        if self.effect.is_none() {
            return Timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
        }
        let now = crate::clock::get();
        let elapsed = now - *start;
        timespec_saturating_sub(self.fps_nanos.into_timespec(), elapsed)
    }

    fn frame(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        wallpapers: &mut [WallpaperCell],
        pixel_format: PixelFormat,
    ) -> bool {
        let Self { effect, img, .. } = self;
        match effect.as_mut() {
            Some(e) => {
                let over = e.execute(backend, objman, pixel_format, wallpapers, img.bytes());
                if over {
                    *effect = None;
                }
                false
            }
            None => true,
        }
    }
}

struct Animation {
    animation: ipc::Animation,
    decompressor: Decompressor,
    i: usize,
}

impl Animation {
    fn time_to_draw(&self, start: &Timespec) -> Timespec {
        let now = crate::clock::get();
        let elapsed = now - *start;
        timespec_saturating_sub(
            self.animation.animation[self.i % self.animation.animation.len()]
                .1
                .into_timespec(),
            elapsed,
        )
    }

    fn frame(
        &mut self,
        backend: &mut Waybackend,
        objman: &mut ObjectManager<WaylandObject>,
        wallpapers: &mut SmallVec<[WallpaperCell; 2]>,
        pixel_format: PixelFormat,
    ) {
        let Self {
            animation,
            decompressor,
            i,
            ..
        } = self;

        let frame = &animation.animation[*i % animation.animation.len()].0;

        if *i < animation.animation.len() {
            wallpapers.retain(|w| {
                let mut borrow = w.borrow_mut();
                let result = borrow.canvas_change(backend, objman, pixel_format, |canvas| {
                    decompressor.decompress(frame, canvas, pixel_format)
                });
                match result {
                    Ok(()) => true,
                    Err(e) => {
                        error!("failed to unpack frame: {e}");
                        false
                    }
                }
            });
        } else {
            // if we already went through one loop, we can use the unsafe version, because
            // everything was already validated
            for w in wallpapers {
                let mut borrow = w.borrow_mut();
                // SAFETY: we have already validated every frame and removed the ones that have
                // errors in the previous loops. The only ones left should be those that can be
                // decompressed correctly
                borrow.canvas_change(backend, objman, pixel_format, |canvas| unsafe {
                    decompressor.decompress_unchecked(frame, canvas, pixel_format);
                });
            }
        }

        *i += 1;
    }
}

/// inspired by the std Duration implementation
fn timespec_saturating_sub(a: Timespec, b: Timespec) -> Timespec {
    let mut res = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    if a.tv_sec >= b.tv_sec {
        let mut secs = a.tv_sec - b.tv_sec;
        let nanos = if a.tv_nsec >= b.tv_nsec {
            a.tv_nsec - b.tv_nsec
        } else if secs > 0 {
            secs -= 1;
            a.tv_nsec + 1_000_000_000 - b.tv_nsec
        } else {
            return res;
        };
        res.tv_sec = secs;
        res.tv_nsec = nanos;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_invalid_transition_values() {
        let transition = ipc::Transition {
            transition_type: ipc::TransitionType::Ink,
            duration: f32::NAN,
            step: core::num::NonZeroU8::new(90).unwrap(),
            fps: 0,
            angle: f64::INFINITY,
            pos: ipc::Position::new(ipc::Coord::Percent(f32::NAN), ipc::Coord::Percent(0.5)),
            bezier: (f32::NAN, 0.0, 0.34, 0.99),
            wave: (f32::INFINITY, 20.0),
            invert_y: false,
        };
        let sanitized = sanitize_transition(&transition);
        assert_eq!(sanitized.fps, 30);
        assert_eq!(sanitized.duration, 3.0);
        assert_eq!(sanitized.angle, 0.0);
        assert_eq!(sanitized.wave, (20.0, 20.0));
        assert_eq!(sanitized.bezier, (0.54, 0.0, 0.34, 0.99));
        assert!(matches!(sanitized.pos.x, ipc::Coord::Percent(value) if value == 0.5));
    }
}
