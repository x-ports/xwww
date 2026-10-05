xwww-scene(1)

# NAME
xwww-scene

# SYNOPSIS
*xwww scene* <check|render|run> [OPTIONS] <SCRIPT>

# DESCRIPTION

Renders a JavaScript scene as the wallpaper. A scene is a plain JavaScript file
with two optional top-level functions:

```
function setup(ctx) { /* runs once */ }
function render(t, ctx) { /* runs once per frame; t is in seconds */ }
```

Scenes draw through a global *canvas* object (shapes, paths, gradients, images
and text) and can read the active color palette from *ctx.palette*. The client
renders the frames and sends them to the daemon through the regular image
pipeline.

This command requires a build with the *scene* feature. Builds without it list
the subcommand but fail with a clear message.

# SUBCOMMANDS

*check* <SCRIPT>
	Compiles the scene and exits.

*render* <SCRIPT> [OPTIONS]
	Renders a single frame to a PNG file.

*run* <SCRIPT> [OPTIONS]
	Renders the scene continuously and sends the frames to the daemon.

# COMMON OPTIONS

*--palette* <SOURCE>
	Palette source: _xwww[:<path>]_, _equisdots[:<slug>]_, _file:<path>_ or
	_command:<cmd>_. Defaults to the xwww palette file, then the equisdots
	palette, then a neutral fallback.

*--timeout-ms* <MILLISECONDS>
	Per-frame JavaScript execution budget before the call is aborted. Defaults
	to 100.

*--asset* <PATH>
	Extra directory (or file) whose images the scene may load with
	*canvas.image*. The scene's own directory is always allowed. Repeat the
	flag for several paths.

# OPTIONS FOR render

*-o*, *--output* <FILE>
	Output PNG file. Defaults to _scene.png_.

*--size* <WxH>
	Canvas size in physical pixels. Defaults to _2560x1440_.

# OPTIONS FOR run

*--fps* <N>
	Frames per second rendered and pushed to the daemon. Defaults to 10.

*--palette-fade* <MILLISECONDS>
	Crossfade duration when the active palette changes. Set to 0 to disable.
	Defaults to 800.

*-o*, *--outputs*
	Comma separated list of outputs to display the scene at. Defaults to all
	outputs.

*-n*, *--namespace* <namespace>
	The daemon namespace to send frames to. Defaults to the empty namespace.

*--transition-type* <TRANSITION_TYPE>
	Entry transition for the first frame, using the same values as *xwww-img*.
	Every later frame is instant. Defaults to _none_.

	The remaining entry-transition flags (_--transition-step_,
	_--transition-duration_, _--transition-fps_, _--transition-angle_,
	_--transition-pos_, _--transition-bezier_, _--transition-wave_ and
	_--invert-y_) are also accepted; see *xwww-img*(1).

# EXAMPLES

```
xwww scene check  ~/scenes/clock.js
xwww scene render ~/scenes/clock.js -o clock.png --size 2560x1440
xwww scene run    ~/scenes/clock.js --fps 10 --palette xwww
xwww scene run    ~/scenes/clock.js --transition-type fade --transition-duration 2
```

# SEE ALSO
*xwww*(1) *xwww-img*(1) *xwww-screenshot*(1)
