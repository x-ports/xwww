xwww-clear-cache(1)

# NAME
xwww-clear-cache

# SYNOPSIS
*xwww clear-cache*

# OPTIONS

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Deletes the *xwww* cache directory.

The cache resides at _$XDG_CACHE_HOME/xwww_ or _$HOME/.cache/xwww_ when
$XDG_CACHE_HOME is not set. Inside it, each *xwww* version gets its own
directory holding one file per output with the last displayed wallpaper and the
settings used to display it, plus cached animation frames for animated
wallpapers.

*xwww* automatically deletes version directories that do not match the running
version, so only cache data for the current release is kept. Use this command
to reclaim space when animations have made the cache large.

# SEE ALSO
*xwww*(1) *xwww-img*(1)
