/// Note: this file only has basic declarations and some definitions in order to be possible to
/// import it in the build script, to automate shell completion
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

fn from_hex(hex: &str) -> Result<[u8; 4], String> {
    if hex.len() != 6 && hex.len() != 8 {
        return Err(format!(
            "expected 6 or 8 characters in hexadecimal, found {hex}",
        ));
    }

    let mut color = [0, 0, 0, 255];

    for i in (0..hex.len()).step_by(2) {
        color[i / 2] = match u8::from_str_radix(&hex[i..i + 2], 16) {
            Ok(color) => color,
            Err(_) => {
                return Err(format!(
                    "expected [0-9], [a-f], or [A-F], found '{}'",
                    &hex[i..i + 2]
                ));
            }
        }
    }

    Ok(color)
}

#[derive(Clone, ValueEnum)]
pub enum PixelFormat {
    /// No swap, can copy directly onto WlBuffer
    Bgr,
    /// Swap R and B channels at client, can copy directly onto WlBuffer
    Rgb,
    /// No swap, must extend pixel with an extra byte when displaying animations
    Abgr,
    /// Swap R and B channels at client, must extend pixel with an extra byte when displaying
    /// animations
    Argb,
}

#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Bilinear,
    CatmullRom,
    Mitchell,
    Lanczos3,
}

impl Filter {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nearest => "Nearest",
            Self::Bilinear => "Bilinear",
            Self::CatmullRom => "CatmullRom",
            Self::Mitchell => "Mitchell",
            Self::Lanczos3 => "Lanczos3",
        }
    }
}

impl std::str::FromStr for Filter {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Nearest" => Ok(Self::Nearest),
            "Bilinear" => Ok(Self::Bilinear),
            "CatmullRom" => Ok(Self::CatmullRom),
            "Mitchell" => Ok(Self::Mitchell),
            "Lanczos3" => Ok(Self::Lanczos3),
            _ => Err("unrecognized filter. Valid filters are:\n\
                     \tNearest | Bilinear | CatmullRom | Mitchell | Lanczos3\n\
                     see xwww img --help for more details"),
        }
    }
}

#[derive(Clone)]
pub enum TransitionType {
    None,
    Simple,
    Fade,
    Left,
    Right,
    Top,
    Bottom,
    Center,
    Outer,
    Any,
    Random,
    Wipe,
    Wave,
    Grow,
    Glitch,
    Decrypt,
    Dissolve,
    Pixelate,
    Ripple,
    Blinds,
    Spiral,
    Static,
    Parallax,
    ParallaxLeft,
    ParallaxRight,
    ParallaxInvert,
    Melt,
    Shatter,
    Clock,
    Zoom,
}

impl std::str::FromStr for TransitionType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "simple" => Ok(Self::Simple),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "top" => Ok(Self::Top),
            "bottom" => Ok(Self::Bottom),
            "wipe" => Ok(Self::Wipe),
            "grow" => Ok(Self::Grow),
            "center" => Ok(Self::Center),
            "outer" => Ok(Self::Outer),
            "any" => Ok(Self::Any),
            "wave" => Ok(Self::Wave),
            "random" => Ok(Self::Random),
            "fade" => Ok(Self::Fade),
            "glitch" => Ok(Self::Glitch),
            "decrypt" => Ok(Self::Decrypt),
            "dissolve" => Ok(Self::Dissolve),
            "clock" => Ok(Self::Clock),
            "zoom" => Ok(Self::Zoom),
            "pixelate" => Ok(Self::Pixelate),
            "ripple" => Ok(Self::Ripple),
            "blinds" => Ok(Self::Blinds),
            "spiral" => Ok(Self::Spiral),
            "static" => Ok(Self::Static),
            "parallax" => Ok(Self::Parallax),
            "parallax-left" => Ok(Self::ParallaxLeft),
            "parallax-right" => Ok(Self::ParallaxRight),
            "parallax-invert" => Ok(Self::ParallaxInvert),
            "melt" => Ok(Self::Melt),
            "shatter" => Ok(Self::Shatter),
            _ => Err("unrecognized transition type.\nValid transitions are:\n\
                     \tsimple | fade | left | right | top | bottom | wipe | grow | center | outer | random | wave | glitch | decrypt | dissolve | clock | zoom | pixelate | ripple | blinds | spiral | static | parallax | parallax-left | parallax-right | parallax-invert | melt | shatter\n\
                     see xwww img --help for more details"),
        }
    }
}

