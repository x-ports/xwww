xwww-palette(1)

# NAME
xwww-palette

# SYNOPSIS
*xwww palette* [OPTIONS] [IMAGE]

# OPTIONS

*IMAGE*
	Image file to analyze. When omitted, the wallpaper currently displayed by
	the daemon is used instead.

*-c*, *--count* <COUNT>
	Number of colors to output. Defaults to 8.

*--json*
	Emit a JSON array instead of plain hex lines.

*-o*, *--output* <OUTPUT>
	When querying the daemon, use the wallpaper of this output. Defaults to
	the first available output.

*-n*, *--namespace* <namespace>
	The daemon namespace to query. Defaults to the empty namespace.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Extracts the dominant colors of an image, or of the wallpaper the daemon is
currently displaying. This is useful for generating color schemes that match
the wallpaper, for example to theme a shell, terminal or status bar.

The image is downsampled to at most 128x128, pixels are bucketed into a coarse
histogram (4 bits per channel) and the most frequent buckets are averaged to
produce smooth representative colors, ordered from most to least frequent. SVG
files are rendered before analysis. A solid-color wallpaper cannot be analyzed
and produces an error.

# OUTPUT

Plain mode prints one uppercase hex color per line:

```
#2E3440
#81A1C1
#BF616A
```

With *--json*, an array of objects with *hex*, *r*, *g* and *b* fields is
printed instead.

# SEE ALSO
*xwww*(1) *xwww-img*(1) *xwww-query*(1)
