xwww-toggle(1)

# NAME
xwww-toggle

# SYNOPSIS
*xwww toggle* [OPTIONS]

# OPTIONS

*-a*, *--all*
	Send this command to all active *xwww-daemon* namespaces.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww toggle -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Flips the paused state of the daemon: if animations are running, they pause on
the last rendered frame; if they are paused, they resume. Use *xwww pause* or
*xwww unpause* when you need to set the state explicitly.

# SEE ALSO
*xwww*(1) *xwww-pause*(1) *xwww-unpause*(1)