#[derive(Clone)]
pub enum CliCoord {
    Percent(f32),
    Pixel(f32),
}

#[derive(Clone)]
pub struct CliPosition {
    pub x: CliCoord,
    pub y: CliCoord,
    //Unknown(f32, f32),
}

impl CliPosition {
    #[must_use]
    pub fn new(x: CliCoord, y: CliCoord) -> Self {
        Self { x, y }
    }
}

#[derive(Clone)]
pub enum CliImage {
    Path(PathBuf),
    /// Single rgb color
    Color([u8; 4]),
}

#[derive(Parser)]
#[command(version, name = "xwww")]
///A Wayland wallpaper daemon, controlled at runtime.
///
///Change what your monitors display as a background by controlling the xwww daemon at runtime.
///Supports animated gifs and putting different stuff in different monitors. I also did my best to
///make it as resource efficient as possible.
///
///Note `xwww` will only work in a compositor that implements the layer-shell protocol. Typically,
///wlr-roots based compositors.
pub enum Xwww {
    ///Fills the specified outputs with the given color.
    ///
    ///Defaults to filling all outputs with black.
    Clear(Clear),

    ///Restores the last displayed image on the specified outputs.
    Restore(Restore),

    ///Clears the xwww cache.
    ///
    ///We currently store the address of the last file set as wallpaper for each monitor, as well
    ///as the animation frames of every gif ever set for a given version of `xwww`.
    ClearCache,

    /// Sends an image (or animated gif) for the daemon to display.
    ///
    /// Use `-` to read from stdin
    Img(Img),

    ///Toggles the daemon
    Toggle(Toggle),

    ///Pauses the daemon
    Pause(Pause),

    ///Unpauses the daemon
    Unpause(Unpause),

    ///Kills the daemon
    Kill(Kill),

    ///Asks the daemon to print output information (names and dimensions).
    ///
    ///You may use this to find out valid values for the <xwww-img --outputs> option. If you want
    ///more detailed information about your outputs, I would recommend trying wlr-randr.
    Query(Query),

    ///Extracts a color palette (dominant colors) from an image or from the current wallpaper.
    Palette(Palette),

    ///Cycles through the images in a directory on a timer.
    Slideshow(Slideshow),

    ///Sets a single random image from a directory.
    Random(Random),

    ///Captures the currently displayed wallpaper and saves it as an image file.
    Screenshot(Screenshot),

    /// Renders a JavaScript scene as the wallpaper (requires the `scene` feature).
    ///
    /// A scene is a JavaScript file with optional `setup(ctx)` and `render(t, ctx)` functions
    /// that draw with a small canvas API and can read the active color palette.
    Scene(Scene),
}

#[derive(Parser)]
pub struct Clear {
    /// Clear all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// Color to fill the screen with.
    ///
    /// Must be given in rrggbb format (note there is no prepended '#').
    #[arg(value_parser = from_hex, default_value = "000000ff")]
    pub color: [u8; 4],

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,

    /// Comma separated list of outputs to display the image at.
    ///
    /// If it isn't set, the image is displayed on all outputs.
    #[clap(short, long, default_value = "")]
    pub outputs: String,
}

#[derive(Parser)]
pub struct Toggle {
    /// Toggle all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Pause {
    /// Pause all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Unpause {
    /// Unpause all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Kill {
    /// Kill all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Query {
    /// Query all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// Print the information in `json` format
    #[arg(short, long, default_value = "false")]
    pub json: bool,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum ResizeStrategy {
    /// Do not resize the image
    ///
    /// If this is set, the image won't be resized, and will be centralized in the middle of the
    /// screen instead. If it is smaller than the screen's size, it will be padded with the value
    /// of `fill_color`, below.
    No,
    #[default]
    /// Resize the image to fill the whole screen, cropping out parts that don't fit
    Crop,
    /// Resize the image to fit inside the screen, preserving the original aspect ratio
    Fit,
    /// Resize the image to fit inside the screen, without preserving the original aspect ratio
    Stretch,
}

impl ResizeStrategy {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ResizeStrategy::No => "no",
            ResizeStrategy::Crop => "crop",
            ResizeStrategy::Fit => "fit",
            ResizeStrategy::Stretch => "stretch",
        }
    }
}

