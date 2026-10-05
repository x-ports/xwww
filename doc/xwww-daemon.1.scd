xwww-daemon(1)

# NAME
xwww-daemon - The xwww Wayland wallpaper daemon

# SYNOPSIS
*xwww-daemon* [OPTIONS]

# OPTIONS

*-f*, *--format* <argb|abgr|rgb|bgr>
	Force the daemon to use a specific wl_shm format.

	By default, *xwww-daemon* will use _argb_, because it is the most widely
	supported format. Generally speaking, formats with 3 channels use 3/4 of
	the memory of formats with 4 channels. Also, _bgr_ formats are more
	efficient than _rgb_ formats because no extra byte swap is required when
	decoding the image (though the difference is unnoticeable).

*-l*, *--layer* <background|bottom>
	Which layer to display the wallpaper in. Defaults to _background_.

	The layers _top_ and _overlay_ are not accepted, because those would put
	an image on top of everything else and make the desktop unusable.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to append to *xwww-daemon*.

	The resulting namespace will be *xwww-daemon<specified namespace>*. This
	also affects the name of the *xwww-daemon* socket used to communicate with
	the client. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display (for example one per workspace). Most users will not need to set
	this option.

*--no-cache*
	Do not load the cached wallpaper for each output.

	Useful if you always want to select which image *xwww* displays manually
	with *xwww img*.

*-q*, *--quiet*
	Only log errors.

*-h*, *--help*
	Print help (see a summary with '-h').

*-V*, *--version*
	Print version.

# DESCRIPTION

The *xwww-daemon* runs continuously, drawing wallpapers on the
*wlr-layer-shell* protocol and waiting for commands on a UNIX socket. The
socket is created at
_${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}-xwww-daemon[.<namespace>].sock_, or at
_/run/user/<uid>/${WAYLAND_DISPLAY}-xwww-daemon[.<namespace>].sock_ when
$XDG_RUNTIME_DIR is not set. The daemon creates and deletes that file when it
is initialized or killed.

When an output is created or reconfigured and caching is enabled, the daemon
restores the last wallpaper displayed on it by reading the cache maintained by
the client.

# SEE ALSO
*xwww*(1) *xwww-img*(1) *xwww-query*(1)
