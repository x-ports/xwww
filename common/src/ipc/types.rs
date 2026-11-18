use ::alloc::boxed::Box;
use ::alloc::string::String;
use ::alloc::vec::Vec;

use core::fmt;
use core::num::{NonZeroI32, NonZeroU8};

use rustix::io;

use crate::compression::BitPack;
use crate::mmap::Mmap;
use crate::mmap::MmappedBytes;
use crate::mmap::MmappedStr;

use super::ImageRequestBuilder;

#[derive(Clone, Debug, PartialEq)]
pub enum Coord {
    Pixel(f32),
    Percent(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: Coord,
    pub y: Coord,
}

impl Position {
    #[must_use]
    pub fn new(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn to_pixel(&self, dim: (u32, u32), invert_y: bool) -> (f32, f32) {
        let x = match self.x {
            Coord::Pixel(x) => x,
            Coord::Percent(x) => x * dim.0 as f32,
        };

        let y = match self.y {
            Coord::Pixel(y) => {
                if invert_y {
                    y
                } else {
                    dim.1 as f32 - y
                }
            }
            Coord::Percent(y) => {
                if invert_y {
                    y * dim.1 as f32
                } else {
                    (1.0 - y) * dim.1 as f32
                }
            }
        };

        (x, y)
    }

    #[must_use]
    pub fn to_percent(&self, dim: (u32, u32)) -> (f32, f32) {
        let x = match self.x {
            Coord::Pixel(x) => x / dim.0 as f32,
            Coord::Percent(x) => x,
        };

        let y = match self.y {
            Coord::Pixel(y) => y / dim.1 as f32,
            Coord::Percent(y) => y,
        };

        (x, y)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BgImg {
    Color([u8; 4]),
    Img(Box<str>),
}

impl BgImg {
    fn serialized_size(&self) -> usize {
        1 //discriminant
        + match self {
            Self::Color(_) => 4,
            Self::Img(s) => 4 + s.len()
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Img(_))
    }
}

impl fmt::Display for BgImg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BgImg::Color(color) => {
                write!(f, "color: {:02X}{:02X}{:02X}", color[0], color[1], color[2])
            }
            BgImg::Img(p) => write!(f, "image: {p}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PixelFormat {
    /// No swap, can copy directly onto WlBuffer
    Bgr = 0,
    /// Swap R and B channels at client, can copy directly onto WlBuffer
    Rgb = 1,
    /// No swap, must extend pixel with an extra byte when displaying animations
    Abgr = 2,
    /// Swap R and B channels at client, must extend pixel with an extra byte when displaying
    /// animations
    Argb = 3,
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixelFormat::Bgr => f.write_str("Bgr"),
            PixelFormat::Rgb => f.write_str("Rgb"),
            PixelFormat::Abgr => f.write_str("Abgr"),
            PixelFormat::Argb => f.write_str("Argb"),
        }
    }
}

impl PixelFormat {
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> u8 {
        match self {
            Self::Rgb | Self::Bgr => 3,
            Self::Abgr | Self::Argb => 4,
        }
    }

    #[inline]
    #[must_use]
    pub const fn must_swap_r_and_b_channels(&self) -> bool {
        match self {
            Self::Bgr | Self::Abgr => false,
            Self::Rgb | Self::Argb => true,
        }
    }

