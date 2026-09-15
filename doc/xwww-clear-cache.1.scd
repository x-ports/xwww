xwww-clear-cache(1)

# NAME
xwww-clear-cache

# SYNOPSIS
*xwww clear-cache*

# OPTIONS

*-h*, *--help*
	Print help (see a summary with '-h')

# DESCRIPTION

Deletes the *xwww* cache directory.

The cache resides at _$XDG_CACHE_HOME/xwww_ or _$HOME/.cache/xwww_
if $XDG_CACHE_HOME does not exist. In this directory, we create another
directory corresponding to *xwww*s current version number. In there, we have
files that cache the last displayed image and animation frames for animated
wallpapers. There is one file per output.

Note that *xwww* will automatically delete any files and directories in 
_$XDG_CACHE_HOME/xwww_ that do not correspond to the exact current *xwww*
version number. This is to ensure we only keep cache files for the latest
*xwww* version.

# SEE ALSO
*xwww-img*(1)