impl std::str::FromStr for ResizeStrategy {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "no" => Ok(Self::No),
            "crop" => Ok(Self::Crop),
            "fit" => Ok(Self::Fit),
            "stretch" => Ok(Self::Stretch),
            _ => Err(
                "unrecognized resize strategy. Valid resize strategies are:\n\
                     no | crop | fit | stretch\n\
                     see xwww img --help for more details",
            ),
        }
    }
}

#[derive(Default, Clone, Copy, ValueEnum)]
pub enum CropGravity {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl CropGravity {
    pub fn as_centering_tuple(self) -> (f64, f64) {
        match self {
            CropGravity::TopLeft => (0.0, 0.0),
            CropGravity::Top => (0.5, 0.0),
            CropGravity::TopRight => (1.0, 0.0),
            CropGravity::Left => (0.0, 0.5),
            CropGravity::Center => (0.5, 0.5),
            CropGravity::Right => (1.0, 0.5),
            CropGravity::BottomLeft => (0.0, 1.0),
            CropGravity::Bottom => (0.5, 1.0),
            CropGravity::BottomRight => (1.0, 1.0),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            CropGravity::TopLeft => "top-left",
            CropGravity::Top => "top",
            CropGravity::TopRight => "top-right",
            CropGravity::Left => "left",
            CropGravity::Center => "center",
            CropGravity::Right => "right",
            CropGravity::BottomLeft => "bottom-left",
            CropGravity::Bottom => "bottom",
            CropGravity::BottomRight => "bottom-right",
        }
    }
}

impl std::str::FromStr for CropGravity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "top-left" => Ok(Self::TopLeft),
            "top" => Ok(Self::Top),
            "top-right" => Ok(Self::TopRight),
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom" => Ok(Self::Bottom),
            "bottom-right" => Ok(Self::BottomRight),
            _ => Err(format!(
                "Unrecognized crop gravity '{}'. Valid crop gravity values are:\n\
                     top-left | top | top-right | left | center | right | bottom-left | bottom | bottom-right\n\
                     see xwww img --help for more details",
                s
            )),
        }
    }
}

#[derive(Parser)]
pub struct Restore {
    /// Restore all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,
    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,

    /// Comma separated list of outputs to restore.
    ///
    /// If it isn't set, all outputs will be restored.
    #[arg(short, long, default_value = "")]
    pub outputs: String,
}

#[derive(Parser)]
pub struct Img {
    /// Set the image for all xwww-daemon instances (all namespaces)
    #[arg(short, long, default_value = "false")]
    pub all: bool,

    /// Path of image or hexcode (starting with 0x) to display
    #[arg(value_parser = parse_image)]
    pub image: CliImage,

    /// Comma separated list of outputs to display the image at.
    ///
    /// If it isn't set, the image is displayed on all outputs.
    #[arg(short, long, default_value = "")]
    pub outputs: String,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    ///
    /// You can specify multiple namespaces at once with multiple `--namespaces` arguments. The
    /// commands will be sent to every namespace you have specified.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,

    /// Do not update the cache.
    #[arg(long, default_value = "false")]
    pub no_cache: bool,

    /// Do not resize the image. Equivalent to `--resize=no`
    ///
    /// If this is set, the image won't be resized, and will be centralized in the middle of the
    /// screen instead. If it is smaller than the screen's size, it will be padded with the value
    /// of `fill_color`, below.
    #[deprecated(since = "0.7.3", note = "use `resize` instead")]
    #[arg(long)]
    pub no_resize: bool,