    #[inline]
    #[must_use]
    pub const fn can_copy_directly_onto_wl_buffer(&self) -> bool {
        match self {
            Self::Bgr | Self::Rgb => true,
            Self::Abgr | Self::Argb => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Scale {
    /// sent by wl_output::scale events
    Output(NonZeroI32),
    /// sent by wl_surface::preferred_buffer_scale events
    Preferred(NonZeroI32),
    /// sent by wp_fractional_scale_v1::preferred_scale events
    Fractional(NonZeroI32),
}

impl Scale {
    #[inline]
    #[must_use]
    pub fn priority(&self) -> u32 {
        match self {
            Scale::Output(_) => 0,
            Scale::Preferred(_) => 1,
            Scale::Fractional(_) => 2,
        }
    }

    #[inline]
    #[must_use]
    pub fn mul_dim(&self, width: i32, height: i32) -> (i32, i32) {
        match self {
            Scale::Output(i) | Scale::Preferred(i) => (width * i.get(), height * i.get()),
            Scale::Fractional(f) => {
                let width = (width * f.get() + 60) / 120;
                let height = (height * f.get() + 60) / 120;
                (width, height)
            }
        }
    }

    #[must_use]
    pub fn to_f32(&self) -> f32 {
        match self {
            Scale::Output(i) | Scale::Preferred(i) => i.get() as f32,
            Scale::Fractional(f) => f.get() as f32 / 120.0,
        }
    }
}

impl PartialEq for Scale {
    fn eq(&self, other: &Self) -> bool {
        (match self {
            Self::Output(i) | Self::Preferred(i) => i.get() * 120,
            Self::Fractional(f) => f.get(),
        }) == (match other {
            Self::Output(i) | Self::Preferred(i) => i.get() * 120,
            Self::Fractional(f) => f.get(),
        })
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_f32())
    }
}

#[derive(Clone)]
pub struct BgInfo {
    pub name: Box<str>,
    pub dim: (u32, u32),
    pub scale_factor: Scale,
    pub img: BgImg,
    pub pixel_format: PixelFormat,
}

impl BgInfo {
    #[inline]
    #[must_use]
    pub fn real_dim(&self) -> (u32, u32) {
        let dim = self
            .scale_factor
            .mul_dim(self.dim.0 as i32, self.dim.1 as i32);
        (dim.0 as u32, dim.1 as u32)
    }

    pub(super) fn serialized_size(&self) -> usize {
        4 // name len
            + self.name.len()
            + 8 //dim
            + 5 //scale_factor (discriminant + value)
            + self.img.serialized_size()
            + 1 //pixel_format
    }

    pub(super) fn serialize(&self, buf: &mut [u8]) -> usize {
        let Self {
            name,
            dim,
            scale_factor,
            img,
            pixel_format,
        } = self;

        let len = name.len();
        buf[0..4].copy_from_slice(&(len as u32).to_ne_bytes());
        buf[4..4 + len].copy_from_slice(name.as_bytes());
        let mut i = 4 + len;
        buf[i..i + 4].copy_from_slice(&dim.0.to_ne_bytes());
        buf[i + 4..i + 8].copy_from_slice(&dim.1.to_ne_bytes());
        i += 8;

        match scale_factor {
            Scale::Output(value) => {
                buf[i] = 0;
                buf[i + 1..i + 5].copy_from_slice(&value.get().to_ne_bytes());
            }
            Scale::Preferred(value) => {
                buf[i] = 1;
                buf[i + 1..i + 5].copy_from_slice(&value.get().to_ne_bytes());
            }
            Scale::Fractional(value) => {
                buf[i] = 2;
                buf[i + 1..i + 5].copy_from_slice(&value.get().to_ne_bytes());
            }
        }
        i += 5;

        match img {
            BgImg::Color(color) => {
                buf[i] = 0;
                buf[i + 1..i + 5].copy_from_slice(color);
                i += 5;
            }
            BgImg::Img(path) => {
                buf[i] = 1;
                i += 1;
                let len = path.len();
                buf[i..i + 4].copy_from_slice(&(len as u32).to_ne_bytes());
                buf[i + 4..i + 4 + len].copy_from_slice(path.as_bytes());
                i += 4 + len;
            }
        }

        buf[i] = *pixel_format as u8;
        i + 1
    }

    pub(super) fn deserialize(bytes: &[u8]) -> Option<(Self, usize)> {
        let name = deserialize_boxed_str(bytes)?;
        let mut i = name.len() + 4;

        let dim = (
            u32::from_ne_bytes(bytes.get(i..i + 4)?.try_into().ok()?),
            u32::from_ne_bytes(bytes.get(i + 4..i + 8)?.try_into().ok()?),
        );
        i += 8;

        let scale = bytes.get(i..i + 5)?;
        let scale_value = i32::from_ne_bytes(scale.get(1..5)?.try_into().ok()?);
        let scale_value = NonZeroI32::new(scale_value)?;
        let scale_factor = match scale[0] {
            0 => Scale::Output(scale_value),
            1 => Scale::Preferred(scale_value),
            2 => Scale::Fractional(scale_value),
            _ => return None,
        };
        i += 5;

        let img = match *bytes.get(i)? {
            0 => {
                let color = bytes.get(i + 1..i + 5)?;
                i += 5;
                BgImg::Color([color[0], color[1], color[2], color[3]])
            }
            1 => {
                i += 1;
                let path = deserialize_boxed_str(bytes.get(i..)?)?;
                i += 4 + path.len();
                BgImg::Img(path)
            }
            _ => return None,
        };

        let pixel_format = match *bytes.get(i)? {
            0 => PixelFormat::Bgr,
            1 => PixelFormat::Rgb,
            2 => PixelFormat::Abgr,
            3 => PixelFormat::Argb,
            _ => return None,
        };
        i += 1;

        Some((
            Self {
                name,
                dim,
                scale_factor,
                img,
                pixel_format,
            },
            i,
        ))
    }
}

impl fmt::Display for BgInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}x{}, scale: {}, currently displaying: {}",
            self.name, self.dim.0, self.dim.1, self.scale_factor, self.img
        )
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum TransitionType {
    Simple = 0,
    Fade = 1,
    Outer = 2,
    Wipe = 3,
    Grow = 4,
    Wave = 5,
    None = 6,
    Glitch = 7,
    Decrypt = 8,
    Dissolve = 9,
    Clock = 10,
    Zoom = 11,
    Pixelate = 12,
    Ripple = 13,
    Blinds = 14,
    Spiral = 15,
    Static = 16,
    Parallax = 17,
    ParallaxLeft = 18,
    ParallaxRight = 19,
    ParallaxInvert = 20,
    Melt = 21,
    Shatter = 22,
    Ink = 23,
    Paint = 24,
    Contagion = 25,
    Waterdrop = 26,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub transition_type: TransitionType,
    pub duration: f32,
    pub step: NonZeroU8,
    pub fps: u16,
    pub angle: f64,
    pub pos: Position,
    pub bezier: (f32, f32, f32, f32),
    pub wave: (f32, f32),
    pub invert_y: bool,
}

impl Transition {
    pub(super) fn serialize(&self, buf: &mut ImageRequestBuilder) {
        let Self {
            transition_type,
            duration,
            step,
            fps,
            angle,
            pos,
            bezier,
            wave,
            invert_y,
        } = self;

        buf.push_byte(*transition_type as u8);
        buf.extend(&duration.to_ne_bytes());
        buf.push_byte(step.get());
        buf.extend(&fps.to_ne_bytes());
        buf.extend(&angle.to_ne_bytes());
        match pos.x {
            Coord::Pixel(f) => {
                buf.push_byte(0);
                buf.extend(&f.to_ne_bytes());
            }
            Coord::Percent(f) => {
                buf.push_byte(1);
                buf.extend(&f.to_ne_bytes());
            }
        }
        match pos.y {
            Coord::Pixel(f) => {
                buf.push_byte(0);
                buf.extend(&f.to_ne_bytes());
            }
            Coord::Percent(f) => {
                buf.push_byte(1);
                buf.extend(&f.to_ne_bytes());
            }
        }
        buf.extend(&bezier.0.to_ne_bytes());
        buf.extend(&bezier.1.to_ne_bytes());
        buf.extend(&bezier.2.to_ne_bytes());
        buf.extend(&bezier.3.to_ne_bytes());
        buf.extend(&wave.0.to_ne_bytes());
        buf.extend(&wave.1.to_ne_bytes());
        buf.push_byte(*invert_y as u8);
    }

