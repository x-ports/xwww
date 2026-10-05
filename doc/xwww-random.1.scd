xwww-random(1)

# NAME
xwww-random

# SYNOPSIS
*xwww random* [OPTIONS] <DIRECTORY>

# OPTIONS

*DIRECTORY*
	Directory to choose from. Only the top level is scanned, and files are
	selected by extension.

*-t*, *--transition-type* <TRANSITION_TYPE>
	Transition used when setting the wallpaper. Defaults to _any_. See
	*xwww-img*(1) for the available transition types.

*--resize* <no|crop|fit|stretch>
	Resize strategy for the image. Defaults to _crop_.

*-n*, *--namespace* <namespace>
	The daemon namespace to use. Defaults to the empty namespace.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Sets a single random image from a directory and exits. This is the one-shot
counterpart of *xwww slideshow*, and it accepts the same file extensions.

# SEE ALSO
*xwww*(1) *xwww-img*(1) *xwww-slideshow*(1)
