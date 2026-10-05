<a id="top" name="top"></a>

# Commands

`xwww <command> [options]`. Run `xwww --help` or `xwww <command> --help` for the
authoritative option list; this page summarizes every command and links to the
detailed guides.

<div align="center">
  <a href="README.md">Documentation index</a> &middot;
  <a href="../README.md">Project README</a> &middot;
  <a href="#common-options">Common options</a> &middot;
  <a href="#img">img</a> &middot;
  <a href="#clear">clear</a> &middot;
  <a href="#restore">restore</a> &middot;
  <a href="#clear-cache">clear-cache</a> &middot;
  <a href="#query">query</a> &middot;
  <a href="#palette">palette</a> &middot;
  <a href="#slideshow">slideshow</a> &middot;
  <a href="#random">random</a> &middot;
  <a href="#screenshot">screenshot</a> &middot;
  <a href="#scene">scene</a> &middot;
  <a href="#daemon-control">Daemon control</a>
</div>

<details open>
<summary><strong>On this page</strong></summary>

- [Common options](#common-options)
- [img](#img)
- [clear](#clear)
- [restore](#restore)
- [clear-cache](#clear-cache)
- [query](#query)
- [palette](#palette)
- [slideshow](#slideshow)
- [random](#random)
- [screenshot](#screenshot)
- [scene](#scene)
- [Daemon control](#daemon-control)

</details>

<a id="common-options" name="common-options"></a>

## Common options

| Option | Applies to | Meaning |
|--------|------------|---------|
| `-a, --all` | most commands | Send the command to every daemon namespace. |
| `-n, --namespace` | most commands | Target a daemon namespace; repeatable. The matching daemon must be started with `xwww-daemon --namespace <name>`. |
| `-o, --outputs` | `img`, `clear`, `restore`, `scene run` | Comma-separated output names (default: all outputs). |
| `-h, --help` | all | Print help. |
| `-V, --version` | all | Print the version. |

Output names come from `xwww query`. When several namespaces are given, the
command is sent to each of them in order.

<a id="img" name="img"></a>

## `img`

Sets an image, animated image, video or solid color as the wallpaper.

```sh
xwww img path/to/image.png
xwww img -o HDMI-A-1,DP-2 wallpaper.jpg
xwww img 0xff0000ff                  # solid color, no '#'
xwww img - < image.png               # read from stdin
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<image>` | - | File path, `0xrrggbb[aa]` color, or `-` for standard input. |
| `-o, --outputs` | all | Comma-separated output list. |
| `-n, --namespace` | `""` | Daemon namespace; repeatable. |
| `--no-cache` | off | Do not update the per-output cache. |
| `--resize` | `crop` | `crop`, `fit`, `stretch` or `no`. |
| `--crop-gravity` | `center` | Anchor used when cropping. |
| `--fill-color` | `000000ff` | Padding color when the image does not fill the output. |
| `-f, --filter` | `Lanczos3` | `Nearest`, `Bilinear`, `CatmullRom`, `Mitchell`, `Lanczos3`. |
| `--blur` | `0` | Gaussian blur radius (static images only). |
| `--dim` | `1.0` | Dim factor in `[0,1]` (static images only). |
| `--map-palette` | off | Recolor through a palette source (static images only). |
| `--map-strength` | `1.0` | Blend for `--map-palette`. |
| `-t, --transition-type` | `simple` | See [Transitions](transitions.md). |
| `--transition-step` | `2`/`90` | Step towards the target; `90` unless `simple`. |
| `--transition-duration` | `3` | Seconds for time-based effects. |
| `--transition-fps` | `30` | Transition frame rate. |
| `--transition-angle` | `45` | Degrees, for `wipe` and `wave`. |
| `--transition-pos` | `center` | Center for circular effects (`percent`, pixels or keyword). |
| `--transition-bezier` | `.54,0,.34,.99` | Curve for `fade` and circle effects. |
| `--transition-wave` | `20,20` | Wave size for `wave`. |
| `--invert-y` | off | Flips the y coordinate of `--transition-pos`. |
| `--no-resize` | - | Deprecated alias for `--resize no`. |

`--blur`, `--dim` and `--map-palette` are documented in
[Image effects](image-effects.md); the transition system in
[Transitions](transitions.md).

<a id="clear" name="clear"></a>

## `clear`

Fills outputs with a solid color.

```sh
xwww clear                # black
xwww clear 1a804a         # rrggbb
xwww clear 1a804aff -o DP-1
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<color>` | `000000ff` | `rrggbb` or `rrggbbaa`, without `#`. |
| `-a, --all` | off | Every namespace. |
| `-n, --namespace` | `""` | Daemon namespace. |
| `-o, --outputs` | all | Output list. |

Colors are not cached. If you want a solid color to survive a monitor
reconnect, set it with `xwww img 0xrrggbbaa` instead.

<a id="restore" name="restore"></a>

## `restore`

Reloads the last wallpaper each output displayed, using the per-output cache.

```sh
xwww restore
xwww restore -o HDMI-A-1
xwww restore --all
```

| Option | Default | Meaning |
|--------|---------|---------|
| `-a, --all` | off | Every namespace. |
| `-n, --namespace` | `""` | Daemon namespace. |
| `-o, --outputs` | all | Output list. |

<a id="clear-cache" name="clear-cache"></a>

## `clear-cache`

Deletes the entire `xwww` cache directory
(`$XDG_CACHE_HOME/xwww` or `~/.cache/xwww`), including per-output wallpaper
entries and preprocessed animation frames. It takes no options.

<a id="query" name="query"></a>

## `query`

Prints the outputs known to the daemon, their dimensions, scale factor and the
content currently displayed.

```sh
xwww query
xwww query --json
```

| Option | Default | Meaning |
|--------|---------|---------|
| `-a, --all` | off | Every namespace. |
| `-j, --json` | off | Machine-readable output. |
| `-n, --namespace` | `""` | Daemon namespace. |

Plain output uses one line per output:

```
NAMESPACE: OUTPUT: WIDTHxHEIGHT, scale: SCALE, currently displaying: image: /path/to/img
```

JSON output is an object keyed by namespace, each holding an array of objects
with `name`, `width`, `height`, `scale` and `displaying` (`image` or `color`).

<a id="palette" name="palette"></a>

## `palette`

Extracts the dominant colors of an image, or of the wallpaper currently
displayed. See [Palette](palette.md) for details and examples.

```sh
xwww palette image.png --count 5
xwww palette -o HDMI-A-1
xwww palette --json
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<image>` | current wallpaper | Image file to analyze. |
| `-c, --count` | `8` | Number of colors to print. |
| `--json` | off | Emit a JSON array instead of hex lines. |
| `-o, --output` | first output | Restrict the daemon query to one output. |
| `-n, --namespace` | `""` | Daemon namespace. |

<a id="slideshow" name="slideshow"></a>

## `slideshow`

Cycles through the supported images of a directory on a timer. See
[Slideshow](slideshow.md).

```sh
xwww slideshow ~/Pictures/wallpapers --interval 300
xwww slideshow ~/Pictures/wallpapers --random --transition-type any
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<dir>` | - | Directory to scan (non-recursive). |
| `-i, --interval` | `60` | Seconds between wallpapers. |
| `-r, --random` | off | Shuffle instead of alphabetical order. |
| `-t, --transition-type` | `any` | Transition between wallpapers. |
| `--resize` | `crop` | Resize strategy for each image. |
| `-n, --namespace` | `""` | Daemon namespace (the first one is used). |

The command runs until interrupted. The slideshow uses the default filter and
does not expose `-o/--outputs`; use `xwww img` directly for one-off,
per-output control.

<a id="random" name="random"></a>

## `random`

Sets a single random image from a directory and exits. Like `slideshow`, it
scans only the top level. See [Slideshow](slideshow.md).

```sh
xwww random ~/Pictures/wallpapers
xwww random ~/Pictures/wallpapers -t glitch
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<dir>` | - | Directory to choose from. |
| `-t, --transition-type` | `any` | Transition to use. |
| `--resize` | `crop` | Resize strategy. |
| `-n, --namespace` | `""` | Daemon namespace. |

<a id="screenshot" name="screenshot"></a>

## `screenshot`

Captures the wallpaper the daemon is currently displaying and saves it as a
PNG. See [Screenshot](screenshot.md).

```sh
xwww screenshot wallpaper.png
xwww screenshot -o HDMI-A-1 wallpaper.png
```

| Option | Default | Meaning |
|--------|---------|---------|
| `<output-file>` | - | Destination PNG path. |
| `-o, --monitor` | first output | Output to capture. |
| `-n, --namespace` | `""` | Daemon namespace. |

<a id="scene" name="scene"></a>

## `scene`

Renders a JavaScript scene and displays it through the daemon. Requires a build
with the `scene` feature; builds without it show the subcommand but fail with a
clear message. See the [Scene engine](scene.md) guide.

```sh
xwww scene check ~/scenes/clock.js
xwww scene render ~/scenes/clock.js -o out.png --size 2560x1440
xwww scene run ~/scenes/clock.js --fps 10 --palette xwww
```

### `scene check`

Compiles the scene without rendering. Takes the script path.

### `scene render`

Renders one frame to a PNG file.

| Option | Default | Meaning |
|--------|---------|---------|
| `-o, --output` | `scene.png` | Output PNG. |
| `--size` | `2560x1440` | Canvas size in `WxH` physical pixels. |
| `--palette` | xwww file, then equisdots, then fallback | Palette source. |
| `--timeout-ms` | `100` | Per-frame JavaScript budget. |
| `--asset` | scene directory | Extra asset path; repeatable. |

### `scene run`

Renders continuously and sends frames to the daemon.

| Option | Default | Meaning |
|--------|---------|---------|
| `--fps` | `10` | Frames rendered and sent per second. |
| `--palette-fade` | `800` | Crossfade duration in ms when the palette changes (`0` disables). |
| `--palette` | as `render` | Palette source. |
| `--timeout-ms` | `100` | Per-frame JavaScript budget. |
| `--asset` | scene directory | Extra asset path; repeatable. |
| `-o, --outputs` | all | Output list. |
| `-n, --namespace` | `""` | Daemon namespace. |
| `--transition-type` | `none` | Entry transition for the first frame. |
| `--transition-step` | `255` | Step for the entry transition. |
| `--transition-duration` | `1.0` | Seconds the entry transition takes. |
| `--transition-fps` | `144` | Entry transition frame rate. |
| `--transition-angle` | `45` | Angle for `wipe`/`wave`. |
| `--transition-pos` | `center` | Center for circular effects. |
| `--transition-bezier` | `.54,0,.34,.99` | Bezier curve. |
| `--transition-wave` | `20,20` | Wave size. |
| `--invert-y` | off | Flips the entry-transition y coordinate. |

<a id="daemon-control" name="daemon-control"></a>

## Daemon control

| Command | Options | Effect |
|---------|---------|--------|
| `xwww toggle` | `-a`, `-n` | Flip the paused state. |
| `xwww pause` | `-a`, `-n` | Freeze animations on the last rendered frame. |
| `xwww unpause` | `-a`, `-n` | Resume animations. |
| `xwww kill` | `-a`, `-n` | Stop the daemon and wait for its socket to disappear. |

Pausing only stops animation rendering; the daemon keeps answering client
commands. `kill` is preferred over sending signals because it confirms that the
socket file was removed.

<div align="center">
  <a href="#top">Back to top</a>
</div>
