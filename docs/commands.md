# Commands

`xwww <command> [options]`. Most commands accept `-n/--namespace` to target a specific daemon
namespace and `-o/--outputs` to target specific monitors.

## `img`

Sets an image, animated image (gif/webp/apng) or video as the wallpaper.

```sh
xwww img path/to/image.png
xwww img -o HDMI-A-1,DP-2 wallpaper.jpg
xwww img 0xff0000ff                 # solid color
```

Notable options:

| Option | Description |
|--------|-------------|
| `-o, --outputs` | Comma-separated list of outputs (default: all). |
| `--resize` | `crop` (default), `fit`, `stretch` or `no`. |
| `--filter` | `Nearest`, `Bilinear`, `CatmullRom`, `Mitchell`, `Lanczos3`. |
| `-t, --transition-type` | See [transitions](transitions.md). |
| `--transition-step/-fps/-duration` | Transition timing controls. |
| `--blur <radius>` | Gaussian blur radius (see [image effects](image-effects.md)). |
| `--dim <factor>` | Dim factor in `[0,1]` (see [image effects](image-effects.md)). |
| `--no-cache` | Don't update the per-output cache. |

## `clear`

Fills outputs with a color: `xwww clear [color]` (color is `rrggbb[aa]`, default black).

## `restore`

Reloads the last image each output displayed (from the cache).

## `query`

Prints output information (name, dimensions, current image). Use `--json` for machine-readable
output, which is useful for scripting and for tools like the `palette` command.

## `palette`

Extracts the dominant colors of an image, or of the current wallpaper. See [palette](palette.md).

## `slideshow`

Cycles through the images of a directory. See [slideshow](slideshow.md).

## `random`

Sets a single random image from a directory (one-shot, unlike `slideshow`):

```sh
xwww random ~/Pictures/wallpapers
xwww random ~/Pictures/wallpapers -t glitch
```

## `screenshot`

Captures the current wallpaper to a PNG. See [screenshot](screenshot.md).

## `toggle`, `pause`, `unpause`, `kill`

Control the daemon. `toggle` flips the paused state; `pause`/`unpause` stop/resume animations;
`kill` stops the daemon. All accept `-a/--all` to target every namespace.

## `clear-cache`

Deletes the entire cache directory (cached last-wallpaper info and animation frames).