    /// Whether to resize the image and the method by which to resize it
    #[arg(
        long,
        default_value = "crop",
        default_value_if("no_resize", "true", "no")
    )]
    pub resize: ResizeStrategy,

    /// Specify which portion of the image to anchor when cropping. Only used when `--resize crop`
    /// is specified.
    #[arg(long, requires("resize"), default_value_if("resize", "crop", "center"))]
    pub crop_gravity: Option<CropGravity>,

    /// Which color to fill the padding with when output image does not fill screen
    #[arg(value_parser = from_hex, long, default_value = "000000ff")]
    pub fill_color: [u8; 4],

    ///Filter to use when scaling images (run xwww img --help to see options).
    ///
    ///Available options are:
    ///
    ///Nearest | Bilinear | CatmullRom | Mitchell | Lanczos3
    ///
    ///These are offered by the fast_image_resize crate
    ///(https://docs.rs/fast_image_resize/2.5.0/fast_image_resize/). 'Nearest' is
    ///what I recommend for pixel art stuff, and ONLY for pixel art stuff. It is also the
    ///fastest filter.
    ///
    ///For non pixel art stuff, I would usually recommend one of the last three, though some
    ///experimentation will be necessary to see which one you like best. Also note they are
    ///all slower than Nearest.
    #[arg(short, long, default_value = "Lanczos3")]
    pub filter: Filter,

    /// Gaussian blur radius (in pixels) applied to the wallpaper after scaling.
    ///
    /// Set to 0 to disable. Applies only to static images; animated wallpapers are left
    /// untouched.
    #[arg(long, default_value = "0")]
    pub blur: u32,

    /// Dim factor applied to the wallpaper after scaling.
    ///
    /// A value in `[0.0, 1.0]`, where `1.0` leaves the image unchanged and `0.0` turns it fully
    /// black. Applies only to static images.
    #[arg(long, default_value = "1.0")]
    pub dim: f32,

    /// Recolors the wallpaper with the colors of a palette (gradient map over luminance).
    ///
    /// Available sources:
    ///
    /// `xwww` / `xwww:<path>` reads the xwww palette file (`~/.config/xwww/palette.json` by
    /// default), which the desktop is expected to write.
    ///
    /// `equisdots` / `equisdots:<slug>` reads the palette of the equisdots desktop
    /// (`~/.config/hypr/settings.json` -> `bar.palette` and the `dock/palettes` directory).
    ///
    /// `file:<path>` (or a bare path) reads a palette JSON, or a text file with one `#rrggbb`
    /// color per line.
    ///
    /// `command:<cmd>` parses the stdout of a command the same way as `file`.
    ///
    /// The effect maps each pixel's luminance through the palette. Applies only to static
    /// images.
    #[arg(long, value_parser = parse_palette_spec)]
    pub map_palette: Option<String>,

    /// How strongly `--map-palette` overrides the original colors (`0.0` to `1.0`).
    ///
    /// `0.0` keeps the original image, `1.0` fully replaces it with the palette gradient.
    #[arg(long, default_value = "1.0")]
    pub map_strength: f32,

    ///Sets the type of transition. Default is 'simple', that fades into the new image
    ///
    ///Possible transitions are:
    ///
    ///none | simple | fade | left | right | top | bottom | wipe | wave | grow | center | any |
    /// outer | random | glitch | decrypt | dissolve | clock | zoom
    ///
    ///The 'left', 'right', 'top' and 'bottom' options make the transition happen from that
    ///position to its opposite in the screen.
    ///
    ///'none' is an alias to 'simple' that also sets the 'transition-step' to 255. This has the
    ///effect of the transition finishing instantly
    ///
    ///'fade' is similar to 'simple' but the fade is controlled through the --transition-bezier
    /// flag
    ///
    ///'wipe' is similar to 'left' but allows you to specify the angle for transition with the
    /// `--transition-angle` flag.
    ///
    ///'wave' is similar to 'wipe' sweeping line is wavy
    ///
    ///'grow' causes a growing circle to transition across the screen and allows changing the
    /// circle's center position with the `--transition-pos` flag.
    ///
    ///'center' is an alias to 'grow' with position set to center of screen.
    ///
    ///'any' is an alias to 'grow' with position set to a random point on screen.
    ///
    ///'outer' is the same as grow but the circle shrinks instead of growing.
    ///
    ///'glitch' reveals the new image with a digital corruption effect (horizontal tearing
    /// and RGB channel splitting) that settles over time.
    ///
    ///'decrypt' reveals the new image block by block in a pseudo-random order, like a
    /// cipher being decoded.
    ///
    ///'dissolve' reveals the new image pixel by pixel in a pseudo-random order.
    ///
    ///'clock' sweeps the new image in like a clock hand, using `--transition-pos` as the center.
    ///
    ///'zoom' zooms out from `--transition-pos` (default: center) to reveal the new image.
    ///
    ///Finally, 'random' will select a transition effect at random
    #[arg(short, long, env = "XWWW_TRANSITION", default_value = "simple")]
    pub transition_type: TransitionType,

    ///How fast the transition approaches the new image.
    ///
    ///The transition logic works by adding or subtracting from the current rgb values until the
    ///old image transforms in the new one. This controls by how much we add or subtract.
    ///
    ///Larger values will make the transition faster, but more abrupt. A value of 255 will always
    ///switch to the new image immediately.
    ///
    /// This defaults to 2 when transition-type is 'simple', and 90 otherwise
    #[arg(
        long,
        env = "XWWW_TRANSITION_STEP",
        default_value = "90",
        default_value_if("transition_type", "simple", "2")
    )]
    pub transition_step: std::num::NonZeroU8,

    ///How long the transition takes to complete in seconds.
    ///
    ///Note that this doesn't work with the 'simple' transition
    #[arg(long, env = "XWWW_TRANSITION_DURATION", default_value = "3")]
    pub transition_duration: f32,

    ///Frame rate for the transition effect.
    ///
    ///Note there is no point in setting this to a value smaller than what your monitor supports.
    ///
    ///Also note this is **different** from the transition-step. That one controls by how much we
    ///approach the new image every frame.
    #[arg(long, env = "XWWW_TRANSITION_FPS", default_value = "30")]
    pub transition_fps: u16,

    ///This is used for the 'wipe' and 'wave' transitions. It controls the angle of the wipe
    ///
    ///Note that the angle is in degrees, where '0' is right to left and '90' is top to bottom,
    /// and '270' bottom to top
    #[arg(long, env = "XWWW_TRANSITION_ANGLE", default_value = "45")]
    pub transition_angle: f64,

    ///This is only used for the 'grow','outer' transitions. It controls the center of circle
    /// (default is 'center').
    ///
    ///Position values can be given in both percentage values and pixel values:
    ///  float values are interpreted as percentages and integer values as pixel values
    ///  eg: 0.5,0.5 means 50% of the screen width and 50% of the screen height
    ///      200,400 means 200 pixels from the left and 400 pixels from the bottom
    ///
    ///the value can also be an alias which will set the position accordingly):
    /// 'center' | 'top' | 'left' | 'right' | 'bottom' | 'top-left' | 'top-right' | 'bottom-left' |
    /// 'bottom-right'
    #[arg(long, env = "XWWW_TRANSITION_POS", default_value = "center", value_parser=parse_coords)]
    pub transition_pos: CliPosition,

    ///bezier curve to use for the transition
    ///https://cubic-bezier.com is a good website to get these values from
    ///
    ///eg: 0.0,0.0,1.0,1.0 for linear animation
    #[arg(long, env = "XWWW_TRANSITION_BEZIER", default_value = ".54,0,.34,.99", value_parser = parse_bezier)]
    pub transition_bezier: (f32, f32, f32, f32),

    ///currently only used for 'wave' transition to control the width and height of each wave
    #[arg(long, env = "XWWW_TRANSITION_WAVE", default_value = "20,20", value_parser = parse_wave)]
    pub transition_wave: (f32, f32),

    /// inverts the y position sent in 'transition_pos' flag
    #[arg(long, env = "INVERT_Y", default_value = "false")]
    pub invert_y: bool,
}

