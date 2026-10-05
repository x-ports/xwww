xwww-slideshow(1)

# NAME
xwww-slideshow

# SYNOPSIS
*xwww slideshow* [OPTIONS] <DIRECTORY>

# OPTIONS

*DIRECTORY*
	Directory to scan. Only the top level is scanned, and files are selected
	by extension.

*-i*, *--interval* <SECONDS>
	Time between wallpapers. Defaults to 60.

*-r*, *--random*
	Shuffle the order instead of going in alphabetical order.

*-t*, *--transition-type* <TRANSITION_TYPE>
	Transition used between wallpapers. Defaults to _any_. See *xwww-img*(1)
	for the available transition types.

*--resize* <no|crop|fit|stretch>
	Resize strategy for each image. Defaults to _crop_.

*-n*, *--namespace* <namespace>
	The daemon namespace to use. Defaults to the empty namespace.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Cycles through the images of a directory on a timer. The command runs until it
is interrupted with Ctrl-C. If an individual image fails to load, a warning is
printed to standard error and the loop continues with the next file.

The slideshow always uses the _Lanczos3_ filter and targets all outputs. Use
*xwww img* directly when you need per-output control or video files.

Supported extensions are: _png_, _jpg_, _jpeg_, _gif_, _webp_, _bmp_, _tiff_,
_tif_, _avif_, _svg_, _jxl_, _qoi_ and _exr_. The _avif_ and _jxl_ decoders are
optional build features.

# SEE ALSO
*xwww*(1) *xwww-img*(1) *xwww-random*(1)
