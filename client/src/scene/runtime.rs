//! The JavaScript runtime of a scene: a sandboxed QuickJS context with a `canvas` global.
//!
//! Scene scripts define two optional top-level functions:
//!
//! ```js
//! function setup(ctx) { /* runs once */ }
//! function render(t, ctx) { /* runs once per frame */ }
//! ```
//!
//! The runtime exposes only the canvas bindings and the context object. There is no module
//! loader, filesystem, network or process access; memory and stack are capped, and every call is
//! bounded by the configured timeout through QuickJS' interrupt handler.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rquickjs::{Array, Context, Ctx, Function, Object, Runtime, Value};

use super::canvas::Canvas;
use crate::palette_source::{Rgb, ScenePalette};

const MEMORY_LIMIT: usize = 32 * 1024 * 1024;
const MAX_STACK_SIZE: usize = 1024 * 1024;
const WATCHDOG_TICK: Duration = Duration::from_millis(5);

/// JavaScript prelude: wraps the low-level numeric bindings with a canvas 2D style API and color
/// parsing. Errors thrown here surface as normal scene errors.
const PRELUDE: &str = r##"
(function () {
    const raw = globalThis.__canvas;

    function rgba(color) {
        if (typeof color === "string") {
            let s = color.trim().replace(/^#/, "");
            if (s.length === 3) {
                s = s[0] + s[0] + s[1] + s[1] + s[2] + s[2];
            }
            if (s.length !== 6 && s.length !== 8) {
                throw new Error("invalid color: " + color);
            }
            const r = parseInt(s.slice(0, 2), 16);
            const g = parseInt(s.slice(2, 4), 16);
            const b = parseInt(s.slice(4, 6), 16);
            const a = s.length === 8 ? parseInt(s.slice(6, 8), 16) : 255;
            if ([r, g, b, a].some(Number.isNaN)) {
                throw new Error("invalid color: " + color);
            }
            return [r, g, b, a];
        }
        if (Array.isArray(color)) {
            if (color.length < 3 || color.length > 4) {
                throw new Error("invalid color array: " + JSON.stringify(color));
            }
            return [color[0], color[1], color[2], color[3] === undefined ? 255 : color[3]];
        }
        throw new Error("invalid color: " + color);
    }

    function flatten(colors) {
        if (!Array.isArray(colors) || colors.length === 0) {
            throw new Error("gradient needs a non-empty array of colors");
        }
        const flat = [];
        for (const color of colors) {
            const [r, g, b, a] = rgba(color);
            flat.push(r, g, b, a);
        }
        return flat;
    }

    globalThis.canvas = Object.freeze({
        clear(color) { const [r, g, b, a] = rgba(color); raw.clear(r, g, b, a); },
        fill(color) { const [r, g, b, a] = rgba(color); raw.fill(r, g, b, a); },
        no_fill() { raw.no_fill(); },
        stroke(color, width) {
            const [r, g, b, a] = rgba(color);
            raw.stroke(r, g, b, a, width === undefined ? 2 : width);
        },
        no_stroke() { raw.no_stroke(); },
        alpha(value) { raw.alpha(value); },
        rect(x, y, w, h) { raw.rect(x, y, w, h); },
        circle(cx, cy, r) { raw.circle(cx, cy, r); },
        round_rect(x, y, w, h, r) { raw.round_rect(x, y, w, h, r); },
        begin_path() { raw.begin_path(); },
        move_to(x, y) { raw.move_to(x, y); },
        line_to(x, y) { raw.line_to(x, y); },
        quad_to(cx, cy, x, y) { raw.quad_to(cx, cy, x, y); },
        cubic_to(c1x, c1y, c2x, c2y, x, y) { raw.cubic_to(c1x, c1y, c2x, c2y, x, y); },
        close_path() { raw.close_path(); },
        fill_path() { raw.fill_path(); },
        stroke_path() { raw.stroke_path(); },
        linear_gradient(x0, y0, x1, y1, colors) {
            raw.linear_gradient(x0, y0, x1, y1, flatten(colors));
        },
        radial_gradient(cx, cy, r, colors) {
            raw.radial_gradient(cx, cy, r, flatten(colors));
        },
        image(path, x, y, w, h) { raw.image(path, x, y, w, h); },
        image_tinted(path, x, y, w, h, color) {
            const [r, g, b, a] = rgba(color === undefined ? "#ffffff" : color);
            raw.image_tinted(path, x, y, w, h, { color: r * 16777216 + g * 65536 + b * 256 + a });
        },
        remap(x, y, w, h, colors, strength) {
            raw.remap(x, y, w, h, flatten(colors), strength === undefined ? 1 : strength);
        },
        push() { raw.push(); },
        pop() { raw.pop(); },
        translate(x, y) { raw.translate(x, y); },
        rotate(degrees) { raw.rotate(degrees); },
        scale(x, y) { raw.scale(x, y); },
        text(str, x, y, size, color, options) {
            const [r, g, b, a] = rgba(color === undefined ? "#ffffff" : color);
            const opts = options || {};
            const packed = r * 16777216 + g * 65536 + b * 256 + a;
            raw.text(String(str), x, y, size, packed, opts);
        },
    });

    globalThis.log = (...args) => raw.log(args.map(String).join(" "));

    delete globalThis.__canvas;
})();
"##;

/// A loaded scene: QuickJS context, canvas and frame bookkeeping.
pub struct SceneRuntime {
    context: Context,
    canvas: Rc<RefCell<Canvas>>,
    frame: Cell<u64>,
    width: u32,
    height: u32,
    timeout: Duration,
    interrupted: Arc<AtomicBool>,
    deadline: Arc<Mutex<Instant>>,
}

impl SceneRuntime {
    pub fn new(
        script: &str,
        width: u32,
        height: u32,
        timeout: Duration,
        assets: &[PathBuf],
    ) -> Result<Self, String> {
        let mut canvas = Canvas::new(width, height)?;
        for dir in assets {
            canvas.allow_asset_dir(dir);
        }
        let canvas = Rc::new(RefCell::new(canvas));
        let interrupted = Arc::new(AtomicBool::new(false));
        let deadline = Arc::new(Mutex::new(Instant::now() + timeout));

        let runtime = Runtime::new().map_err(|e| format!("failed to create JS runtime: {e}"))?;
        runtime.set_memory_limit(MEMORY_LIMIT);
        runtime.set_max_stack_size(MAX_STACK_SIZE);
        {
            let flag = interrupted.clone();
            runtime.set_interrupt_handler(Some(Box::new(move || flag.load(Ordering::Relaxed))));
        }

        let context =
            Context::full(&runtime).map_err(|e| format!("failed to create JS context: {e}"))?;

        context
            .with(|ctx| -> rquickjs::Result<()> {
                bind_canvas(&ctx, canvas.clone())?;
                ctx.eval::<(), _>(PRELUDE)
                    .map_err(|e| wrap_js_error(&ctx, e))?;
                ctx.eval::<(), _>(script)
                    .map_err(|e| wrap_js_error(&ctx, e))?;
                Ok(())
            })
            .map_err(|e| format!("failed to load scene: {}", describe_error(&e)))?;

        {
            let deadline = deadline.clone();
            let interrupted = interrupted.clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(WATCHDOG_TICK);
                    let expired = *deadline.lock().unwrap() <= Instant::now();
                    if expired {
                        interrupted.store(true, Ordering::Relaxed);
                    }
                }
            });
        }

        Ok(Self {
            context,
            canvas,
            frame: Cell::new(0),
            width,
            height,
            timeout,
            interrupted,
            deadline,
        })
    }

    /// Calls `setup(ctx)` if the script defines it, then `render(t, ctx)`.
    pub fn render(
        &self,
        t: f64,
        palette: &ScenePalette,
        setup_done: &mut bool,
    ) -> Result<(), String> {
        if !*setup_done {
            self.invoke("setup", None, palette)?;
            *setup_done = true;
        }
        self.invoke("render", Some(t), palette)
    }

    /// Calls a global function; `setup` receives `(ctx)` and `render` receives `(t, ctx)`.
    /// A missing function is a no-op.
    fn invoke(&self, name: &str, t: Option<f64>, palette: &ScenePalette) -> Result<(), String> {
        *self.deadline.lock().unwrap() = Instant::now() + self.timeout;
        self.interrupted.store(false, Ordering::Relaxed);

        let frame = self.frame.get();
        self.frame.set(frame + 1);

        let result = self.context.with(|ctx| -> rquickjs::Result<()> {
            let value = ctx
                .globals()
                .get::<_, Value>(name)
                .map_err(|e| wrap_js_error(&ctx, e))?;
            if !value.is_function() {
                return Ok(());
            }
            let function = value
                .into_function()
                .expect("is_function guarantees a Function");
            let context = build_context(&ctx, self.width, self.height, frame, palette)
                .map_err(|e| wrap_js_error(&ctx, e))?;
            match t {
                Some(t) => function
                    .call::<_, ()>((t, context))
                    .map_err(|e| wrap_js_error(&ctx, e))?,
                None => function
                    .call::<_, ()>((context,))
                    .map_err(|e| wrap_js_error(&ctx, e))?,
            }
            Ok(())
        });

        if self.interrupted.swap(false, Ordering::Relaxed) {
            return Err(format!(
                "scene '{name}' exceeded the {} ms budget",
                self.timeout.as_millis()
            ));
        }

        result.map_err(|e| format!("scene '{name}' failed: {}", describe_error(&e)))
    }

    pub fn with_canvas<R>(&self, f: impl FnOnce(&Canvas) -> R) -> R {
        f(&self.canvas.borrow())
    }

    /// Whether the last frame (or an earlier one) painted anything not yet sent.
    pub fn take_dirty(&self) -> bool {
        self.canvas.borrow_mut().take_dirty()
    }

    /// Stores the current canvas as the start of the crossfade.
    pub fn save_fade_from(&self) {
        self.canvas.borrow_mut().save_fade_from();
    }

    /// Stores the current canvas as the end of the crossfade.
    pub fn save_fade_to(&self) {
        self.canvas.borrow_mut().save_fade_to();
    }

    /// Restores the clean crossfade target on the canvas without dropping the fade state.
    pub fn restore_fade_target(&self) {
        self.canvas.borrow_mut().restore_fade_target();
    }

    /// Rebuilds the canvas as the new frame with the old one faded on top.
    pub fn draw_fade(&self, alpha: f32) {
        self.canvas.borrow_mut().draw_fade(alpha);
    }

    /// Restores the clean new frame and drops both saved frames.
    pub fn clear_fade(&self) {
        self.canvas.borrow_mut().clear_fade();
    }
}

