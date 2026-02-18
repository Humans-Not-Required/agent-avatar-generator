use image::{Rgba, RgbaImage};
use serde::Serialize;

/// Available color themes for avatar generation.
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Warm,
    Cool,
    Ocean,
    Forest,
    Sunset,
    Neon,
    Pastel,
    Monochrome,
    Earth,
}

struct ThemeConfig {
    hue_center: f64,
    hue_range: f64,
    sat_range: (f64, f64),
    light_range: (f64, f64),
    bg_sat_range: (f64, f64),
    bg_light_range: (f64, f64),
}

#[derive(Debug, Serialize)]
pub struct ThemeInfo {
    pub name: String,
    pub description: String,
}

impl Theme {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "warm" => Some(Self::Warm),
            "cool" => Some(Self::Cool),
            "ocean" => Some(Self::Ocean),
            "forest" => Some(Self::Forest),
            "sunset" => Some(Self::Sunset),
            "neon" => Some(Self::Neon),
            "pastel" => Some(Self::Pastel),
            "monochrome" | "mono" => Some(Self::Monochrome),
            "earth" => Some(Self::Earth),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            Self::Warm => "warm",
            Self::Cool => "cool",
            Self::Ocean => "ocean",
            Self::Forest => "forest",
            Self::Sunset => "sunset",
            Self::Neon => "neon",
            Self::Pastel => "pastel",
            Self::Monochrome => "monochrome",
            Self::Earth => "earth",
        }
    }

    fn config(&self) -> ThemeConfig {
        match self {
            Self::Warm => ThemeConfig {
                hue_center: 25.0,
                hue_range: 50.0,
                sat_range: (0.5, 0.85),
                light_range: (0.35, 0.65),
                bg_sat_range: (0.2, 0.4),
                bg_light_range: (0.85, 0.95),
            },
            Self::Cool => ThemeConfig {
                hue_center: 220.0,
                hue_range: 60.0,
                sat_range: (0.4, 0.75),
                light_range: (0.35, 0.6),
                bg_sat_range: (0.15, 0.35),
                bg_light_range: (0.88, 0.96),
            },
            Self::Ocean => ThemeConfig {
                hue_center: 200.0,
                hue_range: 40.0,
                sat_range: (0.5, 0.85),
                light_range: (0.3, 0.6),
                bg_sat_range: (0.2, 0.4),
                bg_light_range: (0.85, 0.95),
            },
            Self::Forest => ThemeConfig {
                hue_center: 130.0,
                hue_range: 50.0,
                sat_range: (0.35, 0.7),
                light_range: (0.25, 0.55),
                bg_sat_range: (0.1, 0.3),
                bg_light_range: (0.88, 0.95),
            },
            Self::Sunset => ThemeConfig {
                hue_center: 15.0,
                hue_range: 45.0,
                sat_range: (0.6, 0.95),
                light_range: (0.4, 0.65),
                bg_sat_range: (0.2, 0.5),
                bg_light_range: (0.82, 0.93),
            },
            Self::Neon => ThemeConfig {
                hue_center: 180.0,
                hue_range: 180.0,
                sat_range: (0.85, 1.0),
                light_range: (0.45, 0.6),
                bg_sat_range: (0.1, 0.2),
                bg_light_range: (0.08, 0.15),
            },
            Self::Pastel => ThemeConfig {
                hue_center: 180.0,
                hue_range: 180.0,
                sat_range: (0.25, 0.5),
                light_range: (0.7, 0.85),
                bg_sat_range: (0.05, 0.15),
                bg_light_range: (0.94, 0.98),
            },
            Self::Monochrome => ThemeConfig {
                hue_center: 0.0,
                hue_range: 0.0,
                sat_range: (0.0, 0.05),
                light_range: (0.15, 0.85),
                bg_sat_range: (0.0, 0.02),
                bg_light_range: (0.9, 0.96),
            },
            Self::Earth => ThemeConfig {
                hue_center: 30.0,
                hue_range: 25.0,
                sat_range: (0.25, 0.55),
                light_range: (0.3, 0.6),
                bg_sat_range: (0.1, 0.25),
                bg_light_range: (0.88, 0.95),
            },
        }
    }

    /// Remap a color to match this theme.
    /// Detects background vs foreground by lightness/saturation heuristic.
    pub fn remap_color(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let config = self.config();
        let (h, s, l) = rgb_to_hsl(r, g, b);

        // Detect if pixel is "background-like" (light and desaturated)
        let is_bg = l > 0.72 && s < 0.35;

        let new_h = remap_hue(h, config.hue_center, config.hue_range);

        if is_bg {
            let mid_s = (config.bg_sat_range.0 + config.bg_sat_range.1) / 2.0;
            let new_s = (mid_s + s * 0.2).clamp(config.bg_sat_range.0, config.bg_sat_range.1);
            let new_l = l.clamp(config.bg_light_range.0, config.bg_light_range.1);
            hsl_to_rgb(new_h, new_s, new_l)
        } else {
            let new_s = s.clamp(config.sat_range.0, config.sat_range.1);
            // Preserve relative lightness within the theme's range
            let l_range = config.light_range.1 - config.light_range.0;
            let new_l = config.light_range.0 + (l.clamp(0.1, 0.9) - 0.1) / 0.8 * l_range;
            hsl_to_rgb(new_h, new_s, new_l.clamp(config.light_range.0, config.light_range.1))
        }
    }

    /// Apply theme to a PNG image (mutates in place).
    pub fn apply_to_image(&self, img: &mut RgbaImage) {
        for pixel in img.pixels_mut() {
            let [r, g, b, a] = pixel.0;
            if a == 0 {
                continue;
            }
            let (nr, ng, nb) = self.remap_color(r, g, b);
            *pixel = Rgba([nr, ng, nb, a]);
        }
    }

    /// Apply theme to an SVG string (replaces all hex colors).
    pub fn apply_to_svg(&self, svg: &str) -> String {
        let bytes = svg.as_bytes();
        let mut result = String::with_capacity(svg.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'#' && i + 6 < bytes.len() {
                // Check if next 6 chars are hex digits
                let slice = &svg[i + 1..i + 7];
                if slice.len() == 6 && slice.chars().all(|c| c.is_ascii_hexdigit()) {
                    let r = u8::from_str_radix(&slice[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&slice[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&slice[4..6], 16).unwrap_or(0);
                    let (nr, ng, nb) = self.remap_color(r, g, b);
                    result.push_str(&format!("#{:02x}{:02x}{:02x}", nr, ng, nb));
                    i += 7;
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }
}

/// Remap a hue value from full 0-360 range into a theme's constrained range.
fn remap_hue(original_h: f64, center: f64, range: f64) -> f64 {
    if range >= 180.0 {
        return original_h; // Full spectrum — no remapping
    }
    if range < 1.0 {
        return original_h; // Monochrome — hue irrelevant (sat≈0)
    }
    // Map original hue proportionally into [center-range, center+range]
    let ratio = original_h / 360.0;
    let min_h = center - range;
    let remapped = min_h + ratio * range * 2.0;
    ((remapped % 360.0) + 360.0) % 360.0
}

/// Convert RGB to HSL. Returns (h: 0-360, s: 0-1, l: 0-1).
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l); // Achromatic
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - rf).abs() < f64::EPSILON {
        let mut h = (gf - bf) / d;
        if gf < bf {
            h += 6.0;
        }
        h / 6.0
    } else if (max - gf).abs() < f64::EPSILON {
        ((bf - rf) / d + 2.0) / 6.0
    } else {
        ((rf - gf) / d + 4.0) / 6.0
    };

    (h * 360.0, s, l)
}

/// Convert HSL to RGB. h: 0-360, s: 0-1, l: 0-1.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[allow(dead_code)]
pub fn is_valid_theme(name: &str) -> bool {
    Theme::parse(name).is_some()
}

pub fn available_themes() -> Vec<ThemeInfo> {
    vec![
        ThemeInfo {
            name: "warm".to_string(),
            description: "Reds, oranges, and yellows. Cozy and inviting.".to_string(),
        },
        ThemeInfo {
            name: "cool".to_string(),
            description: "Blues and cyans. Calm and professional.".to_string(),
        },
        ThemeInfo {
            name: "ocean".to_string(),
            description: "Teals and deep blues. Aquatic vibes.".to_string(),
        },
        ThemeInfo {
            name: "forest".to_string(),
            description: "Greens and earth tones. Natural and organic.".to_string(),
        },
        ThemeInfo {
            name: "sunset".to_string(),
            description: "Pinks, reds, and oranges. Dramatic and vivid.".to_string(),
        },
        ThemeInfo {
            name: "neon".to_string(),
            description: "High-saturation on dark background. Electric and bold.".to_string(),
        },
        ThemeInfo {
            name: "pastel".to_string(),
            description: "Soft, light colors. Gentle and approachable.".to_string(),
        },
        ThemeInfo {
            name: "monochrome".to_string(),
            description: "Grayscale only. Minimalist and classic.".to_string(),
        },
        ThemeInfo {
            name: "earth".to_string(),
            description: "Browns, tans, and muted tones. Grounded and warm.".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_str_valid() {
        assert_eq!(Theme::parse("warm"), Some(Theme::Warm));
        assert_eq!(Theme::parse("Cool"), Some(Theme::Cool));
        assert_eq!(Theme::parse("OCEAN"), Some(Theme::Ocean));
        assert_eq!(Theme::parse("mono"), Some(Theme::Monochrome));
        assert_eq!(Theme::parse("monochrome"), Some(Theme::Monochrome));
    }

    #[test]
    fn test_theme_from_str_invalid() {
        assert_eq!(Theme::parse("invalid"), None);
        assert_eq!(Theme::parse(""), None);
        assert_eq!(Theme::parse("rainbow"), None);
    }

    #[test]
    fn test_theme_name_roundtrip() {
        for theme in &[
            Theme::Warm, Theme::Cool, Theme::Ocean, Theme::Forest,
            Theme::Sunset, Theme::Neon, Theme::Pastel, Theme::Monochrome, Theme::Earth,
        ] {
            assert_eq!(Theme::parse(theme.name()), Some(theme.clone()));
        }
    }

    #[test]
    fn test_rgb_to_hsl_pure_red() {
        let (h, s, l) = rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 1.0, "Red hue should be ~0°, got {h}");
        assert!((s - 1.0).abs() < 0.01, "Pure red saturation should be ~1.0");
        assert!((l - 0.5).abs() < 0.01, "Pure red lightness should be ~0.5");
    }

    #[test]
    fn test_rgb_to_hsl_pure_green() {
        let (h, s, l) = rgb_to_hsl(0, 255, 0);
        assert!((h - 120.0).abs() < 1.0, "Green hue should be ~120°, got {h}");
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_pure_blue() {
        let (h, s, l) = rgb_to_hsl(0, 0, 255);
        assert!((h - 240.0).abs() < 1.0, "Blue hue should be ~240°, got {h}");
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rgb_to_hsl_white() {
        let (h, s, l) = rgb_to_hsl(255, 255, 255);
        assert!((s - 0.0).abs() < 0.01, "White should be achromatic");
        assert!((l - 1.0).abs() < 0.01, "White lightness should be 1.0");
    }

    #[test]
    fn test_rgb_to_hsl_black() {
        let (h, s, l) = rgb_to_hsl(0, 0, 0);
        assert!((s - 0.0).abs() < 0.01, "Black should be achromatic");
        assert!((l - 0.0).abs() < 0.01, "Black lightness should be 0.0");
    }

    #[test]
    fn test_rgb_to_hsl_gray() {
        let (h, s, l) = rgb_to_hsl(128, 128, 128);
        assert!((s - 0.0).abs() < 0.01, "Gray should be achromatic");
        assert!((l - 0.502).abs() < 0.01, "Gray lightness should be ~0.5");
    }

    #[test]
    fn test_hsl_roundtrip() {
        for &(r, g, b) in &[(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 64, 192), (200, 100, 50)] {
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(h, s, l);
            assert!(
                (r as i16 - r2 as i16).unsigned_abs() <= 1
                    && (g as i16 - g2 as i16).unsigned_abs() <= 1
                    && (b as i16 - b2 as i16).unsigned_abs() <= 1,
                "HSL roundtrip failed for ({r},{g},{b}) → ({h:.1},{s:.3},{l:.3}) → ({r2},{g2},{b2})"
            );
        }
    }

    #[test]
    fn test_warm_theme_hue_range() {
        let theme = Theme::Warm;
        // Blue (240°) should be remapped into warm range
        let (r, g, b) = theme.remap_color(0, 0, 200);
        let (h, _s, _l) = rgb_to_hsl(r, g, b);
        // Warm range: center 25, range 50 → hues roughly -25..75 = 335..75
        let in_warm = h <= 80.0 || h >= 330.0;
        assert!(in_warm, "Blue remapped to warm should have hue in warm range, got {h}°");
    }

    #[test]
    fn test_cool_theme_hue_range() {
        let theme = Theme::Cool;
        // Red (0°) should be remapped into cool range
        let (r, g, b) = theme.remap_color(200, 50, 50);
        let (h, _s, _l) = rgb_to_hsl(r, g, b);
        // Cool range: center 220, range 60 → hues roughly 160..280
        assert!(h >= 155.0 && h <= 285.0, "Red remapped to cool should be in cool range, got {h}°");
    }

    #[test]
    fn test_monochrome_desaturation() {
        let theme = Theme::Monochrome;
        // Saturated red should become near-gray
        let (r, g, b) = theme.remap_color(220, 50, 50);
        let (_h, s, _l) = rgb_to_hsl(r, g, b);
        assert!(s < 0.1, "Monochrome should desaturate, got s={s:.3}");
    }

    #[test]
    fn test_neon_high_saturation() {
        let theme = Theme::Neon;
        // Muted color should become saturated
        let (r, g, b) = theme.remap_color(120, 100, 110);
        let (_h, s, _l) = rgb_to_hsl(r, g, b);
        assert!(s > 0.8, "Neon should have high saturation, got s={s:.3}");
    }

    #[test]
    fn test_neon_dark_background() {
        let theme = Theme::Neon;
        // Light background color should become dark for neon
        let (r, g, b) = theme.remap_color(220, 220, 230);
        let (_h, _s, l) = rgb_to_hsl(r, g, b);
        assert!(l < 0.3, "Neon background should be dark, got l={l:.3}");
    }

    #[test]
    fn test_pastel_light_colors() {
        let theme = Theme::Pastel;
        let (r, g, b) = theme.remap_color(150, 50, 50);
        let (_h, s, l) = rgb_to_hsl(r, g, b);
        assert!(l > 0.65, "Pastel should be light, got l={l:.3}");
        assert!(s < 0.55, "Pastel should be low-sat, got s={s:.3}");
    }

    #[test]
    fn test_theme_deterministic() {
        let theme = Theme::Ocean;
        let (r1, g1, b1) = theme.remap_color(100, 150, 200);
        let (r2, g2, b2) = theme.remap_color(100, 150, 200);
        assert_eq!((r1, g1, b1), (r2, g2, b2), "Theme should be deterministic");
    }

    #[test]
    fn test_apply_to_svg_remaps_colors() {
        let theme = Theme::Monochrome;
        let svg = r##"<svg><rect fill="#ff0000"/><circle fill="#00ff00"/></svg>"##;
        let result = theme.apply_to_svg(svg);
        // Colors should be replaced (no longer pure red/green)
        assert!(!result.contains("#ff0000"), "Red should be remapped");
        assert!(!result.contains("#00ff00"), "Green should be remapped");
        // Should still be valid SVG structure
        assert!(result.contains("<svg>"));
        assert!(result.contains("</svg>"));
    }

    #[test]
    fn test_apply_to_svg_preserves_structure() {
        let theme = Theme::Warm;
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256"><rect width="256" height="256" fill="#aabbcc"/></svg>"##;
        let result = theme.apply_to_svg(svg);
        assert!(result.starts_with("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("width=\"256\""));
        assert!(result.contains("height=\"256\""));
    }

    #[test]
    fn test_apply_to_svg_no_colors() {
        let theme = Theme::Cool;
        let svg = "<svg><text>Hello</text></svg>";
        let result = theme.apply_to_svg(svg);
        assert_eq!(result, svg, "SVG without colors should be unchanged");
    }

    #[test]
    fn test_apply_to_image_modifies_pixels() {
        let theme = Theme::Monochrome;
        let mut img = RgbaImage::from_pixel(4, 4, Rgba([200, 50, 50, 255]));
        theme.apply_to_image(&mut img);
        let p = img.get_pixel(0, 0);
        // Should be near-gray (monochrome desaturates)
        let diff = (p[0] as i16 - p[1] as i16).unsigned_abs() + (p[1] as i16 - p[2] as i16).unsigned_abs();
        assert!(diff < 15, "Monochrome should produce near-gray pixels, got {:?}", p);
    }

    #[test]
    fn test_apply_to_image_skips_transparent() {
        let theme = Theme::Warm;
        let mut img = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 0]));
        theme.apply_to_image(&mut img);
        let p = img.get_pixel(0, 0);
        assert_eq!(p, &Rgba([0, 0, 0, 0]), "Transparent pixels should be unchanged");
    }

    #[test]
    fn test_available_themes_count() {
        assert_eq!(available_themes().len(), 9);
    }

    #[test]
    fn test_is_valid_theme() {
        assert!(is_valid_theme("warm"));
        assert!(is_valid_theme("Neon"));
        assert!(is_valid_theme("mono"));
        assert!(!is_valid_theme("rainbow"));
        assert!(!is_valid_theme(""));
    }

    #[test]
    fn test_remap_hue_full_spectrum() {
        // Full spectrum (range >= 180) should return original hue
        assert_eq!(remap_hue(120.0, 180.0, 180.0), 120.0);
        assert_eq!(remap_hue(0.0, 180.0, 180.0), 0.0);
        assert_eq!(remap_hue(359.0, 180.0, 180.0), 359.0);
    }

    #[test]
    fn test_remap_hue_monochrome() {
        // Monochrome (range < 1) should return original hue
        assert_eq!(remap_hue(120.0, 0.0, 0.0), 120.0);
    }

    #[test]
    fn test_remap_hue_constrained() {
        // Warm: center 25, range 50 → maps 0-360 into -25..75
        let h = remap_hue(180.0, 25.0, 50.0);
        let in_range = h <= 80.0 || h >= 330.0;
        assert!(in_range, "Hue 180 remapped to warm should be in range, got {h}");
    }

    #[test]
    fn test_earth_theme_brown_tones() {
        let theme = Theme::Earth;
        let (r, g, b) = theme.remap_color(100, 100, 200);
        let (h, s, _l) = rgb_to_hsl(r, g, b);
        // Earth: center 30, range 25 → hues 5-55
        assert!(h >= 0.0 && h <= 60.0, "Earth should produce warm-brown hues, got {h}°");
        assert!(s <= 0.6, "Earth should be muted, got s={s:.3}");
    }

    #[test]
    fn test_forest_theme_green_tones() {
        let theme = Theme::Forest;
        let (r, g, b) = theme.remap_color(200, 50, 50);
        let (h, _s, _l) = rgb_to_hsl(r, g, b);
        // Forest: center 130, range 50 → hues 80-180
        assert!(h >= 75.0 && h <= 185.0, "Forest should produce green hues, got {h}°");
    }
}