    pub(super) fn deserialize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 51 {
            return None;
        }
        let transition_type = match bytes[0] {
            0 => TransitionType::Simple,
            1 => TransitionType::Fade,
            2 => TransitionType::Outer,
            3 => TransitionType::Wipe,
            4 => TransitionType::Grow,
            5 => TransitionType::Wave,
            6 => TransitionType::None,
            7 => TransitionType::Glitch,
            8 => TransitionType::Decrypt,
            9 => TransitionType::Dissolve,
            10 => TransitionType::Clock,
            11 => TransitionType::Zoom,
            12 => TransitionType::Pixelate,
            13 => TransitionType::Ripple,
            14 => TransitionType::Blinds,
            15 => TransitionType::Spiral,
            16 => TransitionType::Static,
            17 => TransitionType::Parallax,
            18 => TransitionType::ParallaxLeft,
            19 => TransitionType::ParallaxRight,
            20 => TransitionType::ParallaxInvert,
            21 => TransitionType::Melt,
            22 => TransitionType::Shatter,
            23 => TransitionType::Ink,
            24 => TransitionType::Paint,
            25 => TransitionType::Contagion,
            26 => TransitionType::Waterdrop,
            _ => return None,
        };
        let duration = f32::from_ne_bytes(bytes[1..5].try_into().ok()?);
        if !duration.is_finite() || duration < 0.0 {
            return None;
        }
        let step = NonZeroU8::new(bytes[5])?;
        let fps = u16::from_ne_bytes(bytes[6..8].try_into().ok()?);
        if fps == 0 {
            return None;
        }
        let angle = f64::from_ne_bytes(bytes[8..16].try_into().ok()?);
        if !angle.is_finite() {
            return None;
        }
        let pos = {
            let x_value = f32::from_ne_bytes(bytes[17..21].try_into().ok()?);
            let y_value = f32::from_ne_bytes(bytes[22..26].try_into().ok()?);
            if !x_value.is_finite() || !y_value.is_finite() {
                return None;
            }
            let x = if bytes[16] == 0 {
                Coord::Pixel(x_value)
            } else {
                Coord::Percent(x_value)
            };
            let y = if bytes[21] == 0 {
                Coord::Pixel(y_value)
            } else {
                Coord::Percent(y_value)
            };
            Position { x, y }
        };