fn parse_wave(raw: &str) -> Result<(f32, f32), String> {
    let mut iter = raw.split(',');
    let mut parse = || {
        iter.next()
            .ok_or_else(|| "Not enough values".to_string())
            .and_then(|s| s.parse::<f32>().map_err(|e| e.to_string()))
    };

    let parsed = (parse()?, parse()?);
    Ok(parsed)
}

fn parse_bezier(raw: &str) -> Result<(f32, f32, f32, f32), String> {
    let mut iter = raw.split(',');
    let mut parse = || {
        iter.next()
            .ok_or_else(|| "Not enough values".to_string())
            .and_then(|s| s.parse::<f32>().map_err(|e| e.to_string()))
    };

    let parsed = (parse()?, parse()?, parse()?, parse()?);
    if parsed == (0.0, 0.0, 0.0, 0.0) {
        return Err("Invalid bezier curve: 0,0,0,0 (try using 0,0,1,1 instead)".to_string());
    }
    Ok(parsed)
}

/// Validates the `--map-palette` spec syntax early (parsing lives in `palette_source`).
fn parse_palette_spec(raw: &str) -> Result<String, String> {
    if raw == "xwww" || raw == "equisdots" {
        return Ok(raw.to_string());
    }

    match raw.split_once(':') {
        Some(("xwww", value)) if !value.is_empty() => Ok(raw.to_string()),
        Some(("equisdots", value)) if !value.is_empty() => Ok(raw.to_string()),
        Some(("file", value)) if !value.is_empty() => Ok(raw.to_string()),
        Some(("command", value)) if !value.is_empty() => Ok(raw.to_string()),
        Some((kind, _)) if matches!(kind, "xwww" | "equisdots" | "file" | "command") => {
            Err(format!("'{kind}' requires a value after ':'"))
        }
        None if raw.starts_with('/') || raw.starts_with('~') || raw.starts_with('.') => {
            Ok(raw.to_string())
        }
        _ => Err(format!(
            "unrecognized palette source '{raw}'. Valid sources: \
             xwww | xwww:<path> | equisdots | equisdots:<slug> | file:<path> | command:<cmd>"
        )),
    }
}

