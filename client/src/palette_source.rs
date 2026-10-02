#![cfg_attr(not(feature = "scene"), allow(dead_code))]

//! Palette sources for the client-side effects.
//!
//! A palette is normalized into [`ScenePalette`] regardless of where it comes from. The
//! `--map-palette` flag of `xwww img` consumes it through [`ScenePalette::gradient_stops`], and
//! the future scene engine will use the same type for its `ctx.palette`.

use std::path::PathBuf;
use std::process::Command;

/// A single 8-bit sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Rec. 709 relative luminance, in `[0.0, 255.0]`.
    #[must_use]
    pub fn luma(self) -> f32 {
        f32::from(self.r) * 0.2126 + f32::from(self.g) * 0.7152 + f32::from(self.b) * 0.0722
    }

    #[must_use]
    pub const fn to_array(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

/// A palette normalized from any supported source.
#[derive(Debug, Clone)]
pub struct ScenePalette {
    pub slug: String,
    pub name: String,
    pub background: Rgb,
    pub foreground: Rgb,
    /// Base16 `color0..color15`, or the colors found in a plain hex list.
    pub colors: Vec<Rgb>,
    /// Semantic overrides (`workspaceActive`, ...), in file order.
    pub roles: Vec<(String, Rgb)>,
}

/// Where a palette comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteSpec {
    /// `xwww` or `xwww:<path>`: the xwww palette file
    /// (`$XDG_CONFIG_HOME/xwww/palette.json`, or `~/.config/xwww/palette.json`).
    Xwww(Option<PathBuf>),
    /// `equisdots` or `equisdots:<slug>`: the equisdots desktop palette (explicit opt-in).
    Equisdots(Option<String>),
    /// `file:<path>` or a bare path: palette JSON or one `#rrggbb` per line.
    File(PathBuf),
    /// `command:<cmd>`: stdout parsed like `File`.
    Command(String),
}

const DEFAULT_SLUG: &str = "x";

impl std::str::FromStr for PaletteSpec {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "unrecognized palette source '{raw}'. Valid sources: \
                 xwww | xwww:<path> | equisdots | equisdots:<slug> | file:<path> | command:<cmd>"
            )
        };

        match raw.split_once(':') {
            Some(("xwww", value)) if !value.is_empty() => Ok(Self::Xwww(Some(expand_tilde(value)))),
            Some(("equisdots", value)) if !value.is_empty() => {
                Ok(Self::Equisdots(Some(value.to_lowercase())))
            }
            Some(("file", value)) if !value.is_empty() => Ok(Self::File(expand_tilde(value))),
            Some(("command", value)) if !value.is_empty() => Ok(Self::Command(value.to_string())),
            Some((kind, _)) if matches!(kind, "xwww" | "equisdots" | "file" | "command") => {
                Err(format!("'{kind}' requires a value after ':'"))
            }
            Some(_) => Err(invalid()),
            None if raw == "xwww" => Ok(Self::Xwww(None)),
            None if raw == "equisdots" => Ok(Self::Equisdots(None)),
            None if raw.starts_with('/') || raw.starts_with('~') || raw.starts_with('.') => {
                Ok(Self::File(expand_tilde(raw)))
            }
            None => Err(invalid()),
        }
    }
}

impl ScenePalette {
    /// Loads a palette from a `--map-palette` spec string.
    pub fn load(spec: &str) -> Result<Self, String> {
        match spec.parse::<PaletteSpec>()? {
            PaletteSpec::Xwww(path) => Self::load_xwww(path.as_deref()),
            PaletteSpec::Equisdots(slug) => Self::load_equisdots(slug.as_deref()),
            PaletteSpec::File(path) => {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
                Self::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
            }
            PaletteSpec::Command(command) => {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .map_err(|e| format!("failed to run '{command}': {e}"))?;
                if !output.status.success() {
                    return Err(format!(
                        "'{command}' exited with {}",
                        output.status.code().map_or_else(
                            || "a signal".to_string(),
                            |code| format!("status {code}")
                        )
                    ));
                }
                let text = String::from_utf8_lossy(&output.stdout);
                Self::from_text(&text).map_err(|e| format!("'{command}': {e}"))
            }
        }
    }

