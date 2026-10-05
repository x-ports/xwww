<a id="top" name="top"></a>

# Scene engine (smart wallpapers)

The scene engine renders procedural wallpapers written in JavaScript. A scene
can read the active desktop color palette, react to time, and draw shapes,
gradients, images and text on a small canvas. Frames are rendered by the
`xwww` client and displayed by the daemon through the normal image pipeline, so
the wallpaper stays click-through and the daemon remains a plain frame consumer.

The engine is available in builds with the `scene` feature (off by default;
release archives include it). See [Cargo features](cargo-features.md).

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#commands">Commands</a> &middot;
  <a href="#scene-contract">Scene contract</a> &middot;
  <a href="#canvas-api">Canvas API</a> &middot;
  <a href="#authoring-a-scene">Authoring a scene</a> &middot;
  <a href="#execution-rules">Execution rules</a> &middot;
  <a href="#palette-sources">Palette sources</a> &middot;
  <a href="#transport-and-performance">Transport and performance</a> &middot;
  <a href="#integrations">Integrations</a> &middot;
  <a href="#testing">Testing</a> &middot;
  <a href="#roadmap">Roadmap</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Commands](#commands)
- [Scene contract](#scene-contract)
  - [Context object](#context-object)
- [Canvas API](#canvas-api)
- [Authoring a scene](#authoring-a-scene)
- [Execution rules](#execution-rules)
- [Palette sources](#palette-sources)
  - [Palette file contract](#palette-file-contract)
  - [Equisdots integration](#equisdots-integration)
- [Crossfade on palette changes](#crossfade-on-palette-changes)
- [Transport and performance](#transport-and-performance)
- [Integrations](#integrations)
- [Testing](#testing)
- [Roadmap](#roadmap)
- [Design decisions](#design-decisions)
- [File map](#file-map)

</details>

<a id="commands" name="commands"></a>

## Commands

```sh
xwww scene check  scene.js                                 # compile only
xwww scene render scene.js -o out.png --size 2560x1440     # one-shot render
xwww scene run    scene.js --fps 10 --palette xwww         # run until interrupted
```

### `scene check`

Compiles the script in a runtime created for a small canvas and exits. It
prints `<path>: ok` on success and fails with the JavaScript error otherwise.

### `scene render`

Renders a single frame to a PNG. Useful while iterating on a scene offline.

| Flag | Default | Meaning |
|------|---------|---------|
| `-o, --output` | `scene.png` | Output PNG path. |
| `--size` | `2560x1440` | Canvas size in `WxH` physical pixels. |
| `--palette` | xwww file, then equisdots, then fallback | Palette source. |
| `--timeout-ms` | `100` | Per-frame JavaScript execution budget. |
| `--asset <PATH>` | scene directory | Extra directory (or file) whose images the scene may load. Repeatable. |

### `scene run`

Renders continuously and pushes frames to the daemon.

| Flag | Default | Meaning |
|------|---------|---------|
| `--fps` | `10` | Frames rendered and pushed per second. |
| `--palette-fade` | `800` | Crossfade in milliseconds when the palette changes (`0` disables it). |
| `--palette` | xwww file, then equisdots, then fallback | Palette source. |
| `--timeout-ms` | `100` | Per-frame JavaScript execution budget. |
| `--asset <PATH>` | scene directory | Extra asset paths. Repeatable. |
| `-o, --outputs` | all | Comma-separated outputs. |
| `-n, --namespace` | `""` | Daemon namespace. |
| `--transition-type` | `none` | Entry transition for the first frame (same set as `xwww img`). |
| `--transition-step` | `255` | Step for the entry transition. |
| `--transition-duration` | `1.0` | Seconds the entry transition takes. |
| `--transition-fps` | `144` | Entry transition frame rate. |
| `--transition-angle` | `45` | Angle for `wipe`/`wave`. |
| `--transition-pos` | `center` | Center for circular effects. |
| `--transition-bezier` | `.54,0,.34,.99` | Bezier curve. |
| `--transition-wave` | `20,20` | Wave size. |
| `--invert-y` | off | Flips the entry-transition y coordinate. |

Only the first frame uses the entry transition. The loop waits for
`--transition-duration` (plus a small margin) before sending the next frame, so
the animation is not cut off; every later frame is instant.

`xwww scene` is dispatched inside the client before the regular IPC flow, like
`palette`, `slideshow` and `screenshot`. Builds without the `scene` feature
still list the subcommand but fail with a clear message.

<a id="scene-contract" name="scene-contract"></a>

## Scene contract

A scene is a single ES module (a plain `.js` file) with two optional top-level
functions:

```js
function setup(ctx) {
  // Runs once, before the first render, in the same realm.
}

function render(t, ctx) {
  // Runs once per frame. t is the number of seconds since the scene started.
  // Must finish within the frame budget; on error the last good frame stays.
}
```

Module-level variables persist between frames, so scenes can carry state
counters, cached geometry and so on.

<a id="context-object" name="context-object"></a>

### Context object

`ctx` is rebuilt before every frame and is read-only:

```js
{
  width, height,       // output size in physical pixels
  frame,               // monotonically increasing frame counter
  now,                 // wall clock in milliseconds since the epoch
  palette: {
    slug: "x",                 // active palette slug
    name: "X",
    background: { hex: "#050505", r: 5, g: 5, b: 5 },
    foreground: { hex: "#f7f1ff", r: 247, g: 241, b: 255 },
    colors: [ /* base16 color0..color15 as { hex, r, g, b } */ ],
    roles: { workspaceActive: { hex: "#eab308", r: 234, g: 179, b: 8 } },
  },
  events: [],          // reserved for event providers; currently always empty
}
```

The `palette` object is refreshed at most once per second, so a palette switch
restyles the scene within a second.

<a id="canvas-api" name="canvas-api"></a>

## Canvas API

The global `canvas` is backed by `tiny-skia`. Colors accept `#rgb`, `#rrggbb`,
`#rrggbbaa` or `[r, g, b(, a)]` arrays. The canvas is transparent by default and
is composited over the palette background when sent to the daemon.

| Call | Notes |
|------|-------|
| `canvas.clear(color)` | Overwrites the whole surface without blending. |
| `canvas.fill(color)` / `canvas.no_fill()` | Sets the fill paint for the next shapes and paths. |
| `canvas.stroke(color, width)` / `canvas.no_stroke()` | Sets the stroke paint. |
| `canvas.alpha(a)` | Multiplies the opacity of subsequent paints. |
| `canvas.rect(x, y, w, h)` | Fills and strokes a rectangle. |
| `canvas.circle(cx, cy, r)` | Fills and strokes a circle. |
| `canvas.round_rect(x, y, w, h, r)` | Fills and strokes a rounded rectangle. |
| `canvas.begin_path()` | Starts a new path. |
| `canvas.move_to(x, y)` | Moves the current point. |
| `canvas.line_to(x, y)` | Adds a line segment. |
| `canvas.quad_to(cx, cy, x, y)` | Adds a quadratic bezier segment. |
| `canvas.cubic_to(c1x, c1y, c2x, c2y, x, y)` | Adds a cubic bezier segment. |
| `canvas.close_path()` | Closes the current path. |
| `canvas.fill_path()` / `canvas.stroke_path()` | Draws the current path. |
| `canvas.linear_gradient(x0, y0, x1, y1, colors)` | Colors are an array; stops are evenly spaced. |
| `canvas.radial_gradient(cx, cy, r, colors)` | Radial gradient. |
| `canvas.image(path, x, y, w, h)` | Draws an image asset scaled into the box. |
| `canvas.image_tinted(path, x, y, w, h, color)` | Like `canvas.image`, but replaces the image colors with `color` while keeping alpha. |
| `canvas.remap(x, y, w, h, colors, strength)` | Recolors only that rectangle: luminance through the gradient of `colors`, blended by `strength`. |
| `canvas.text(str, x, y, size, color, options)` | Draws text with its baseline at `x, y`. `options` is `{ family, anchor, bold }`, where `anchor` is `start`, `middle` or `end`. |
| `canvas.push()` / `canvas.pop()` | Saves and restores the transform stack. |
| `canvas.translate(x, y)` | Translates the current transform. |
| `canvas.rotate(deg)` | Rotates the current transform, in degrees. |
| `canvas.scale(x, y)` | Scales the current transform. |
| `log(...values)` | Writes to stderr with a `[scene]` prefix. |

Text is rasterized through `resvg`/`usvg` with the system font database (loaded
once per process). The generic `sans-serif` and `monospace` families map to the
first available family from a built-in preference list. Shaping text per call is
not free: if a scene draws many strings, do it once per palette or context
change instead of on every frame.

<a id="authoring-a-scene" name="authoring-a-scene"></a>

## Authoring a scene

A scene is a plain JavaScript file; there is no build step. Save it anywhere
and run `xwww scene run`:

```js
// clock.js - a palette-driven background that pulses with the accent color.
function render(t, ctx) {
  const { background, foreground, colors, roles } = ctx.palette;
  const accent = roles.workspaceActive || colors[1] || foreground;

  canvas.clear(background.hex);
  canvas.linear_gradient(0, 0, ctx.width, ctx.height, [background.hex, accent.hex]);
  canvas.rect(0, 0, ctx.width, ctx.height);

  const pulse = 0.5 + 0.5 * Math.sin(t * 1.5);
  canvas.fill(accent.hex);
  canvas.circle(ctx.width / 2, ctx.height / 2, 80 + 120 * pulse);
}
```

```sh
xwww scene render clock.js -o clock.png   # iterate offline
xwww scene run clock.js --fps 30          # live
```

<a id="execution-rules" name="execution-rules"></a>

## Execution rules

- One frame at a time; `render` must be synchronous. Promises are not awaited.
- A frame that throws or exceeds `--timeout-ms` is discarded with a rate-limited
  warning, and the previous frame stays on screen.
- `setup` runs once in the same realm; global state persists across frames.
- No `require`, no `import` of external files, no filesystem, network or process
  access. Only the bundled JavaScript builtins (`Math`, `JSON`, `Date`, ...)
  are available.
- Sandbox limits: 32 MiB of runtime memory, 1 MiB of stack, and the per-frame
  interrupt handler tied to `--timeout-ms`.
- `canvas.image` and `canvas.image_tinted` only load assets from the scene's own
  directory plus explicit `--asset` paths; any other path is rejected.
- Logs from scene code are prefixed with `[scene]` and rate-limited.

<a id="palette-sources" name="palette-sources"></a>

## Palette sources

`--palette` accepts the same specs as `xwww img --map-palette`:

| Spec | Meaning |
|------|---------|
| `xwww` / `xwww:<path>` | xwww palette file (`$XDG_CONFIG_HOME/xwww/palette.json` by default). |
| `equisdots` / `equisdots:<slug>` | equisdots desktop palette (`settings.json` plus `dock/palettes`). |
| `file:<path>` (or a bare path) | Palette JSON or one `#rrggbb` per line. |
| `command:<cmd>` | Runs the command with `sh -c` and parses stdout like `file`. |

Without `--palette`, the engine tries the xwww palette file first, then the
equisdots palette, then a neutral three-color fallback, so scenes always render
even on a machine with no theming integration.

<a id="palette-file-contract" name="palette-file-contract"></a>

### Palette file contract

The xwww palette file is the primary contract: the desktop writes the active
palette and xwww only reads it. The file is either a base16-style JSON object or
a plain text list of colors.

```json
{
  "name": "X",
  "slug": "x",
  "background": "#050505",
  "foreground": "#f7f1ff",
  "base16": {
    "color0": "#0a0a0a", "color1": "#fc618d", "color2": "#7bd88f",
    "color3": "#fce566", "color4": "#fd9353", "color5": "#948ae3",
    "color6": "#5ad4e6", "color7": "#f7f1ff", "color8": "#7e7b82",
    "color9": "#fc618d", "color10": "#7bd88f", "color11": "#fce566",
    "color12": "#fd9353", "color13": "#948ae3", "color14": "#5ad4e6",
    "color15": "#f7f1ff"
  },
  "roles": { "workspaceActive": "#eab308" }
}
```

Required: `base16.color0` through `base16.color15`. Optional: `background`
(falls back to `color0`), `foreground` (falls back to `color7`), `slug`,
`name` and `roles` (arbitrary semantic color names). A plain file with one
`#rrggbb` per line also works; `key=#rrggbb` lines are accepted and comments
start with `//` or `;`.

A typical desktop hook is a one-line copy whenever the theme changes:

```sh
install -Dm644 "$active_palette.json" "${XDG_CONFIG_HOME:-$HOME/.config}/xwww/palette.json"
```

The provider polls the file at most once per second and keeps the last good
palette on failure, so atomic writes (`mv`) are safe.

<a id="equisdots-integration" name="equisdots-integration"></a>

### Equisdots integration

As a reference desktop integration, the engine can read the equisdots palette
directly when explicitly requested with `--palette equisdots[:<slug>]`:

| Item | Path or rule |
|------|--------------|
| Settings | `~/.config/hypr/settings.json` |
| Active slug | `settings.json` `bar.palette`, then legacy `dock.palette`, then `x` |
| Palette file | `~/.config/hypr/scripts/quickshell/dock/palettes/<slug>.json`, searched recursively one level (for example `community/`) |
| Metadata to skip | `index.json` and `schema.json` |
| Schema | Same base16 schema described above |

This is an explicit opt-in source, not the default. Any desktop can satisfy the
xwww file contract with the copy hook shown above.

<a id="crossfade-on-palette-changes" name="crossfade-on-palette-changes"></a>

## Crossfade on palette changes

When `scene run` detects a palette change and `--palette-fade` is non-zero, the
previous frame is blended out over the new one instead of snapping. The fade
clock starts after the scene has painted the new frame, so a slow first frame
does not eat the animation, and animated scenes keep moving during the fade.
The clean new frame is restored when the fade ends. Scenes that only repaint
when their inputs change also fade correctly, because the runtime restores the
fade target before the scene paints.

Frames are delivered on change only: the run loop calls `render` at `--fps` and
pushes a frame to the daemon only when the canvas actually painted something
since the previous frame. Idle scenes therefore cost almost no CPU.

<a id="transport-and-performance" name="transport-and-performance"></a>

## Transport and performance

Each frame is sent as a normal image request with `no_cache = true` and a
synthetic path `scene:<path>`, so scenes never touch the on-disk wallpaper
cache and `xwww query` still shows something meaningful. The first frame
carries the requested entry transition; every later frame is instant.

A single 2560x1440 ARGB frame is about 14.7 MB raw (about 33 MB at 4K). The
client compresses frames with LZ4 over the existing IPC path and the daemon
blits them into its `wl_shm` buffer, which is a good fit for the 5-15 fps range
that palette-aware scenes need. A full streaming protocol for sustained 30-60
fps remains future work (see the [Roadmap](#roadmap)).

<a id="integrations" name="integrations"></a>

## Integrations

Scenes coexist with the rest of a scripted desktop:

- While a scene runs, `xwww query` reports `scene:<path>` as the displayed
  content, and `xwww screenshot` still captures the current frame.
- Wallpaper switchers that grep `xwww query` for a file path should treat
  `scene:` entries as opaque, or start/stop `xwww scene run` alongside their
  still-image and video producers.
- A systemd user unit is the recommended way to keep a scene running for a
  session; monitor hotplug is handled by the daemon, and the runtime renders
  one scene instance per output size.

<a id="testing" name="testing"></a>

## Testing

The engine has a unit-test suite that runs without a compositor:

```sh
cargo test -p xwww --no-default-features --features scene
```

It covers palette parsing and specs, canvas rasterization, scene lifecycle,
palette crossfades, timeouts and error mapping. Layer-shell interaction and
real palette writes must be verified on a live compositor: render a frame to
PNG first, then `scene run --fps 1` against a running daemon, and finally write
`~/.config/xwww/palette.json` to confirm the scene restyles within a second.

<a id="roadmap" name="roadmap"></a>

## Roadmap

Implemented today: the JavaScript runtime, the canvas API (shapes, paths,
gradients, assets, region remap and text), palette and clock providers, change
driven frame delivery, palette crossfades and entry transitions.

Planned, in order of priority:

1. **Compositor events** - a Hyprland `socket2` provider (workspace,
   activewindow, monitor, fullscreen) feeding `ctx.events`, enabling reactive
   scenes. The provider interface is designed for this.
2. **More providers** - image/dominant-color, pywal/wallust, battery and CPU
   load.
3. **Streaming IPC** - a ring-buffer frame channel if measured performance
   makes the per-frame request path insufficient.

Pointer input is intentionally out of scope: the wallpaper surface stays
click-through.

<a id="design-decisions" name="design-decisions"></a>

## Design decisions

- **QuickJS through `rquickjs`** as the scripting engine. The project already
  links C code (liblz4, optionally FFmpeg), and per-frame JavaScript math stays
  cheap. A pure-Rust engine such as Boa would simplify cross-compilation but is
  much slower.
- **The runtime lives in the client**, and the daemon stays unchanged. The
  daemon only receives fully rendered frames, which keeps its attack surface
  small and avoids coupling the IPC to a scripting ABI.
- **External triggers only.** The layer surface has an empty input region,
  which is what keeps wallpapers click-through; scenes react to palettes, time
  and (in the future) compositor events instead of pointer input.
- **Frames travel over the existing image pipeline** until a streaming
  protocol proves necessary. This costs one shared-memory mapping per frame but
  needs no protocol changes, and `query`, `screenshot` and the cache
  semantics keep working.
- **The xwww palette file is the default palette source**, with the equisdots
  palette as a compatibility fallback. Scenes do not depend on one desktop's
  layout: any desktop can satisfy the contract with a one-line copy.
- **Scene frames bypass the on-disk cache.** Generated frames have no natural
  file path, and caching them would thrash `~/.cache`.

<a id="file-map" name="file-map"></a>

## File map

| File | Contents |
|------|----------|
| `client/src/cli.rs` | `Scene` subcommand with `check`, `render` and `run`. |
| `client/src/main.rs` | Dispatch, PNG output and the frame-sending loop. |
| `client/src/palette_source.rs` | `ScenePalette`, palette specs and parsing. |
| `client/src/scene/mod.rs` | Engine, frame lifecycle and crossfade logic. |
| `client/src/scene/runtime.rs` | QuickJS context, bindings, limits and timeouts. |
| `client/src/scene/canvas.rs` | tiny-skia wrapper: shapes, paths, gradients, transforms, assets, remap and text. |
| `client/src/scene/providers.rs` | `PaletteProvider` (1 s polling) and `now_ms()`. |

<div align="center">
  <a href="#top">Back to top</a>
</div>
