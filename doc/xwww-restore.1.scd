xwww-restore(1)

# NAME
xwww-restore

# SYNOPSIS
*xwww restore* [OPTIONS]

# OPTIONS

*-a*, *--all*
	Restore all active *xwww-daemon* namespaces.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww restore -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*-o*, *--outputs*
	Comma separated list of outputs to restore. Use *xwww query* to know which
	outputs are currently available.

	If it is not set, all outputs are restored.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Restores, from the cache, the last wallpaper each selected output displayed.
The daemon already does this automatically when an output is (re)connected, so
this command is mainly useful to force a reload or to split initialization into
separate steps in scripts.

The cache is written by *xwww img* under _$XDG_CACHE_HOME/xwww/<version>_ or
_$HOME/.cache/xwww/<version>_. Wallpapers set with *xwww clear* are not cached;
use *xwww img 0xrrggbbaa* if you want a solid color to be restored on
reconnect.

# SEE ALSO
*xwww*(1) *xwww-clear-cache*(1) *xwww-img*(1)
