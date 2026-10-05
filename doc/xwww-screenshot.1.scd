xwww-screenshot(1)

# NAME
xwww-screenshot

# SYNOPSIS
*xwww screenshot* [OPTIONS] <OUTPUT_FILE>

# OPTIONS

*OUTPUT_FILE*
	Destination path. The file is encoded as a PNG.

*-o*, *--monitor* <OUTPUT>
	Which output to capture. Defaults to the first available output.

*-n*, *--namespace* <namespace>
	The daemon namespace to query. Defaults to the empty namespace.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Captures the wallpaper the daemon is currently displaying on an output and
saves it as a PNG file. This captures the wallpaper only, not the full
desktop, which makes it useful for previews and scripts.

The daemon sends the current canvas buffer back as raw RGB, dropping any alpha
channel, and the client re-encodes it. Animations and scenes are captured at
whatever frame is currently displayed.

# SEE ALSO
*xwww*(1) *xwww-query*(1)