pub fn parse_image(raw: &str) -> Result<CliImage, String> {
    let path = PathBuf::from(raw);
    if raw == "-" || path.exists() {
        return Ok(CliImage::Path(path));
    }
    if let Some(color) = raw.strip_prefix("0x")
        && let Ok(color) = from_hex(color)
    {
        return Ok(CliImage::Color(color));
    }
    Err(format!("Path '{raw}' does not exist"))
}

// parses Percents and numbers in format of "<coord1>,<coord2>"
fn parse_coords(raw: &str) -> Result<CliPosition, String> {
    let coords = raw.split(',').map(str::trim).collect::<Vec<&str>>();
    if coords.len() != 2 {
        match coords[0] {
            "center" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.5),
                    CliCoord::Percent(0.5),
                ));
            }
            "top" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.5),
                    CliCoord::Percent(1.0),
                ));
            }
            "bottom" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.5),
                    CliCoord::Percent(0.0),
                ));
            }
            "left" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.0),
                    CliCoord::Percent(0.5),
                ));
            }
            "right" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(1.0),
                    CliCoord::Percent(0.5),
                ));
            }
            "top-left" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.0),
                    CliCoord::Percent(1.0),
                ));
            }
            "top-right" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(1.0),
                    CliCoord::Percent(1.0),
                ));
            }
            "bottom-left" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(0.0),
                    CliCoord::Percent(0.0),
                ));
            }
            "bottom-right" => {
                return Ok(CliPosition::new(
                    CliCoord::Percent(1.0),
                    CliCoord::Percent(0.0),
                ));
            }
            _ => return Err(format!("Invalid position keyword: {raw}")),
        }
    }

    let x = coords[0];
    let y = coords[1];

    let parsed_x = match x.parse::<u32>() {
        Ok(x) => CliCoord::Pixel(x as f32),
        Err(_) => match x.parse::<f32>() {
            Ok(x) => CliCoord::Percent(x),
            Err(_) => return Err(format!("Invalid x coord: {x}")),
        },
    };

    let parsed_y = match y.parse::<u32>() {
        Ok(y) => CliCoord::Pixel(y as f32),
        Err(_) => match y.parse::<f32>() {
            Ok(y) => CliCoord::Percent(y),
            Err(_) => return Err(format!("Invalid y coord: {y}")),
        },
    };

    Ok(CliPosition::new(parsed_x, parsed_y))
}

#[derive(Parser)]
pub struct Palette {
    /// Image to extract colors from. If omitted, the currently displayed wallpaper is used.
    pub image: Option<PathBuf>,

    /// Number of colors to output.
    #[arg(short, long, default_value = "8")]
    pub count: usize,

    /// Emit a JSON array instead of plain hex lines.
    #[arg(long)]
    pub json: bool,

    /// When querying the current wallpaper, only consider this output.
    #[arg(short, long)]
    pub output: Option<String>,

    /// The daemon's namespace.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Slideshow {
    /// Directory containing the images to cycle through.
    pub dir: PathBuf,

    /// Time in seconds between wallpapers.
    #[arg(short, long, default_value = "60")]
    pub interval: u64,

    /// Shuffle the images instead of going in alphabetical order.
    #[arg(short, long)]
    pub random: bool,

    /// Transition to use between wallpapers (default: `any`).
    #[arg(short, long, default_value = "any")]
    pub transition_type: TransitionType,

    /// Resize strategy for each image.
    #[arg(long, default_value = "crop")]
    pub resize: ResizeStrategy,

    /// The daemon's namespace.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Random {
    /// Directory to pick a random image from.
    pub dir: PathBuf,

    /// Transition to use when setting the wallpaper.
    #[arg(short, long, default_value = "any")]
    pub transition_type: TransitionType,