    /// Loads the xwww palette file (`$XDG_CONFIG_HOME/xwww/palette.json` by default).
    ///
    /// This is the canonical contract: the desktop (Hyprland/dotfiles) writes the active palette
    /// there and xwww only reads it. The file accepts the base16 JSON schema or a plain color
    /// list.
    pub fn load_xwww(path: Option<&std::path::Path>) -> Result<Self, String> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => xwww_palette_path()?,
        };
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        Self::from_text(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Default palette resolution when no source is given: the xwww palette file, then the
    /// equisdots desktop palette, then the neutral fallback. Never fails.
    #[must_use]
    pub fn load_default() -> Self {
        Self::load_xwww(None)
            .or_else(|_| Self::load_equisdots(None))
            .unwrap_or_else(|_| Self::fallback())
    }

    /// Loads the equisdots desktop palette.
    ///
    /// The active slug comes from `~/.config/hypr/settings.json` (`bar.palette`, legacy
    /// `dock.palette`, fallback `x`); the file lives in `dock/palettes/<slug>.json`. A missing
    /// slug falls back to `x`, mirroring `theme-sync`.
    pub fn load_equisdots(slug_override: Option<&str>) -> Result<Self, String> {
        let dir = equisdots_palettes_dir()?;
        let slug = slug_override.map_or_else(equisdots_active_slug, str::to_lowercase);

        let path = find_palette_file(&dir, &slug)
            .or_else(|| {
                (slug != DEFAULT_SLUG)
                    .then(|| find_palette_file(&dir, DEFAULT_SLUG))
                    .flatten()
            })
            .ok_or_else(|| format!("palette '{slug}' not found in {}", dir.display()))?;

        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        parse_palette_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Parses a palette JSON (equisdots/base16 schema) or a plain hex list.
    pub fn from_text(text: &str) -> Result<Self, String> {
        if text.trim_start().starts_with('{') {
            parse_palette_str(text)
        } else {
            let colors = parse_hex_lines(text)?;
            if colors.is_empty() {
                return Err(
                    "no colors found (expected palette JSON or one #rrggbb per line)".into(),
                );
            }
            Ok(Self::from_colors(colors))
        }
    }

    /// Builds a palette from an arbitrary non-empty list of colors.
    #[must_use]
    pub fn from_colors(colors: Vec<Rgb>) -> Self {
        let background = colors[0];
        let foreground = *colors.last().unwrap();
        Self {
            slug: "custom".into(),
            name: "Custom".into(),
            background,
            foreground,
            colors,
            roles: Vec::new(),
        }
    }

    /// Neutral fallback palette, used when no palette source is configured or one fails to load.
    #[must_use]
    pub fn fallback() -> Self {
        Self::from_colors(vec![
            Rgb::new(10, 10, 14),
            Rgb::new(36, 40, 59),
            Rgb::new(230, 230, 240),
        ])
    }

    /// Gradient stops for `effects::palette_map`: every color plus background and foreground,
    /// sorted by luminance and with duplicate luminances removed.
    #[must_use]
    pub fn gradient_stops(&self) -> Vec<[u8; 3]> {
        let mut stops = Vec::with_capacity(self.colors.len() + 2);
        stops.extend(self.colors.iter().copied());
        stops.push(self.background);
        stops.push(self.foreground);
        stops.sort_by(|a, b| a.luma().total_cmp(&b.luma()));
        stops.dedup_by(|a, b| a.luma().total_cmp(&b.luma()) == std::cmp::Ordering::Equal);
        stops.into_iter().map(Rgb::to_array).collect()
    }
}

/// Parses the equisdots/base16 palette schema.
pub fn parse_palette_str(text: &str) -> Result<ScenePalette, String> {
    let value = jzon::parse(text).map_err(|e| format!("invalid palette JSON: {e}"))?;

    let string_field = |key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
    };

    let base16 = value
        .get("base16")
        .ok_or("palette is missing the \"base16\" object")?;

    let mut colors = Vec::with_capacity(16);
    for i in 0..16 {
        let key = format!("color{i}");
        let raw = base16
            .get(&key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("palette is missing base16.{key}"))?;
        colors.push(parse_hex(raw)?);
    }

    let background = match string_field("background") {
        Some(raw) => parse_hex(&raw)?,
        None => colors[0],
    };
    let foreground = match string_field("foreground") {
        Some(raw) => parse_hex(&raw)?,
        None => colors[7],
    };

    let mut roles = Vec::new();
    if let Some(obj) = value.get("roles").and_then(|v| v.as_object()) {
        for (name, raw) in obj.iter() {
            if let Some(raw) = raw.as_str() {
                roles.push((name.to_string(), parse_hex(raw)?));
            }
        }
    }

    Ok(ScenePalette {
        slug: string_field("slug").unwrap_or_else(|| "custom".into()),
        name: string_field("name").unwrap_or_else(|| "Custom".into()),
        background,
        foreground,
        colors,
        roles,
    })
}

