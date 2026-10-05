xwww(1)

# NAME
xwww - A Wayland wallpaper daemon, controlled at runtime

# SYNOPSIS
*xwww* <COMMAND> [OPTIONS]

# COMMANDS

*clear*
	Fills the specified outputs with the given color.

*restore*
	Restores the last displayed wallpaper on the specified outputs from the
	cache.

*clear-cache*
	Deletes the *xwww* cache directory.

*img*
	Sends an image, animated image, video or solid color for the daemon to
	display.

*toggle*
	Toggles the paused state of the daemon.

*pause*
	Pauses animations.

*unpause*
	Resumes animations.

*kill*
	Kills the daemon.

*query*
	Asks the daemon to print output information (names and dimensions).

*palette*
	Extracts the dominant colors of an image or of the current wallpaper.

*slideshow*
	Cycles through the images of a directory on a timer.

*random*
	Sets a single random image from a directory.

*screenshot*
	Captures the currently displayed wallpaper to a PNG file.

*scene*
	Renders a JavaScript scene as the wallpaper (requires a build with the
	*scene* feature).

*help [COMMAND]*
	Print help or the help of the given command.

# OPTIONS

*-h*, *--help*
	Print help (see a summary with '-h').

*-V*, *--version*
	Print version.

# DESCRIPTION
*xwww* is a Wayland wallpaper daemon, controlled at runtime. It draws
wallpapers through the *wlr-layer-shell* protocol and lets you change what your
monitors display without restarting the daemon.

It supports static images, animated GIF/WebP/APNG files, videos, solid colors,
transitions, palette effects and JavaScript scenes, and it can target
individual outputs or independent daemon namespaces.

To start, run *xwww-daemon*. Then send wallpapers with *xwww img*. To stop the
daemon, use *xwww kill*.

*Note that xwww only works in a compositor that implements the layer-shell
protocol*. Typically, _wlroots_ based compositors. GNOME is not supported.

# FILES
*xwww* creates the following files:
	- A socket in
	_${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}-xwww-daemon[.<namespace>].sock_, or
	_/run/user/<uid>/${WAYLAND_DISPLAY}-xwww-daemon[.<namespace>].sock_ when
	$XDG_RUNTIME_DIR is not set.
	- Cache files in _$XDG_CACHE_HOME/xwww/<version>_, or
	_$HOME/.cache/xwww/<version>_ when $XDG_CACHE_HOME is not set. These store
	the last wallpaper per output and preprocessed animation frames, and the
	daemon reloads them when a monitor is (re)connected or turned on.

# SEE ALSO
*xwww-daemon*(1) *xwww-clear*(1) *xwww-clear-cache*(1) *xwww-img*(1)
*xwww-kill*(1) *xwww-palette*(1) *xwww-pause*(1) *xwww-query*(1)
*xwww-random*(1) *xwww-restore*(1) *xwww-scene*(1) *xwww-screenshot*(1)
*xwww-slideshow*(1) *xwww-toggle*(1) *xwww-unpause*(1)
