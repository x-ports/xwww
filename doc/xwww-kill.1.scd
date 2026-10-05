xwww-kill(1)

# NAME
xwww-kill

# SYNOPSIS
*xwww kill* [OPTIONS]

# OPTIONS

*-a*, *--all*
	Kill all active *xwww-daemon* namespaces.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww kill -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Kills the daemon. This is the recommended way of doing it, because the client
waits until the socket file has been deleted, which confirms that the daemon
exited.

Sending SIGTERM would also work correctly, but SIGKILL leaves the socket file
behind. That is not a big problem; it only causes a warning the next time the
daemon is initialized.

# SEE ALSO
*xwww*(1) *xwww-daemon*(1)
