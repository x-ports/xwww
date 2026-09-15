xwww-restore(1)

# NAME
xwww-restore

# SYNOPSIS
*xwww restore*

# OPTIONS

*-a*,*--all*
	Send this command to all active *xwww-daemon* namespaces.

*-n*,*--namespace* <namespace>
	Which wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww restore -n <namespace1> -n <namespace 2>*.

	This affects the name of the *xwww-daemon* socket we will use to
	communicate with. Specifically, our socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.socket_.

	Some compositors can have several different wallpapers per output. This
	allows you to differentiate between them. Most users will probably not have
	to set anything in this option.


*-o*, *--outputs*
	Comma separated list of outputs to restore. Use *xwww query* to know which
	outputs are currently being used.

	If it isn't set, all outputs will be restored.

*-h*, *--help*
	Print help (see a summary with '-h')

# DESCRIPTION

Restores the last displayed image on the specified outputs.

This can be used to split initialization (with *xwww init --no-daemon*) and
cache loading into different steps, in order to avoid race condition traps.

You can also use this command to restore the last displayed image when
reconnecting a monitor.

# SEE ALSO
*xwww-clear-cache*(1)