        let bezier = (
            f32::from_ne_bytes(bytes[26..30].try_into().ok()?),
            f32::from_ne_bytes(bytes[30..34].try_into().ok()?),
            f32::from_ne_bytes(bytes[34..38].try_into().ok()?),
            f32::from_ne_bytes(bytes[38..42].try_into().ok()?),
        );
        if !bezier.0.is_finite()
            || !bezier.1.is_finite()
            || !bezier.2.is_finite()
            || !bezier.3.is_finite()
        {
            return None;
        }

        let wave = (
            f32::from_ne_bytes(bytes[42..46].try_into().ok()?),
            f32::from_ne_bytes(bytes[46..50].try_into().ok()?),
        );
        if !wave.0.is_finite() || !wave.1.is_finite() {
            return None;
        }

        let invert_y = bytes[50] != 0;

        Some(Self {
            transition_type,
            duration,
            step,
            fps,
            angle,
            pos,
            bezier,
            wave,
            invert_y,
        })
    }
}

pub struct ClearSend {
    pub color: [u8; 4],
    pub outputs: Box<[String]>,
}

impl ClearSend {
    pub fn create_request(self) -> io::Result<Mmap> {
        // 1 - output length
        // 4 - color bytes
        // 4 + output.len() - output len + bytes
        let len = 5 + self.outputs.iter().map(|o| 4 + o.len()).sum::<usize>();
        let mut mmap = Mmap::create(len)?;
        let bytes = mmap.slice_mut();
        // we assume someone does not have more than
        // 255 monitors. Seems reasonable
        bytes[0] = self.outputs.len() as u8;
        let mut i = 1;
        for output in self.outputs {
            let len = output.len() as u32;
            bytes[i..i + 4].copy_from_slice(&len.to_ne_bytes());
            bytes[i + 4..i + 4 + len as usize].copy_from_slice(output.as_bytes());
            i += 4 + len as usize;
        }
        bytes[i..i + 4].copy_from_slice(&self.color);
        Ok(mmap)
    }
}

pub struct ClearReq {
    pub color: [u8; 4],
    pub outputs: Box<[MmappedStr]>,
}

/// Request body for the `screenshot` command. It carries the name of the output to capture, or an
/// empty string to capture the first available output.
pub struct ScreenshotReq {
    pub output: Box<str>,
}

impl ScreenshotReq {
    pub fn create_request(self) -> io::Result<Mmap> {
        let len = 4 + self.output.len();
        let mut mmap = Mmap::create(len)?;
        let bytes = mmap.slice_mut();
        let n = self.output.len() as u32;
        bytes[0..4].copy_from_slice(&n.to_ne_bytes());
        bytes[4..4 + self.output.len()].copy_from_slice(self.output.as_bytes());
        Ok(mmap)
    }
}

/// Answer body for the `screenshot` command. Contains the captured wallpaper, always normalized to
/// standard RGB (3 bytes per pixel, no alpha).
pub struct ScreenshotData {
    pub width: u32,
    pub height: u32,
    pub rgb: Box<[u8]>,
}

pub struct ImgSend {
    pub path: String,
    pub dim: (u32, u32),
    pub format: PixelFormat,
    pub img: Box<[u8]>,
}

pub struct ImgReq {
    pub path: MmappedStr,
    pub dim: (u32, u32),
    pub format: PixelFormat,
    pub img: MmappedBytes,
}

impl ImgReq {
    pub(super) fn deserialize(mmap: &Mmap, bytes: &[u8]) -> Option<(Self, usize)> {
        let mut i = 0;
        let path = MmappedStr::new_checked(mmap, bytes.get(i..)?)?;
        i += 4 + path.str().len();

        let img = MmappedBytes::new_checked(mmap, bytes.get(i..)?)?;
        i += 4 + img.bytes().len();

        let dim_bytes = bytes.get(i..i + 8)?;
        let dim = (
            u32::from_ne_bytes(dim_bytes.get(0..4)?.try_into().ok()?),
            u32::from_ne_bytes(dim_bytes.get(4..8)?.try_into().ok()?),
        );
        i += 8;

        let format = match *bytes.get(i)? {
            0 => PixelFormat::Bgr,
            1 => PixelFormat::Rgb,
            2 => PixelFormat::Abgr,
            3 => PixelFormat::Argb,
            _ => return None,
        };
        i += 1;

        Some((
            Self {
                path,
                dim,
                format,
                img,
            },
            i,
        ))
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Nanos(u64);

impl Nanos {
    #[inline]
    pub fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    #[inline]
    pub fn from_millis(millis: u64) -> Self {
        Self(millis * 1_000_000)
    }

    #[inline]
    pub fn into_timespec(self) -> rustix::time::Timespec {
        let secs = (self.0 / 1_000_000_000) as rustix::time::Secs;
        let nsecs = (self.0 % 1_000_000_000) as rustix::time::Nsecs;
        rustix::time::Timespec {
            tv_sec: secs,
            tv_nsec: nsecs,
        }
    }
}

impl core::ops::AddAssign for Nanos {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

pub struct Animation {
    pub animation: Box<[(BitPack, Nanos)]>,
}

impl Animation {
    pub(crate) fn serialize(&self, buf: &mut ImageRequestBuilder) {
        let Self { animation } = self;

        buf.extend(&(animation.len() as u32).to_ne_bytes());
        for (bitpack, duration) in animation {
            bitpack.serialize(buf);
            buf.extend(&duration.0.to_ne_bytes());
        }
    }

    pub(crate) fn deserialize(mmap: &Mmap, bytes: &[u8]) -> Option<(Self, usize)> {
        let mut i = 0;
        let animation_len = u32::from_ne_bytes(bytes.get(i..i + 4)?.try_into().unwrap()) as usize;
        i += 4;
        let mut animation = Vec::with_capacity(animation_len);
        for _ in 0..animation_len {
            let (anim, offset) = BitPack::deserialize(mmap, bytes.get(i..)?)?;
            i += offset;
            let duration = Nanos(u64::from_ne_bytes(bytes.get(i..i + 8)?.try_into().unwrap()));
            i += 8;
            animation.push((anim, duration));
        }

        Some((
            Self {
                animation: animation.into(),
            },
            i,
        ))
    }
}

pub struct ImageReq {
    pub transition: Transition,
    pub imgs: Vec<ImgReq>,
    pub outputs: Vec<Box<[MmappedStr]>>,
    pub animations: Option<Vec<Animation>>,
}

fn deserialize_boxed_str(bytes: &[u8]) -> Option<Box<str>> {
    let size = u32::from_ne_bytes(bytes.get(0..4)?.try_into().ok()?) as usize;
    let end = 4usize.checked_add(size)?;
    core::str::from_utf8(bytes.get(4..end)?)
        .ok()
        .map(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition_bytes(transition_type: u8) -> [u8; 51] {
        let mut bytes = [0u8; 51];
        bytes[0] = transition_type;
        bytes[1..5].copy_from_slice(&3.0f32.to_ne_bytes());
        bytes[5] = 90;
        bytes[6..8].copy_from_slice(&30u16.to_ne_bytes());
        bytes[8..16].copy_from_slice(&45.0f64.to_ne_bytes());
        bytes[16] = 1;
        bytes[17..21].copy_from_slice(&0.5f32.to_ne_bytes());
        bytes[21] = 1;
        bytes[22..26].copy_from_slice(&0.5f32.to_ne_bytes());
        for (index, value) in [0.54f32, 0.0, 0.34, 0.99].iter().enumerate() {
            let start = 26 + index * 4;
            bytes[start..start + 4].copy_from_slice(&value.to_ne_bytes());
        }
        bytes[42..46].copy_from_slice(&20.0f32.to_ne_bytes());
        bytes[46..50].copy_from_slice(&20.0f32.to_ne_bytes());
        bytes[50] = 0;
        bytes
    }

    #[test]
    fn deserializes_new_transition_types() {
        for (kind, expected) in [
            (23, "ink"),
            (24, "paint"),
            (25, "contagion"),
            (26, "waterdrop"),
        ] {
            let transition = Transition::deserialize(&transition_bytes(kind)).unwrap();
            let matched = match transition.transition_type {
                TransitionType::Ink => "ink",
                TransitionType::Paint => "paint",
                TransitionType::Contagion => "contagion",
                TransitionType::Waterdrop => "waterdrop",
                _ => "other",
            };
            assert_eq!(matched, expected);
        }
    }

    #[test]
    fn rejects_malformed_transitions() {
        assert!(Transition::deserialize(&transition_bytes(0)[..50]).is_none());

        let mut bytes = transition_bytes(0);
        bytes[5] = 0;
        assert!(Transition::deserialize(&bytes).is_none());

        let mut bytes = transition_bytes(0);
        bytes[6..8].copy_from_slice(&0u16.to_ne_bytes());
        assert!(Transition::deserialize(&bytes).is_none());

        let mut bytes = transition_bytes(0);
        bytes[1..5].copy_from_slice(&f32::NAN.to_ne_bytes());
        assert!(Transition::deserialize(&bytes).is_none());

        assert!(Transition::deserialize(&transition_bytes(200)).is_none());
    }

    #[test]
    fn rejects_truncated_strings() {
        assert!(deserialize_boxed_str(&[]).is_none());
        assert!(deserialize_boxed_str(&[1, 0, 0, 0]).is_none());
        assert!(deserialize_boxed_str(&[1, 0, 0, 0, b'a']).is_some());
        assert!(deserialize_boxed_str(&[2, 0, 0, 0, b'a']).is_none());
    }
}
