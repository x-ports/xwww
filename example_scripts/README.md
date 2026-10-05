<a id="top" name="top"></a>

# Example scripts

This directory contains example shell scripts that show how to combine `xwww`
with other tools. They are starting points: copy them, adapt the paths, and run
them instead of (or alongside) `xwww-daemon`.

<div align="center">
  <a href="../README.md">Project README</a> &middot;
  <a href="../docs/README.md">Documentation</a> &middot;
  <a href="#scripts">Scripts</a> &middot;
  <a href="#requirements">Requirements</a>
</div>

<a id="scripts" name="scripts"></a>

## Scripts

| Script | Purpose |
|--------|---------|
| `xwww_init_according_to_time_of_day.sh` | Starts the daemon and picks an initial image based on the current hour. Edit the match cases to choose your images. |
| `xwww_randomize.sh` | Changes the wallpaper to a random image from a directory at a fixed interval. |
| `xwww_randomize_multi.sh` | Like `xwww_randomize.sh`, but picks an independent random image for each connected output. |
| `xwww_scheduler.sh` | Uses the `at` command to schedule wallpaper changes at specific times of day. |

Usage examples:

```sh
./xwww_randomize.sh ~/Pictures/wallpapers 300
./xwww_randomize_multi.sh ~/Pictures/wallpapers 600
./xwww_scheduler.sh '~/Pictures/day.png --transition-type fade' 08:00
```

<a id="requirements" name="requirements"></a>

## Requirements

- A `sh`-compatible shell and the standard utilities used by each script
  (`find`, `sort`, `awk`, `tr`, ...).
- `xwww_scheduler.sh` additionally requires the `at` command.
- The scripts assume `xwww` and `xwww-daemon` are on your `PATH`.

See [Commands](../docs/commands.md) and the `xwww <command> --help` output for
every available option.

<div align="center">
  <a href="#top">Back to top</a>
</div>