/// Converts a QuickJS exception into a conversion error carrying its message, so the message
/// survives `Context::with` (which only sees a generic `Error::Exception`).
fn wrap_js_error(ctx: &Ctx<'_>, error: rquickjs::Error) -> rquickjs::Error {
    if !error.is_exception() {
        return error;
    }
    let value = ctx.catch();
    rquickjs::Error::new_from_js_message("scene", "exception", describe_value(value))
}

fn describe_value(value: rquickjs::Value<'_>) -> String {
    if let Some(object) = value.as_object()
        && let Some(exception) = rquickjs::Exception::from_object(object.clone())
    {
        let message = exception
            .message()
            .unwrap_or_else(|| "uncaught exception".into());
        if let Some(stack) = exception.stack().filter(|stack| !stack.is_empty()) {
            return format!("{message} ({stack})");
        }
        return message;
    }
    format!("{value:?}")
}

fn describe_error(error: &rquickjs::Error) -> String {
    match error {
        rquickjs::Error::FromJs {
            message: Some(message),
            ..
        }
        | rquickjs::Error::IntoJs {
            message: Some(message),
            ..
        } => message.clone(),
        other => other.to_string(),
    }
}

fn build_context<'js>(
    ctx: &Ctx<'js>,
    width: u32,
    height: u32,
    frame: u64,
    palette: &ScenePalette,
) -> rquickjs::Result<Object<'js>> {
    let context = Object::new(ctx.clone())?;
    context.set("width", width)?;
    context.set("height", height)?;
    context.set("frame", frame)?;
    context.set("now", super::providers::now_ms())?;
    context.set("palette", palette_to_js(ctx, palette)?)?;
    context.set("events", Array::new(ctx.clone())?)?;
    Ok(context)
}

