xwww-query(1)

# NAME
xwww-query

# SYNOPSIS
*xwww query* [OPTIONS]

# OPTIONS

*-a*, *--all*
	Query all active *xwww-daemon* namespaces.

*-j*, *--json*
	Print the information in JSON format.

*-n*, *--namespace* <namespace>
	Which Wayland namespace to query.

	You can specify this option multiple times to query multiple namespaces at
	once: *xwww query -n <namespace1> -n <namespace2>*.

	This affects the name of the *xwww-daemon* socket used to communicate
	with. Specifically, the socket name is
	_${WAYLAND_DISPLAY}-xwww-daemon.<specified namespace>.sock_.

	Namespaces let several independent wallpapers run on the same Wayland
	display. Most users will not need to set this option.

*-h*, *--help*
	Print help (see a summary with '-h').

# DESCRIPTION

Asks the daemon to print output information (names, dimensions, scale factor
and the content currently displayed).

You may use this to find out valid values for the *xwww-img --outputs* option.
If you want more detailed information about your outputs, try *wlr-randr*.

# OUTPUT FORMAT

Plain output prints one line per output:

```
NAMESPACE: OUTPUT: SIZE, scale: SCALE, currently displaying: IMAGE_OR_COLOR
```

where *SIZE* is in the format *WxH* (for example *1920x1080*) and
*IMAGE_OR_COLOR* is either _image: IMAGENAME_ or _color: RGB_ (uppercase hex).

# JSON FORMAT

With *--json*, the command prints an object keyed by namespace. Each value is
an array of outputs:

```
{
    "namespace": [
        {
            "name": "OUTPUT",
            "width": 1920,
            "height": 1080,
            "scale": 1,
            "displaying": {
                "image": "/path/to/image"
            }
        }
    ]
}
```

For a solid color, the _displaying_ object contains a _color_ key with a
`#rrggbb` value instead.

# SEE ALSO
*xwww*(1) *xwww-img*(1)
