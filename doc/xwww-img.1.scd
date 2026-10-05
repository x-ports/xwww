xwww-img(1)

# NAME
xwww-img

# SYNOPSIS
*xwww img* [OPTIONS] <path/to/img>

# OPTIONS

*-a*, *--all*
	Send this command to all active *xwww-daemon* namespaces.

*-f*, *--filter* <FILTER>
	Filter to use when scaling images.

	Available options are:

	_Nearest_ | _Bilinear_ | _CatmullRom_ | _Mitchell_ | _Lanczos3_

	These are offered by the fast_image_resize crate
	(https://docs.rs/fast_image_resize/). _Nearest_ is recommended for pixel
	art, and only for pixel art; it is also the fastest filter. For other
	images, one of the last three usually works best, though some
	experimentation will be necessary.

	Note that with *--resize no* the filter has no effect.

	Default is _Lanczos3_.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww img -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*--no-cache*
	Do not update the cache. The previous cache entry for each output is left
	untouched.

*--no-resize*
	Deprecated alias for *--resize no*.

*--resize* <no|crop|fit|stretch>
	Whether to resize the image and the method by which to resize it.

	Possible values:
		- _no_:       Do not resize the image
		- _crop_:     Resize the image to fill the whole screen, cropping out parts that do not fit
		- _fit_:      Resize the image to fit inside the screen, preserving the original aspect ratio
		- _stretch_:  Resize the image to fit inside the screen, without preserving the original aspect ratio

	With _no_, the image is centered and padded with *--fill-color*.

	Default is _crop_.

*--crop-gravity* <CROP_GRAVITY>
	Specify which portion of the image to anchor when cropping. Only used with
	*--resize crop*.

	Possible values:
		- _center_:       Crops top and bottom equally and left and right equally
		- _top-left_:     Crops bottom and right
		- _top_:          Crops bottom, and left and right equally
		- _top-right_:    Crops bottom and left
		- _left_:         Crops right, and top and bottom equally
		- _right_:        Crops left, and top and bottom equally
		- _bottom-left_:  Crops top and right
		- _bottom_:       Crops top, and left and right equally
		- _bottom-right_: Crops top and left

	Default is _center_.

*--fill-color* <RRGGBB[AA]>
	Color used to fill the padding when the image does not fill the screen.

	Accepts *rrggbb* or *rrggbbaa* hex values, without a prepended '#'.

	Default is _000000ff_.

*-o*, *--outputs*
	Comma separated list of outputs to display the image at. Use *xwww query*
	to know which outputs are currently available.

	If it is not set, the image is displayed on all outputs.

*--blur* <RADIUS>
	Gaussian blur radius in pixels, applied after scaling.

	Set to 0 to disable. Applies only to static images; animated images and
	videos are left untouched.

	Default is 0.

*--dim* <FACTOR>
	Dim factor applied after scaling, in the range \[0.0, 1.0]. A value of
	1.0 leaves the image unchanged and 0.0 turns it fully black. The alpha
	channel is never modified.

	Applies only to static images.

	Default is 1.0.

*--map-palette* <SOURCE>
	Recolors the wallpaper through a palette: every pixel's luminance is
	mapped onto a gradient built from the palette colors, background and
	foreground.

	Available sources:
		- _xwww_ / _xwww:<path>_: the xwww palette file
		  (_$XDG_CONFIG_HOME/xwww/palette.json_ by default).
		- _equisdots_ / _equisdots:<slug>_: the equisdots desktop palette
		  (_~/.config/hypr/settings.json_ plus the _dock/palettes_ directory).
		- _file:<path>_ (or a bare path): palette JSON (base16 schema) or a
		  text file with one _#rrggbb_ per line.
		- _command:<cmd>_: parses the stdout of a command like _file_.

	Applies only to static images.

*--map-strength* <FACTOR>
	How strongly *--map-palette* overrides the original colors, in
	\[0.0, 1.0]. 0.0 keeps the original image and 1.0 fully replaces it.

	Default is 1.0.

*-t*, *--transition-type* <TRANSITION_TYPE>
	\[Environment Variable: $XWWW_TRANSITION]

	Sets the type of transition. Default is _simple_, which fades into the new
	image.

	Possible transitions are:

[- _none_
:- _simple_
:- _fade_
:- _left_
:- _right_
:- _top_
:- _bottom_
:- _wipe_
:- _wave_
:- _grow_
:- _center_
:- _any_
:- _outer_
:- _random_
:- _glitch_
:- _decrypt_
:- _dissolve_
:- _clock_
:- _zoom_
:- _pixelate_
:- _ripple_
:- _blinds_
:- _spiral_
:- _static_
:- _parallax_
:- _parallax-left_
:- _parallax-right_
:- _parallax-invert_
:- _melt_
:- _shatter_

	_none_ completes the transition instantly.

	_simple_ fades by stepping each pixel channel toward the target, with no
	fixed duration.

	_fade_ is like _simple_ but driven by the *--transition-bezier* curve.

	The _left_, _right_, _top_ and _bottom_ options make the transition happen
	from that side of the screen.

	_wipe_ is similar to _left_ but allows specifying the angle with
	*--transition-angle*.

	_wave_ is similar to _wipe_ but the sweeping line is wavy; use
	*--transition-wave* to control it.

	_grow_ grows a circle from a center set with *--transition-pos*.

	_center_ is an alias to _grow_ with the position set to the center of the
	screen.

	_any_ randomly picks _grow_ or _outer_ at a random position.

	_outer_ is like _grow_, but the circle shrinks instead of growing.

	_glitch_ reveals the new image with horizontal tearing, channel shifts and
	noise that decay over the duration.

	_decrypt_ reveals the image in 32x32 blocks in pseudo-random order.

	_dissolve_ reveals the image pixel by pixel in pseudo-random order.

	_clock_ sweeps the image in angularly around *--transition-pos*.

	_zoom_ zooms out from *--transition-pos* (default: center).

	_pixelate_ shows 48x48 flat color blocks in raster order, then sharpens.

	_ripple_ reveals the image with concentric wavy rings.

	_blinds_ opens horizontal venetian blinds in staggered order.

	_spiral_ unwinds the image like a spiral arm.

	_static_ dissolves the image through television-like static.

	_parallax_, _parallax-left_, _parallax-right_ and _parallax-invert_ push
	the old image out while both move at different speeds.

	_melt_ reveals the image from the top with ragged, column-specific drips.

	_shatter_ reassembles the image from randomly flying tiles.

	_random_ selects one of _fade_, _wave_, _outer_, _grow_, _glitch_,
	_decrypt_, _dissolve_, _clock_ or _zoom_ at random, with random shape
	parameters.

*--transition-step* <1-255>
	\[Environment Variable: $XWWW_TRANSITION_STEP]

	How fast the transition approaches the new image.

	The transition logic adds to or subtracts from the current RGB values
	until the old image becomes the new one. This controls by how much.

	For example, if pixel A is 000010 and must transition to pixel B, which is
	000020, with a *transition-step* of 2, pixel A becomes 000012 in one frame,
	000014 in the next, and so on.

	Larger values make the transition faster but more abrupt. A value of 255
	switches to the new image immediately.

	Default is 90. If *--transition-type* is _simple_, the default is 2.

*--transition-duration* <seconds>
	\[Environment Variable: $XWWW_TRANSITION_DURATION]

	How long time-based transitions take to complete, in seconds. Decimal
	values are accepted.

	This has no effect on the _simple_ and _none_ transitions.

	Default is 3.

*--transition-fps* <frames per second>
	\[Environment Variable: $XWWW_TRANSITION_FPS]

	Frame rate for the transition effect.

	There is no point in setting this higher than what your monitor supports.

	Note this is different from *--transition-step*, which controls how far the
	image approaches the target every frame.

	Default is 30.

*--transition-angle* <degrees>
	\[Environment Variable: $XWWW_TRANSITION_ANGLE]

	Angle of the wipe for the _wipe_ and _wave_ transitions.

	The angle is in degrees, where 0 is right to left, 90 is top to bottom and
	270 is bottom to top.

	Default is 45.

*--transition-pos* <x,y>
	\[Environment Variable: $XWWW_TRANSITION_POS]

	Center used by circular and radial transitions such as _grow_, _outer_,
	_zoom_ and _clock_.

	Position values can be given as percentages or pixels: float values are
	interpreted as percentages and integer values as pixels. For example,
	_0.5,0.5_ means 50% of the screen width and 50% of the screen height, while
	_200,400_ means 200 pixels from the left and 400 pixels from the bottom.

	The value can also be one of the aliases _center_, _top_, _left_, _right_,
	_bottom_, _top-left_, _top-right_, _bottom-left_ or _bottom-right_.

	Default is _center_.

*--invert-y*
	\[Environment Variable: $INVERT_Y]

	Inverts the y position sent in *--transition-pos*.

*--transition-bezier* <f1,f2,f3,f4>
	\[Environment Variable: $XWWW_TRANSITION_BEZIER]

	Bezier curve used for the animation. https://cubic-bezier.com is a good
	website to get these values from; _0.0,0.0,1.0,1.0_ is a linear animation.

	Default is _.54,0,.34,.99_.

*--transition-wave* <width,height>
	\[Environment Variable: $XWWW_TRANSITION_WAVE]

	Only used by the _wave_ transition; controls the width and height of each
	wave.

	Default is _20,20_.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION
Sends an image, animated image (GIF/WebP/APNG), video or solid color for the
daemon to display. The image argument may be:

	- a path to a file;
	- a solid color as _0xrrggbb_ or _0xrrggbbaa_; or
	- *-* to read the image from standard input.

# ABOUT THE CACHE

Unless *--no-cache* is given, the image path and the resize, crop-gravity and
filter settings are cached per output at _$XDG_CACHE_HOME/xwww/<version>_ or
_$HOME/.cache/xwww/<version>_ when $XDG_CACHE_HOME is not set. The daemon uses
that entry to restore the wallpaper when an output is (re)connected, and
*xwww restore* does the same on demand.

The cache also keeps preprocessed versions of animated wallpapers, so a large
GIF or video only pays the processing cost the first time. If you load many
large animations, the cache can grow; run *xwww clear-cache* to delete it.

# SEE ALSO
*xwww*(1) *xwww-clear-cache*(1) *xwww-daemon*(1) *xwww-query*(1)