    /// Resize strategy for the image.
    #[arg(long, default_value = "crop")]
    pub resize: ResizeStrategy,

    /// The daemon's namespace.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Screenshot {
    /// Output file path for the captured wallpaper (PNG).
    pub output: PathBuf,

    /// Which monitor to capture (defaults to the first one).
    #[arg(short, long)]
    pub monitor: Option<String>,

    /// The daemon's namespace.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,
}

#[derive(Parser)]
pub struct Scene {
    #[command(subcommand)]
    pub command: SceneCommand,
}

#[derive(Subcommand)]
pub enum SceneCommand {
    ///Compiles the scene and exits.
    Check(SceneCheck),

    ///Renders a single frame to a PNG file.
    Render(SceneRender),

    ///Renders the scene continuously and sends the frames to the daemon.
    Run(SceneRun),
}

#[derive(Parser)]
pub struct SceneCheck {
    /// Path to the JavaScript scene.
    pub script: PathBuf,
}

#[derive(Parser)]
pub struct SceneRender {
    /// Path to the JavaScript scene.
    pub script: PathBuf,

    /// Output PNG file.
    #[arg(short, long, default_value = "scene.png")]
    pub output: PathBuf,

    /// Canvas size in `WxH` physical pixels.
    #[arg(long, default_value = "2560x1440")]
    pub size: String,

    /// Palette source: `xwww[:<path>]`, `equisdots[:<slug>]`, `file:<path>` or `command:<cmd>`.
    ///
    /// Defaults to `~/.config/xwww/palette.json`, then the equisdots palette, then a neutral
    /// fallback.
    #[arg(long)]
    pub palette: Option<String>,

    /// Per-frame JavaScript execution budget in milliseconds.
    #[arg(long, default_value = "100")]
    pub timeout_ms: u64,

    /// Extra directory (or file) whose images the scene may load with `canvas.image`.
    ///
    /// The scene's own directory is always allowed. Repeat the flag for several paths.
    #[arg(long = "asset", value_name = "PATH")]
    pub assets: Vec<PathBuf>,
}

#[derive(Parser)]
pub struct SceneRun {
    /// Path to the JavaScript scene.
    pub script: PathBuf,

    /// Frames per second to render and send.
    #[arg(long, default_value = "10")]
    pub fps: u32,

    /// Crossfade duration when the active palette changes, in milliseconds (0 disables it).
    ///
    /// While the crossfade runs, the previous frame is blended over the new one so palette
    /// switches transition smoothly instead of snapping. It looks best when the frame rate
    /// provides several frames during the crossfade.
    #[arg(long, env = "XWWW_PALETTE_FADE", default_value = "600")]
    pub palette_fade: u64,

    /// Palette source: `xwww[:<path>]`, `equisdots[:<slug>]`, `file:<path>` or `command:<cmd>`.
    ///
    /// Defaults to `~/.config/xwww/palette.json`, then the equisdots palette, then a neutral
    /// fallback.
    #[arg(long)]
    pub palette: Option<String>,

    /// Per-frame JavaScript execution budget in milliseconds.
    #[arg(long, default_value = "100")]
    pub timeout_ms: u64,

    /// Extra directory (or file) whose images the scene may load with `canvas.image`.
    ///
    /// The scene's own directory is always allowed. Repeat the flag for several paths.
    #[arg(long = "asset", value_name = "PATH")]
    pub assets: Vec<PathBuf>,

    /// Comma separated list of outputs to display the scene at.
    ///
    /// If it isn't set, the scene is displayed on all outputs.
    #[arg(short, long, default_value = "")]
    pub outputs: String,

    /// The daemon's namespace.
    ///
    /// The resulting namespace will be 'xwww-daemon' appended to what you pass in this argument.
    /// For this to work, you must call `xwww-daemon --namespace <custom_namespace>` with the same
    /// value you use here.
    #[arg(short, long, default_value = "")]
    pub namespace: Vec<String>,

    /// Entry transition for the first frame (same set as `xwww img --transition-type`).
    ///
    /// The first frame is sent with this transition, then the render loop waits for
    /// `--transition-duration` before sending instant frames, so the effect is not cut off.
    /// Default `none` starts instantly.
    #[arg(long, env = "XWWW_TRANSITION", default_value = "none")]
    pub transition_type: TransitionType,