/// Parses a text file with one color per line. Accepts `#rrggbb`, `rrggbb`, `key=#rrggbb` and
/// `key='#rrggbb'`; blank lines and comments (`#`, `//`, `;`) are skipped.
pub fn parse_hex_lines(text: &str) -> Result<Vec<Rgb>, String> {
    let mut colors = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with(';') {
            continue;
        }

        let value = line
            .rsplit_once('=')
            .map_or(line, |(_, value)| value.trim())
            .trim_matches(['"', '\'']);

        if is_hex_color(value) {
            colors.push(parse_hex(value)?);
        } else if !line.starts_with('#') {
            return Err(format!("unrecognized line in palette: '{line}'"));
        }
    }

    Ok(colors)
}

fn is_hex_color(raw: &str) -> bool {
    let body = raw.strip_prefix('#').unwrap_or(raw);
    body.len() == 6 && body.bytes().all(|b| b.is_ascii_hexdigit())
}

fn parse_hex(raw: &str) -> Result<Rgb, String> {
    let body = raw.trim().strip_prefix('#').unwrap_or(raw.trim());
    if !is_hex_color(body) {
        return Err(format!("expected #rrggbb, found '{raw}'"));
    }
    let channel = |i: usize| {
        u8::from_str_radix(&body[i..i + 2], 16).map_err(|_| format!("invalid hex color '{raw}'"))
    };
    Ok(Rgb::new(channel(0)?, channel(2)?, channel(4)?))
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn xwww_palette_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or("neither XDG_CONFIG_HOME nor HOME are set")?;
    Ok(base.join("xwww").join("palette.json"))
}

fn equisdots_palettes_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config/hypr/scripts/quickshell/dock/palettes"))
}

fn equisdots_settings_path() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config/hypr/settings.json"))
}

/// Active palette slug: `bar.palette`, then legacy `dock.palette`, then `x`.
fn equisdots_active_slug() -> String {
    let Ok(path) = equisdots_settings_path() else {
        return DEFAULT_SLUG.into();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return DEFAULT_SLUG.into();
    };
    let Ok(value) = jzon::parse(&text) else {
        return DEFAULT_SLUG.into();
    };
    value
        .get("bar")
        .and_then(|bar| bar.get("palette"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            value
                .get("dock")
                .and_then(|dock| dock.get("palette"))
                .and_then(|v| v.as_str())
        })
        .map(str::trim)
        .filter(|slug| !slug.is_empty())
        .map(str::to_lowercase)
        .unwrap_or_else(|| DEFAULT_SLUG.into())
}