fn palette_to_js<'js>(ctx: &Ctx<'js>, palette: &ScenePalette) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.set("slug", palette.slug.clone())?;
    object.set("name", palette.name.clone())?;
    object.set("background", color_to_js(ctx, palette.background)?)?;
    object.set("foreground", color_to_js(ctx, palette.foreground)?)?;

    let colors = Array::new(ctx.clone())?;
    for (index, color) in palette.colors.iter().enumerate() {
        colors.set(index, color_to_js(ctx, *color)?)?;
    }
    object.set("colors", colors)?;

    let roles = Object::new(ctx.clone())?;
    for (name, color) in &palette.roles {
        roles.set(name.as_str(), color_to_js(ctx, *color)?)?;
    }
    object.set("roles", roles)?;

    Ok(object)
}

fn color_to_js<'js>(ctx: &Ctx<'js>, color: Rgb) -> rquickjs::Result<Object<'js>> {
    let object = Object::new(ctx.clone())?;
    object.set("hex", color.hex())?;
    object.set("r", color.r)?;
    object.set("g", color.g)?;
    object.set("b", color.b)?;
    Ok(object)
}

fn channel(value: f64) -> u8 {
    value.clamp(0.0, 255.0) as u8
}

fn bind_canvas(ctx: &Ctx<'_>, canvas: Rc<RefCell<Canvas>>) -> rquickjs::Result<()> {
    let raw = Object::new(ctx.clone())?;

    {
        let cell = canvas.clone();
        raw.set(
            "clear",
            Function::new(ctx.clone(), move |r: f64, g: f64, b: f64, a: f64| {
                cell.borrow_mut()
                    .clear([channel(r), channel(g), channel(b), channel(a)]);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "fill",
            Function::new(ctx.clone(), move |r: f64, g: f64, b: f64, a: f64| {
                cell.borrow_mut()
                    .fill(Some([channel(r), channel(g), channel(b), channel(a)]));
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "no_fill",
            Function::new(ctx.clone(), move || cell.borrow_mut().fill(None))?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "stroke",
            Function::new(
                ctx.clone(),
                move |r: f64, g: f64, b: f64, a: f64, width: f64| {
                    cell.borrow_mut().stroke(Some((
                        [channel(r), channel(g), channel(b), channel(a)],
                        width as f32,
                    )));
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "no_stroke",
            Function::new(ctx.clone(), move || cell.borrow_mut().stroke(None))?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "alpha",
            Function::new(ctx.clone(), move |value: f64| {
                cell.borrow_mut().alpha(value as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "rect",
            Function::new(ctx.clone(), move |x: f64, y: f64, w: f64, h: f64| {
                cell.borrow_mut()
                    .rect(x as f32, y as f32, w as f32, h as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "circle",
            Function::new(ctx.clone(), move |cx: f64, cy: f64, r: f64| {
                cell.borrow_mut().circle(cx as f32, cy as f32, r as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "round_rect",
            Function::new(
                ctx.clone(),
                move |x: f64, y: f64, w: f64, h: f64, r: f64| {
                    cell.borrow_mut()
                        .round_rect(x as f32, y as f32, w as f32, h as f32, r as f32);
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "begin_path",
            Function::new(ctx.clone(), move || cell.borrow_mut().begin_path())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "move_to",
            Function::new(ctx.clone(), move |x: f64, y: f64| {
                cell.borrow_mut().move_to(x as f32, y as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "line_to",
            Function::new(ctx.clone(), move |x: f64, y: f64| {
                cell.borrow_mut().line_to(x as f32, y as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "quad_to",
            Function::new(ctx.clone(), move |cx: f64, cy: f64, x: f64, y: f64| {
                cell.borrow_mut()
                    .quad_to(cx as f32, cy as f32, x as f32, y as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "cubic_to",
            Function::new(
                ctx.clone(),
                move |c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64| {
                    cell.borrow_mut().cubic_to(
                        c1x as f32, c1y as f32, c2x as f32, c2y as f32, x as f32, y as f32,
                    );
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "close_path",
            Function::new(ctx.clone(), move || cell.borrow_mut().close_path())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "fill_path",
            Function::new(ctx.clone(), move || cell.borrow_mut().fill_path())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "stroke_path",
            Function::new(ctx.clone(), move || cell.borrow_mut().stroke_path())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "linear_gradient",
            Function::new(
                ctx.clone(),
                move |x0: f64, y0: f64, x1: f64, y1: f64, colors: Vec<f64>| {
                    let colors = to_colors(&colors);
                    cell.borrow_mut()
                        .linear_gradient(x0 as f32, y0 as f32, x1 as f32, y1 as f32, &colors);
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "radial_gradient",
            Function::new(
                ctx.clone(),
                move |cx: f64, cy: f64, r: f64, colors: Vec<f64>| {
                    let colors = to_colors(&colors);
                    cell.borrow_mut()
                        .radial_gradient(cx as f32, cy as f32, r as f32, &colors);
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "image",
            Function::new(
                ctx.clone(),
                move |path: String,
                      x: f64,
                      y: f64,
                      w: f64,
                      h: f64|
                      -> Result<(), rquickjs::Error> {
                    cell.borrow_mut()
                        .draw_image(&path, x as f32, y as f32, w as f32, h as f32)
                        .map_err(|message| {
                            rquickjs::Error::new_from_js_message("canvas", "image", message)
                        })
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "image_tinted",
            Function::new(
                ctx.clone(),
                move |path: String,
                      x: f64,
                      y: f64,
                      w: f64,
                      h: f64,
                      options: Object|
                      -> Result<(), rquickjs::Error> {
                    let packed = options.get::<_, Option<f64>>("color")?.unwrap_or(0xffff_ffff_u32 as f64) as u32;
                    let color = [
                        (packed >> 24) as u8,
                        (packed >> 16) as u8,
                        (packed >> 8) as u8,
                        packed as u8,
                    ];
                    cell.borrow_mut()
                        .draw_image_tinted(&path, x as f32, y as f32, w as f32, h as f32, color)
                        .map_err(|message| {
                            rquickjs::Error::new_from_js_message("canvas", "image_tinted", message)
                        })
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "remap",
            Function::new(
                ctx.clone(),
                move |x: f64,
                      y: f64,
                      w: f64,
                      h: f64,
                      colors: Vec<f64>,
                      strength: f64|
                      -> Result<(), rquickjs::Error> {
                    let colors = to_colors(&colors);
                    cell.borrow_mut()
                        .remap(
                            x as f32,
                            y as f32,
                            w as f32,
                            h as f32,
                            &colors,
                            strength as f32,
                        )
                        .map_err(|message| {
                            rquickjs::Error::new_from_js_message("canvas", "remap", message)
                        })
                },
            )?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "push",
            Function::new(ctx.clone(), move || cell.borrow_mut().push())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "pop",
            Function::new(ctx.clone(), move || cell.borrow_mut().pop())?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "translate",
            Function::new(ctx.clone(), move |x: f64, y: f64| {
                cell.borrow_mut().translate(x as f32, y as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "rotate",
            Function::new(ctx.clone(), move |degrees: f64| {
                cell.borrow_mut().rotate(degrees as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "scale",
            Function::new(ctx.clone(), move |x: f64, y: f64| {
                cell.borrow_mut().scale(x as f32, y as f32);
            })?,
        )?;
    }
    {
        let cell = canvas.clone();
        raw.set(
            "text",
            Function::new(
                ctx.clone(),
                move |text: String,
                      x: f64,
                      y: f64,
                      size: f64,
                      packed: f64,
                      options: Object|
                      -> Result<(), rquickjs::Error> {
                    let packed = packed as u32;
                    let color = [
                        (packed >> 24) as u8,
                        (packed >> 16) as u8,
                        (packed >> 8) as u8,
                        packed as u8,
                    ];
                    let family = options
                        .get::<_, Option<String>>("family")?
                        .unwrap_or_else(|| "sans-serif".into());
                    let anchor = options
                        .get::<_, Option<String>>("anchor")?
                        .unwrap_or_else(|| "start".into());
                    let bold = options.get::<_, Option<bool>>("bold")?.unwrap_or(false);
                    cell.borrow_mut()
                        .text(
                            &text,
                            x as f32,
                            y as f32,
                            size as f32,
                            color,
                            &family,
                            &anchor,
                            bold,
                        )
                        .map_err(|message| {
                            rquickjs::Error::new_from_js_message("canvas", "text", message)
                        })
                },
            )?,
        )?;
    }
    {
        raw.set(
            "log",
            Function::new(ctx.clone(), move |message: String| {
                eprintln!("[scene] {message}");
            })?,
        )?;
    }

    ctx.globals().set("__canvas", raw)?;
    Ok(())
}

fn to_colors(flat: &[f64]) -> Vec<[u8; 4]> {
    flat.as_chunks::<4>()
        .0
        .iter()
        .map(|c| [channel(c[0]), channel(c[1]), channel(c[2]), channel(c[3])])
        .collect()
}
