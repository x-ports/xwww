xwww(1)

# NAME
xwww - A Wayland wallpaper daemon, controlled at runtime

# SYNOPSIS
*xwww* <COMMAND>

# COMMANDS

*clear*
	Fills the specified outputs with the given color

*restore*
	Restores the last displayed image on the specified outputs

*clear-cache*
	Deletes the *xwww* cache directory

*img*
	Sends an image (or animated gif) for the daemon to display

*kill*
	Kills the daemon

*query*
	Asks the daemon to print output information (names and dimensions)

*help [COMMAND]*
	Print help or the help of the given command

# OPTIONS

*-h*, *--help*
	Print help (see a summary with '-h')

*-V*, *--version*
	Print  version

# DESCRIPTION
*xwww* is a wallpaper manager that lets you change what your monitors display as
a background by controlling the *xwww-daemon* at runtime.

It supports animated gifs and putting different stuff in different monitors. I
also did my best to make it as resource efficient as possible.

To start, begin by running *xwww-daemon*. That will set up the *xwww-daemon*.
Then, you can send images to be displayed with *xwww img*. To kill the daemon,
use *xwww kill*.

*Note that xwww only works in a compositor that implements the layer-shell
protocol*. Typically, _wlr-roots_ based compositors.

# FILES
*xwww* will create the following files in your system:
	- A socket in _${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}-xwww-daemon.<specified
	namespace>.socket_,  or
	_/tmp/xwww/${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.socket_, if
	$XDG_RUNTIME_DIR is not set.
	- Cache files in _$XDG_CACHE_HOME/xwww_ or _$HOME/.cache/xwww_ if
	  $XDG_CACHE_HOME does not exist. These are used to set the wallpaper to the
	  previous image when a monitor is (re)connected or turned on.

# SEE ALSO
*xwww-daemon*(1) *xwww-clear*(1) *xwww-img*(1) *xwww-kill*(1) *xwww-query*(1)
