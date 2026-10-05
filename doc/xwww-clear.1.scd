xwww-clear(1)

# NAME
xwww-clear

# SYNOPSIS
*xwww clear* [OPTIONS] <COLOR>

# OPTIONS

*-a*, *--all*
	Send this command to all active *xwww-daemon* namespaces.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to send this command to.

	You can specify this option multiple times to send commands to multiple
	namespaces at once: *xwww clear -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*-o*, *--outputs*
	Comma separated list of outputs to fill. Use *xwww query* to know which
	outputs are currently available.

	If it is not set, all outputs are filled.

*-h*, *--help*
	Print help (see a summary with '-h').

# COLOR

The color to fill the screen with, in *RRGGBB* or *RRGGBBAA* hex format,
without a prepended '#'. Defaults to *000000ff* (opaque black).

# DESCRIPTION
Fills the specified outputs with the given color.

This does not update the wallpaper cache, so if you want a color to survive a
monitor reconnect, set it with *xwww img 0xrrggbbaa* instead:

```
xwww-daemon && xwww img 0x1a804aff
```

# SEE ALSO
*xwww*(1) *xwww-query*(1)