    /// How fast the transition approaches the new frame (`simple` transition only).
    #[arg(long, env = "XWWW_TRANSITION_STEP", default_value = "255")]
    pub transition_step: std::num::NonZeroU8,

    /// How long the entry transition takes to complete, in seconds.
    ///
    /// The scene does not send further frames until it finishes. Ignored by `simple`.
    #[arg(long, env = "XWWW_TRANSITION_DURATION", default_value = "1.0")]
    pub transition_duration: f32,

    /// Frame rate for the entry transition.
    #[arg(long, env = "XWWW_TRANSITION_FPS", default_value = "144")]
    pub transition_fps: u16,

    /// Angle for the `wipe`/`wave` transitions, in degrees.
    #[arg(long, env = "XWWW_TRANSITION_ANGLE", default_value = "45")]
    pub transition_angle: f64,

    /// Center used by `grow`/`outer`/`zoom`.
    #[arg(long, env = "XWWW_TRANSITION_POS", default_value = "center", value_parser = parse_coords)]
    pub transition_pos: CliPosition,

    /// Bezier curve for the `fade`/`grow`/`outer` transitions.
    #[arg(long, env = "XWWW_TRANSITION_BEZIER", default_value = ".54,0,.34,.99", value_parser = parse_bezier)]
    pub transition_bezier: (f32, f32, f32, f32),

    /// Wave size for the `wave` transition.
    #[arg(long, env = "XWWW_TRANSITION_WAVE", default_value = "20,20", value_parser = parse_wave)]
    pub transition_wave: (f32, f32),

    /// Inverts the y position sent in `--transition-pos`.
    #[arg(long, env = "INVERT_Y", default_value = "false")]
    pub invert_y: bool,
}

/// The transition options shared by `img` and `scene run`; [`Img::transition_args`] and
/// [`SceneRun::transition_args`] normalize them for [`crate::imgproc::make_transition`].
#[derive(Clone)]
pub struct TransitionArgs {
    pub transition_type: TransitionType,
    pub transition_step: std::num::NonZeroU8,
    pub transition_duration: f32,
    pub transition_fps: u16,
    pub transition_angle: f64,
    pub transition_pos: CliPosition,
    pub transition_bezier: (f32, f32, f32, f32),
    pub transition_wave: (f32, f32),
    pub invert_y: bool,
}

impl Img {
    /// View of this command's transition flags for the shared transition builder.
    #[must_use]
    pub fn transition_args(&self) -> TransitionArgs {
        TransitionArgs {
            transition_type: self.transition_type.clone(),
            transition_step: self.transition_step,
            transition_duration: self.transition_duration,
            transition_fps: self.transition_fps,
            transition_angle: self.transition_angle,
            transition_pos: self.transition_pos.clone(),
            transition_bezier: self.transition_bezier,
            transition_wave: self.transition_wave,
            invert_y: self.invert_y,
        }
    }
}

impl SceneRun {
    /// View of this command's entry-transition flags for the shared transition builder.
    #[must_use]
    pub fn transition_args(&self) -> TransitionArgs {
        TransitionArgs {
            transition_type: self.transition_type.clone(),
            transition_step: self.transition_step,
            transition_duration: self.transition_duration,
            transition_fps: self.transition_fps,
            transition_angle: self.transition_angle,
            transition_pos: self.transition_pos.clone(),
            transition_bezier: self.transition_bezier,
            transition_wave: self.transition_wave,
            invert_y: self.invert_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reject_wrong_colors() {
        assert!(
            from_hex("0012231").is_err(),
            "function is accepting strings with more than 6 chars"
        );
        assert!(
            from_hex("00122").is_err(),
            "function is accepting strings with less than 6 chars"
        );
        assert!(
            from_hex("00r223").is_err(),
            "function is accepting strings with chars that aren't hex"
        );
    }

    #[test]
    fn should_convert_colors_from_hex() {
        let color = from_hex("101010").unwrap();
        assert_eq!(color, [16, 16, 16, 255]);

        let color = from_hex("ffffff").unwrap();
        assert_eq!(color, [255, 255, 255, 255]);

        let color = from_hex("000000").unwrap();
        assert_eq!(color, [0, 0, 0, 255]);

        let color = from_hex("00000000").unwrap();
        assert_eq!(color, [0, 0, 0, 0]);

        let color = from_hex("eeeeeeee").unwrap();
        assert_eq!(color, [0xee, 0xee, 0xee, 0xee]);
    }
}