/// Finds `<slug>.json` in `dir`, searching one level of subdirectories (e.g. `community/`).
fn find_palette_file(dir: &std::path::Path, slug: &str) -> Option<PathBuf> {
    let filename = format!("{slug}.json");

    let direct = dir.join(&filename);
    if direct.is_file() {
        return Some(direct);
    }

    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join(&filename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const X_JSON: &str = r##"{
        "author": "xscriptor",
        "background": "#050505",
        "base16": {
            "color0": "#0a0a0a", "color1": "#fc618d", "color2": "#7bd88f",
            "color3": "#fce566", "color4": "#fd9353", "color5": "#948ae3",
            "color6": "#5ad4e6", "color7": "#f7f1ff", "color8": "#7e7b82",
            "color9": "#fc618d", "color10": "#7bd88f", "color11": "#fce566",
            "color12": "#fd9353", "color13": "#948ae3", "color14": "#5ad4e6",
            "color15": "#f7f1ff"
        },
        "foreground": "#f7f1ff",
        "name": "X",
        "roles": { "workspaceActive": "#eab308" },
        "slug": "x"
    }"##;

    #[test]
    fn parses_equisdots_schema() {
        let palette = parse_palette_str(X_JSON).unwrap();
        assert_eq!(palette.slug, "x");
        assert_eq!(palette.name, "X");
        assert_eq!(palette.background.hex(), "#050505");
        assert_eq!(palette.foreground.hex(), "#F7F1FF");
        assert_eq!(palette.colors.len(), 16);
        assert_eq!(palette.colors[1].hex(), "#FC618D");
        let (_, active) = palette
            .roles
            .iter()
            .find(|(name, _)| name == "workspaceActive")
            .unwrap();
        assert_eq!(active.hex(), "#EAB308");
    }

    #[test]
    fn falls_back_to_color0_and_color7() {
        let json = r##"{
            "base16": {
                "color0": "#000000", "color1": "#111111", "color2": "#222222",
                "color3": "#333333", "color4": "#444444", "color5": "#555555",
                "color6": "#666666", "color7": "#777777", "color8": "#888888",
                "color9": "#999999", "color10": "#aaaaaa", "color11": "#bbbbbb",
                "color12": "#cccccc", "color13": "#dddddd", "color14": "#eeeeee",
                "color15": "#ffffff"
            }
        }"##;
        let palette = parse_palette_str(json).unwrap();
        assert_eq!(palette.background.hex(), "#000000");
        assert_eq!(palette.foreground.hex(), "#777777");
        assert_eq!(palette.slug, "custom");
    }

    #[test]
    fn rejects_palettes_without_base16() {
        assert!(parse_palette_str(r#"{"name":"X"}"#).is_err());
        assert!(parse_palette_str(r##"{"base16":{"color0":"#000000"}}"##).is_err());
    }

    #[test]
    fn rejects_invalid_hex() {
        let json = X_JSON.replace("#fc618d", "not-a-color");
        assert!(parse_palette_str(&json).is_err());
    }

    #[test]
    fn parses_hex_lines_with_comments_and_assignments() {
        let text =
            "# comment\n// another\n\n#fc618d\n7bd88f\naccent='#eab308'\nbackground=#101010\n";
        let colors = parse_hex_lines(text).unwrap();
        let hexes: Vec<_> = colors.iter().map(|c| c.hex()).collect();
        assert_eq!(hexes, ["#FC618D", "#7BD88F", "#EAB308", "#101010"]);
    }

    #[test]
    fn rejects_malformed_hex_lines() {
        assert!(parse_hex_lines("hello world\n").is_err());
    }

    #[test]
    fn from_text_detects_json_and_hex_lists() {
        assert_eq!(ScenePalette::from_text(X_JSON).unwrap().slug, "x");
        let palette = ScenePalette::from_text("#000000\n#ffffff\n").unwrap();
        assert_eq!(palette.background.hex(), "#000000");
        assert_eq!(palette.foreground.hex(), "#FFFFFF");
        assert!(ScenePalette::from_text("\n\n").is_err());
    }

    #[test]
    fn gradient_stops_are_sorted_by_luminance() {
        let palette = parse_palette_str(X_JSON).unwrap();
        let stops = palette.gradient_stops();
        let lumas: Vec<f32> = stops
            .iter()
            .map(|[r, g, b]| Rgb::new(*r, *g, *b).luma())
            .collect();
        assert!(lumas.windows(2).all(|w| w[0] <= w[1]), "{lumas:?}");
        assert_eq!(stops.first().unwrap(), &palette.background.to_array());
    }

    #[test]
    fn parses_palette_specs() {
        assert_eq!(
            "xwww".parse::<PaletteSpec>().unwrap(),
            PaletteSpec::Xwww(None)
        );
        assert_eq!(
            "xwww:/tmp/p.json".parse::<PaletteSpec>().unwrap(),
            PaletteSpec::Xwww(Some(PathBuf::from("/tmp/p.json")))
        );
        assert_eq!(
            "equisdots".parse::<PaletteSpec>().unwrap(),
            PaletteSpec::Equisdots(None)
        );
        assert_eq!(
            "equisdots:London".parse::<PaletteSpec>().unwrap(),
            PaletteSpec::Equisdots(Some("london".into()))
        );
        assert_eq!(
            "/tmp/p.json".parse::<PaletteSpec>().unwrap(),
            PaletteSpec::File(PathBuf::from("/tmp/p.json"))
        );
        assert!("xwww:".parse::<PaletteSpec>().is_err());
        assert!("equisdots:".parse::<PaletteSpec>().is_err());
        assert!("bogus:thing".parse::<PaletteSpec>().is_err());
    }

    #[test]
    fn loads_an_explicit_xwww_palette_file() {
        let path = std::env::temp_dir().join("xwww-palette-source-test.json");
        std::fs::write(&path, X_JSON).unwrap();
        let palette = ScenePalette::load(&format!("xwww:{}", path.display())).unwrap();
        assert_eq!(palette.slug, "x");
        assert_eq!(palette.background.hex(), "#050505");
    }
}
