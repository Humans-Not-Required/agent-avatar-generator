use image::{ImageBuffer, Rgba, RgbaImage};
use sha2::{Digest, Sha256};

/// Hash a seed string into 32 bytes for deterministic generation.
pub fn hash_seed(seed: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    bytes
}

/// Extract a color from hash bytes at a given offset.
pub fn color_from_hash(hash: &[u8; 32], offset: usize) -> (u8, u8, u8) {
    let r = hash[offset % 32];
    let g = hash[(offset + 1) % 32];
    let b = hash[(offset + 2) % 32];
    // Ensure colors are not too dark or too light for visibility
    let brighten = |c: u8| -> u8 {
        let v = c as u16;
        ((v * 180 / 255) + 40) as u8 // Range: 40-220
    };
    (brighten(r), brighten(g), brighten(b))
}

/// Extract a background color (lighter/more pastel).
pub fn bg_color_from_hash(hash: &[u8; 32]) -> (u8, u8, u8) {
    let r = hash[29];
    let g = hash[30];
    let b = hash[31];
    // Pastel: high brightness, low saturation
    let pastel = |c: u8| -> u8 {
        let v = c as u16;
        ((v * 80 / 255) + 175) as u8 // Range: 175-255
    };
    (pastel(r), pastel(g), pastel(b))
}

/// Convert HSL to RGB. h: 0-360, s: 0.0-1.0, l: 0.0-1.0
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
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

/// Generate a harmonious color palette from hash bytes.
/// Returns 4-5 colors that work well together based on color theory.
fn harmonious_palette(hash: &[u8; 32]) -> Vec<(u8, u8, u8)> {
    // Base hue from first 2 bytes (0-360)
    let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
    let saturation = 0.55 + (hash[2] as f64 / 255.0) * 0.3; // 0.55-0.85
    let lightness = 0.45 + (hash[3] as f64 / 255.0) * 0.15; // 0.45-0.60

    // Choose harmony type from hash
    let harmony = hash[4] % 4;
    let hues = match harmony {
        0 => {
            // Complementary: base + opposite
            vec![base_hue, (base_hue + 180.0) % 360.0, (base_hue + 30.0) % 360.0, (base_hue + 210.0) % 360.0]
        }
        1 => {
            // Triadic: 120° apart
            vec![base_hue, (base_hue + 120.0) % 360.0, (base_hue + 240.0) % 360.0, (base_hue + 60.0) % 360.0]
        }
        2 => {
            // Analogous: close hues
            vec![base_hue, (base_hue + 30.0) % 360.0, (base_hue + 60.0) % 360.0, (base_hue + 330.0) % 360.0]
        }
        _ => {
            // Split-complementary: base + two flanking opposite
            vec![base_hue, (base_hue + 150.0) % 360.0, (base_hue + 210.0) % 360.0, (base_hue + 90.0) % 360.0]
        }
    };

    hues.iter()
        .enumerate()
        .map(|(i, &h)| {
            // Vary lightness slightly per color
            let l_offset = (i as f64 * 0.06) - 0.09;
            hsl_to_rgb(h, saturation, (lightness + l_offset).clamp(0.3, 0.7))
        })
        .collect()
}

/// Generate a mosaic avatar with harmonious color palette.
pub fn generate_mosaic(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let palette = harmonious_palette(&hash);
    let bg = bg_override.unwrap_or_else(|| {
        let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
        hsl_to_rgb(base_hue, 0.15, 0.92) // Very light pastel bg
    });

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let grid = 6u32;
    let cell = size / grid;
    let margin = (cell as f64 * 0.12) as u32;

    for row in 0..grid {
        for col in 0..grid {
            let idx = (row * grid + col) as usize;
            let h = hash[(idx * 3) % 32];
            let shape = hash[(idx * 3 + 1) % 32];

            // Pick color from palette
            let color = palette[(h as usize) % palette.len()];
            let x0 = col * cell + margin;
            let y0 = row * cell + margin;
            let x1 = (col + 1) * cell - margin;
            let y1 = (row + 1) * cell - margin;
            let cx = x0 + (x1 - x0) / 2;
            let cy = y0 + (y1 - y0) / 2;
            let r = ((x1 - x0) / 2).min((y1 - y0) / 2);

            match shape % 5 {
                0 => {
                    // Full square
                    fill_rect(&mut img, x0, y0, x1, y1, color);
                }
                1 => {
                    // Circle
                    fill_circle(&mut img, cx, cy, r, color);
                }
                2 => {
                    // Diamond (filled rect rotated — approximate with triangle pairs)
                    // Top triangle
                    for dy in 0..r {
                        let w = dy;
                        fill_rect(&mut img, cx.saturating_sub(w), cy.saturating_sub(r) + dy, cx + w + 1, cy.saturating_sub(r) + dy + 1, color);
                    }
                    // Bottom triangle
                    for dy in 0..r {
                        let w = r - dy;
                        fill_rect(&mut img, cx.saturating_sub(w), cy + dy, cx + w + 1, cy + dy + 1, color);
                    }
                }
                3 => {
                    // Half-filled (top half)
                    fill_rect(&mut img, x0, y0, x1, cy, color);
                }
                _ => {
                    // Quarter circle (bottom-right)
                    for py in cy..y1 {
                        for px in cx..x1 {
                            let dx = (px as f64 - cx as f64).abs();
                            let dy = (py as f64 - cy as f64).abs();
                            if dx * dx + dy * dy <= (r as f64 * r as f64)
                                && px < size && py < size
                            {
                                img.put_pixel(px, py, Rgba([color.0, color.1, color.2, 255]));
                            }
                        }
                    }
                }
            }
        }
    }

    img
}

/// Generate a geometric identicon (5×5 grid, vertically symmetric).
pub fn generate_geometric(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let fg = color_from_hash(&hash, 0);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    let grid_size = 5u32;
    let cell_size = size / grid_size;
    let actual_size = cell_size * grid_size;
    let mut img: RgbaImage = ImageBuffer::new(actual_size, actual_size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    // Determine which cells are filled (symmetric around vertical axis)
    // We only need to decide for the left half + center column
    for row in 0..grid_size {
        for col in 0..grid_size.div_ceil(2) {
            let byte_idx = (row * 3 + col) as usize;
            let filled = hash[byte_idx % 32] > 127;

            if filled {
                // Fill this cell and its mirror
                fill_cell(&mut img, col, row, cell_size, fg);
                let mirror_col = grid_size - 1 - col;
                fill_cell(&mut img, mirror_col, row, cell_size, fg);
            }
        }
    }

    img
}

/// Generate a rings-style avatar (concentric rings).
pub fn generate_rings(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let center = size as f64 / 2.0;
    let num_rings = 5u32;
    let ring_width = center / num_rings as f64;

    for ring in 0..num_rings {
        let color = color_from_hash(&hash, (ring as usize) * 3);
        let outer_r = center - (ring as f64 * ring_width);
        let inner_r = outer_r - ring_width * 0.7;

        for y in 0..size {
            for x in 0..size {
                let dx = x as f64 - center;
                let dy = y as f64 - center;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= outer_r && dist >= inner_r {
                    img.put_pixel(x, y, Rgba([color.0, color.1, color.2, 255]));
                }
            }
        }
    }

    img
}

/// Generate a robot face avatar.
pub fn generate_robot(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let body_color = color_from_hash(&hash, 0);
    let eye_color = color_from_hash(&hash, 3);
    let accent_color = color_from_hash(&hash, 12);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let s = size as f64;

    // Head (rounded rectangle area)
    let head_left = (s * 0.2) as u32;
    let head_right = (s * 0.8) as u32;
    let head_top = (s * 0.2) as u32;
    let head_bottom = (s * 0.7) as u32;
    fill_rect(&mut img, head_left, head_top, head_right, head_bottom, body_color);

    // Ears (drawn after head so they layer properly)
    let ear_style = hash[12] % 4;
    let ear_w = (s * 0.06) as u32;
    let ear_h = (s * 0.12) as u32;
    let ear_y = (s * 0.35) as u32;
    match ear_style {
        1 => {
            // Rectangular side ears
            fill_rect(&mut img, head_left.saturating_sub(ear_w), ear_y, head_left, ear_y + ear_h, accent_color);
            fill_rect(&mut img, head_right, ear_y, head_right + ear_w, ear_y + ear_h, accent_color);
        }
        2 => {
            // Round disc ears
            let er = (s * 0.05) as u32;
            fill_circle(&mut img, head_left, ear_y + ear_h / 2, er, accent_color);
            fill_circle(&mut img, head_right, ear_y + ear_h / 2, er, accent_color);
        }
        3 => {
            // Pointed ears (triangular — approximated with stacked rects)
            let tri_h = (s * 0.1) as u32;
            for i in 0..tri_h {
                let w = ear_w.saturating_sub(i * ear_w / tri_h);
                if w > 0 {
                    // Left ear
                    fill_rect(&mut img, head_left.saturating_sub(w), ear_y + i, head_left, ear_y + i + 1, accent_color);
                    // Right ear
                    fill_rect(&mut img, head_right, ear_y + i, head_right + w, ear_y + i + 1, accent_color);
                }
            }
        }
        _ => {} // No ears
    }

    // Antenna (varies based on hash)
    let antenna_style = hash[6] % 3;
    let antenna_x = size / 2;
    let antenna_top = (s * 0.05) as u32;
    let antenna_base = head_top;
    match antenna_style {
        0 => {
            // Straight antenna with ball
            fill_rect(&mut img, antenna_x - 1, antenna_top, antenna_x + 2, antenna_base, body_color);
            fill_circle(&mut img, antenna_x, antenna_top, (s * 0.04) as u32, eye_color);
        }
        1 => {
            // V antenna
            let spread = (s * 0.08) as u32;
            fill_rect(&mut img, antenna_x - spread, antenna_top, antenna_x - spread + 2, antenna_base, body_color);
            fill_rect(&mut img, antenna_x + spread - 1, antenna_top, antenna_x + spread + 2, antenna_base, body_color);
        }
        _ => {
            // T antenna
            fill_rect(&mut img, antenna_x - 1, antenna_top + 3, antenna_x + 2, antenna_base, body_color);
            let bar_w = (s * 0.1) as u32;
            fill_rect(&mut img, antenna_x - bar_w, antenna_top, antenna_x + bar_w, antenna_top + 3, eye_color);
        }
    }

    // Forehead marking
    let forehead_style = hash[14] % 3;
    match forehead_style {
        1 => {
            // LED dot on forehead
            let led_y = (s * 0.26) as u32;
            let led_r = (s * 0.025) as u32;
            fill_circle(&mut img, size / 2, led_y, led_r, accent_color);
        }
        2 => {
            // Horizontal stripe
            let stripe_y = (s * 0.25) as u32;
            let stripe_w = (s * 0.15) as u32;
            fill_rect(&mut img, size / 2 - stripe_w, stripe_y, size / 2 + stripe_w, stripe_y + 2, accent_color);
        }
        _ => {} // No marking
    }

    // Visor (drawn before eyes so eyes sit on top)
    let visor_style = hash[13] % 3;
    let eye_y = (s * 0.38) as u32;
    if visor_style > 0 {
        let visor_h = (s * 0.06) as u32;
        let visor_left = head_left + (s * 0.04) as u32;
        let visor_right = head_right - (s * 0.04) as u32;
        // Darken the visor area
        let visor_color = (
            body_color.0.saturating_sub(30),
            body_color.1.saturating_sub(30),
            body_color.2.saturating_sub(30),
        );
        if visor_style == 1 {
            // Full band visor
            fill_rect(&mut img, visor_left, eye_y - visor_h, visor_right, eye_y + visor_h, visor_color);
        } else {
            // Segmented visor (two blocks around eye positions)
            let seg_w = (s * 0.14) as u32;
            let left_eye_x = (s * 0.35) as u32;
            let right_eye_x = (s * 0.65) as u32;
            fill_rect(&mut img, left_eye_x - seg_w, eye_y - visor_h, left_eye_x + seg_w, eye_y + visor_h, visor_color);
            fill_rect(&mut img, right_eye_x - seg_w, eye_y - visor_h, right_eye_x + seg_w, eye_y + visor_h, visor_color);
        }
    }

    // Eyes
    let eye_size = (s * 0.08) as u32;
    let eye_shape = hash[7] % 3;
    let left_eye_x = (s * 0.35) as u32;
    let right_eye_x = (s * 0.65) as u32;

    match eye_shape {
        0 => {
            // Round eyes
            fill_circle(&mut img, left_eye_x, eye_y, eye_size, eye_color);
            fill_circle(&mut img, right_eye_x, eye_y, eye_size, eye_color);
        }
        1 => {
            // Square eyes
            fill_rect(&mut img, left_eye_x - eye_size, eye_y - eye_size, left_eye_x + eye_size, eye_y + eye_size, eye_color);
            fill_rect(&mut img, right_eye_x - eye_size, eye_y - eye_size, right_eye_x + eye_size, eye_y + eye_size, eye_color);
        }
        _ => {
            // Pixel eyes (small dots)
            let dot = eye_size / 2;
            fill_rect(&mut img, left_eye_x - dot, eye_y - dot, left_eye_x + dot, eye_y + dot, eye_color);
            fill_rect(&mut img, right_eye_x - dot, eye_y - dot, right_eye_x + dot, eye_y + dot, eye_color);
        }
    }

    // Cheek bolts
    let bolt_style = hash[15] % 2;
    if bolt_style == 1 {
        let bolt_r = (s * 0.02) as u32;
        let bolt_y = (s * 0.48) as u32;
        let bolt_lx = (s * 0.25) as u32;
        let bolt_rx = (s * 0.75) as u32;
        fill_circle(&mut img, bolt_lx, bolt_y, bolt_r, accent_color);
        fill_circle(&mut img, bolt_rx, bolt_y, bolt_r, accent_color);
    }

    // Mouth
    let mouth_y = (s * 0.55) as u32;
    let mouth_style = hash[8] % 4;
    let mouth_color = color_from_hash(&hash, 9);

    match mouth_style {
        0 => {
            // Wide grin
            let w = (s * 0.2) as u32;
            fill_rect(&mut img, size / 2 - w, mouth_y, size / 2 + w, mouth_y + 3, mouth_color);
        }
        1 => {
            // Grid mouth (speaker)
            let w = (s * 0.15) as u32;
            let h = (s * 0.06) as u32;
            for row in 0..3 {
                let y = mouth_y + row * (h / 2);
                fill_rect(&mut img, size / 2 - w, y, size / 2 + w, y + 2, mouth_color);
            }
        }
        2 => {
            // Circle mouth
            fill_circle(&mut img, size / 2, mouth_y, (s * 0.05) as u32, mouth_color);
        }
        _ => {
            // Zigzag mouth
            let w = (s * 0.15) as u32;
            let step = w / 3;
            for i in 0..6 {
                let x = size / 2 - w + i * step;
                let y_off = if i % 2 == 0 { 0 } else { 3 };
                fill_rect(&mut img, x, mouth_y + y_off, x + step / 2, mouth_y + y_off + 2, mouth_color);
            }
        }
    }

    // Chin plate
    let chin_style = hash[16] % 3;
    if chin_style > 0 {
        let chin_y = (s * 0.62) as u32;
        let chin_w = if chin_style == 1 { (s * 0.1) as u32 } else { (s * 0.18) as u32 };
        let chin_h = (s * 0.03) as u32;
        fill_rect(&mut img, size / 2 - chin_w, chin_y, size / 2 + chin_w, chin_y + chin_h, accent_color);
    }

    img
}

/// Generate a blockies-style avatar (8×8 pixel grid, Ethereum-style).
pub fn generate_blockies(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    let grid = 8u32;
    let cell_size = size / grid;
    let actual_size = cell_size * grid;
    let mut img: RgbaImage = ImageBuffer::new(actual_size, actual_size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    for row in 0..grid {
        for col in 0..grid {
            let idx = (row * grid + col) as usize;
            let val = hash[idx % 32];
            let color = if val > 170 {
                Some(color1)
            } else if val > 85 {
                Some(color2)
            } else {
                None // background
            };

            if let Some(c) = color {
                fill_cell(&mut img, col, row, cell_size, c);
            }
        }
    }

    img
}

/// Generate a gradient-style avatar with overlay shape.
pub fn generate_gradient(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let color1 = bg_override.unwrap_or_else(|| color_from_hash(&hash, 0));
    let color2 = color_from_hash(&hash, 3);
    let shape_color = color_from_hash(&hash, 6);

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    let center = size as f64 / 2.0;

    // Linear gradient (angle from hash)
    let angle = (hash[9] as f64 / 255.0) * std::f64::consts::PI;
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    for y in 0..size {
        for x in 0..size {
            let nx = (x as f64 - center) / center;
            let ny = (y as f64 - center) / center;
            let t = ((nx * cos_a + ny * sin_a) + 1.0) / 2.0;
            let t = t.clamp(0.0, 1.0);

            let r = lerp_u8(color1.0, color2.0, t);
            let g = lerp_u8(color1.1, color2.1, t);
            let b = lerp_u8(color1.2, color2.2, t);
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Overlay shape
    let shape = hash[10] % 3;
    let shape_size = center * 0.5;

    match shape {
        0 => {
            // Circle
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f64 - center;
                    let dy = y as f64 - center;
                    if (dx * dx + dy * dy).sqrt() < shape_size {
                        img.put_pixel(x, y, Rgba([shape_color.0, shape_color.1, shape_color.2, 180]));
                    }
                }
            }
        }
        1 => {
            // Diamond
            for y in 0..size {
                for x in 0..size {
                    let dx = (x as f64 - center).abs();
                    let dy = (y as f64 - center).abs();
                    if dx + dy < shape_size {
                        img.put_pixel(x, y, Rgba([shape_color.0, shape_color.1, shape_color.2, 180]));
                    }
                }
            }
        }
        _ => {
            // Hexagon
            for y in 0..size {
                for x in 0..size {
                    let dx = (x as f64 - center).abs();
                    let dy = (y as f64 - center).abs();
                    if dy < shape_size * 0.866 && dx + dy * 0.577 < shape_size {
                        img.put_pixel(x, y, Rgba([shape_color.0, shape_color.1, shape_color.2, 180]));
                    }
                }
            }
        }
    }

    img
}

/// Embedded 5×7 bitmap font for initials style.
/// Each character is a u64 where the lower 35 bits represent a 5-wide × 7-tall grid.
/// Bit layout: row 0 (top) bits 34-30, row 1 bits 29-25, ..., row 6 bits 4-0.
fn bitmap_font() -> [u64; 36] {
    [
        // A-Z (indices 0-25)
        0b01110_10001_10001_11111_10001_10001_10001, // A
        0b11110_10001_10001_11110_10001_10001_11110, // B
        0b01110_10001_10000_10000_10000_10001_01110, // C
        0b11100_10010_10001_10001_10001_10010_11100, // D
        0b11111_10000_10000_11110_10000_10000_11111, // E
        0b11111_10000_10000_11110_10000_10000_10000, // F
        0b01110_10001_10000_10111_10001_10001_01110, // G
        0b10001_10001_10001_11111_10001_10001_10001, // H
        0b01110_00100_00100_00100_00100_00100_01110, // I
        0b00111_00010_00010_00010_00010_10010_01100, // J
        0b10001_10010_10100_11000_10100_10010_10001, // K
        0b10000_10000_10000_10000_10000_10000_11111, // L
        0b10001_11011_10101_10101_10001_10001_10001, // M
        0b10001_11001_10101_10011_10001_10001_10001, // N
        0b01110_10001_10001_10001_10001_10001_01110, // O
        0b11110_10001_10001_11110_10000_10000_10000, // P
        0b01110_10001_10001_10001_10101_10010_01101, // Q
        0b11110_10001_10001_11110_10100_10010_10001, // R
        0b01110_10001_10000_01110_00001_10001_01110, // S
        0b11111_00100_00100_00100_00100_00100_00100, // T
        0b10001_10001_10001_10001_10001_10001_01110, // U
        0b10001_10001_10001_10001_01010_01010_00100, // V
        0b10001_10001_10001_10101_10101_11011_10001, // W
        0b10001_01010_00100_00100_00100_01010_10001, // X
        0b10001_01010_00100_00100_00100_00100_00100, // Y
        0b11111_00001_00010_00100_01000_10000_11111, // Z
        // 0-9 (indices 26-35)
        0b01110_10001_10011_10101_11001_10001_01110, // 0
        0b00100_01100_00100_00100_00100_00100_01110, // 1
        0b01110_10001_00001_00110_01000_10000_11111, // 2
        0b01110_10001_00001_00110_00001_10001_01110, // 3
        0b00010_00110_01010_10010_11111_00010_00010, // 4
        0b11111_10000_11110_00001_00001_10001_01110, // 5
        0b01110_10001_10000_11110_10001_10001_01110, // 6
        0b11111_00001_00010_00100_01000_01000_01000, // 7
        0b01110_10001_10001_01110_10001_10001_01110, // 8
        0b01110_10001_10001_01111_00001_10001_01110, // 9
    ]
}

/// Get bitmap index for a character (A-Z → 0-25, 0-9 → 26-35), or None.
fn char_to_font_idx(c: char) -> Option<usize> {
    match c {
        'A'..='Z' => Some((c as u8 - b'A') as usize),
        'a'..='z' => Some((c as u8 - b'a') as usize),
        '0'..='9' => Some(26 + (c as u8 - b'0') as usize),
        _ => None,
    }
}

/// Extract 1-2 initials from a seed string.
fn extract_initials(seed: &str) -> Vec<usize> {
    let mut result = Vec::new();

    // Try to find alphanumeric characters
    for c in seed.chars() {
        if let Some(idx) = char_to_font_idx(c) {
            result.push(idx);
            if result.len() >= 2 {
                break;
            }
        }
    }

    // If we got nothing, use hash bytes as fallback
    if result.is_empty() {
        let hash = hash_seed(seed);
        result.push((hash[0] % 26) as usize); // A-Z
    }

    result
}

/// Render a bitmap character onto an image at (ox, oy) with given scale.
fn render_char(
    img: &mut RgbaImage,
    font: &[u64; 36],
    idx: usize,
    ox: u32,
    oy: u32,
    scale: u32,
    color: (u8, u8, u8),
) {
    let bits = font[idx];
    for row in 0..7u32 {
        for col in 0..5u32 {
            let bit_pos = (6 - row) * 5 + (4 - col);
            if (bits >> bit_pos) & 1 == 1 {
                // Fill a scale × scale block
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = ox + col * scale + dx;
                        let py = oy + row * scale + dy;
                        if px < img.width() && py < img.height() {
                            img.put_pixel(px, py, Rgba([color.0, color.1, color.2, 255]));
                        }
                    }
                }
            }
        }
    }
}

/// Generate an initials-style avatar (1-2 letters on colored background).
pub fn generate_initials(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| color_from_hash(&hash, 0));
    // Ensure letter color contrasts well with background
    let letter_color = {
        let brightness = (bg.0 as u16 + bg.1 as u16 + bg.2 as u16) / 3;
        if brightness > 140 {
            (40, 40, 40) // Dark text on light bg
        } else {
            (240, 240, 240) // Light text on dark bg
        }
    };

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let initials = extract_initials(seed);
    let font = bitmap_font();
    let num_chars = initials.len() as u32;

    // Calculate scale: each char is 5 wide × 7 tall
    // For 1 char: fill ~60% of width, for 2 chars: fit both with gap
    let char_w = 5u32;
    let char_h = 7u32;
    let total_w = num_chars * char_w + (num_chars - 1); // chars + gaps

    // Scale to fit ~70% of avatar width
    let target_w = (size as f64 * 0.7) as u32;
    let scale = (target_w / total_w).max(1).min(size / char_h);

    // Center vertically and horizontally
    let rendered_w = num_chars * char_w * scale + (num_chars - 1) * scale;
    let rendered_h = char_h * scale;
    let start_x = (size.saturating_sub(rendered_w)) / 2;
    let start_y = (size.saturating_sub(rendered_h)) / 2;

    for (i, &idx) in initials.iter().enumerate() {
        let ox = start_x + i as u32 * (char_w * scale + scale);
        render_char(&mut img, &font, idx, ox, start_y, scale, letter_color);
    }

    img
}

/// Generate a starburst-style avatar (radial rays from center).
pub fn generate_starburst(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let color3 = color_from_hash(&hash, 6);

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Fill background
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let center = size as f64 / 2.0;
    // Number of rays: 8-20 based on hash
    let num_rays = 8 + (hash[12] % 13) as u32;
    let rotation = (hash[13] as f64 / 255.0) * std::f64::consts::PI * 2.0;
    let ray_angle = std::f64::consts::PI * 2.0 / num_rays as f64;

    let colors = [color1, color2, color3];
    let max_radius = center * 0.9;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f64 - center;
            let dy = y as f64 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > max_radius {
                continue;
            }

            let mut angle = dy.atan2(dx) + rotation;
            if angle < 0.0 {
                angle += std::f64::consts::PI * 2.0;
            }

            // Determine which ray this pixel belongs to
            let ray_idx = (angle / ray_angle) as u32;
            if ray_idx.is_multiple_of(2) {
                let color = colors[(ray_idx as usize / 2) % colors.len()];
                // Fade toward edges
                let alpha = 1.0 - (dist / max_radius).powi(2);
                let blend = |fg: u8, bg: u8| -> u8 {
                    ((fg as f64 * alpha + bg as f64 * (1.0 - alpha)).round()) as u8
                };
                let r = blend(color.0, bg.0);
                let g = blend(color.1, bg.1);
                let b = blend(color.2, bg.2);
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    // Center dot
    let dot_radius = (size as f64 * 0.08) as u32;
    let dot_color = color_from_hash(&hash, 15);
    fill_circle(&mut img, size / 2, size / 2, dot_radius, dot_color);

    img
}

/// Generate a pixel art creature avatar (space-invader inspired, horizontal symmetry).
pub fn generate_pixel(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let color3 = color_from_hash(&hash, 6);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    // 11×11 grid with horizontal symmetry (odd for center column)
    let grid = 11u32;
    let half = grid / 2 + 1; // 6 columns to compute (including center)

    // Cell size with padding around the creature
    let cell = size / (grid + 2);
    let offset = (size - cell * grid) / 2;

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    // Gap between pixels for retro look (1px minimum, ~10% of cell)
    let gap = (cell / 10).clamp(1, 3);

    let colors = [color1, color2, color3];

    for row in 0..grid {
        for col in 0..half {
            let idx = (row * half + col) as usize;
            let byte = hash[idx % 32];

            // Shape probability: center columns and middle rows more likely filled
            let col_dist = (half as i32 - 1 - col as i32).unsigned_abs();
            let row_center = grid / 2;
            let row_dist = row.abs_diff(row_center);

            // Base threshold — lower means more likely to be filled
            // Center is ~80, edges ~140, top/bottom rows add penalty
            let threshold = 80 + col_dist * 14 + row_dist * 8;

            if (byte as u32) > threshold {
                // Pick color deterministically
                let color_byte = hash[(idx * 3 + 11) % 32];
                let color = colors[(color_byte as usize) % 3];

                // Draw this cell with gap
                let x = offset + col * cell;
                let y = offset + row * cell;
                fill_rect(
                    &mut img,
                    x + gap,
                    y + gap,
                    x + cell - gap,
                    y + cell - gap,
                    color,
                );

                // Mirror horizontally (skip if this IS the center column)
                if col < grid / 2 {
                    let mx = offset + (grid - 1 - col) * cell;
                    fill_rect(
                        &mut img,
                        mx + gap,
                        y + gap,
                        mx + cell - gap,
                        y + cell - gap,
                        color,
                    );
                }
            }
        }
    }

    img
}

/// Generate a layered sunset/horizon avatar using harmonious color palette.
pub fn generate_sunset(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> RgbaImage {
    let hash = hash_seed(seed);
    let palette = harmonious_palette(&hash);
    let bg = bg_override.unwrap_or_else(|| {
        // Deep sky color as background
        let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
        hsl_to_rgb(base_hue, 0.6, 0.25)
    });

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let s = size as f64;

    // Number of horizontal bands (4-6 based on hash)
    let num_bands = 4 + (hash[5] % 3) as usize;

    // Build band colors from palette + variations
    let mut band_colors: Vec<(u8, u8, u8)> = Vec::new();
    for i in 0..num_bands {
        let base = palette[i % palette.len()];
        // Shift lightness progressively: top bands darker, bottom brighter
        let factor = 0.7 + (i as f64 / num_bands as f64) * 0.6; // 0.7 → 1.3
        let r = ((base.0 as f64 * factor).min(255.0)) as u8;
        let g = ((base.1 as f64 * factor).min(255.0)) as u8;
        let b = ((base.2 as f64 * factor).min(255.0)) as u8;
        band_colors.push((r, g, b));
    }

    // Band boundaries with wave distortion
    let band_height = s / num_bands as f64;

    for band_idx in 0..num_bands {
        let base_y = (band_idx as f64 * band_height) as u32;
        let next_y = ((band_idx + 1) as f64 * band_height) as u32;
        let color = band_colors[band_idx];
        let next_color = if band_idx + 1 < num_bands {
            band_colors[band_idx + 1]
        } else {
            color
        };

        // Wave parameters for this band's top edge
        let wave_amp = s * 0.02 + (hash[(band_idx * 4 + 10) % 32] as f64 / 255.0) * s * 0.03;
        let wave_freq = 2.0 + (hash[(band_idx * 4 + 11) % 32] as f64 / 255.0) * 3.0;
        let wave_phase = (hash[(band_idx * 4 + 12) % 32] as f64 / 255.0) * std::f64::consts::PI * 2.0;

        for y in base_y..next_y.min(size) {
            // Progress through this band (0.0 → 1.0)
            let t = (y - base_y) as f64 / (next_y - base_y).max(1) as f64;

            // Blend with next band near the boundary
            let blend_start = 0.7;
            let (r, g, b) = if t > blend_start {
                let bt = (t - blend_start) / (1.0 - blend_start);
                (
                    lerp_u8(color.0, next_color.0, bt),
                    lerp_u8(color.1, next_color.1, bt),
                    lerp_u8(color.2, next_color.2, bt),
                )
            } else {
                color
            };

            for x in 0..size {
                // Apply wave distortion to the y coordinate
                let wave = (x as f64 / s * wave_freq * std::f64::consts::PI + wave_phase).sin() * wave_amp;
                let effective_y = y as f64 + wave;

                // Only draw if the effective y is within this band's range
                if effective_y >= base_y as f64 && effective_y < next_y as f64 {
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
    }

    // Sun/moon circle (position and size from hash)
    let sun_present = hash[7] % 3 != 0; // 2/3 chance of having a sun
    if sun_present {
        let sun_x = s * 0.2 + (hash[8] as f64 / 255.0) * s * 0.6;
        let sun_y = s * 0.15 + (hash[9] as f64 / 255.0) * s * 0.3;
        let sun_r = s * 0.06 + (hash[10] as f64 / 255.0) * s * 0.06;

        // Sun color: warm tint of first palette color
        let sun_base = palette[0];
        let sun_color = (
            ((sun_base.0 as u16 + 255) / 2) as u8,
            ((sun_base.1 as u16 + 200) / 2) as u8,
            ((sun_base.2 as u16 + 150) / 2) as u8,
        );

        // Draw sun with soft glow
        let glow_r = sun_r * 1.8;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f64 - sun_x;
                let dy = y as f64 - sun_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < sun_r {
                    // Solid sun
                    img.put_pixel(x, y, Rgba([sun_color.0, sun_color.1, sun_color.2, 255]));
                } else if dist < glow_r {
                    // Glow: blend with existing pixel
                    let glow_t = 1.0 - (dist - sun_r) / (glow_r - sun_r);
                    let alpha = (glow_t * glow_t * 120.0) as u8; // Quadratic falloff
                    let existing = img.get_pixel(x, y);
                    let r = lerp_u8(existing[0], sun_color.0, alpha as f64 / 255.0);
                    let g = lerp_u8(existing[1], sun_color.1, alpha as f64 / 255.0);
                    let b = lerp_u8(existing[2], sun_color.2, alpha as f64 / 255.0);
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
    }

    img
}

/// Generate an avatar as PNG bytes.
pub fn generate_png(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>) -> Result<Vec<u8>, String> {
    let img = generate_image(seed, style, size, bg)?;
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding error: {e}"))?;
    Ok(buf)
}

/// Generate an avatar as SVG string.
pub fn generate_svg(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>) -> Result<String, String> {
    match style {
        "geometric" => Ok(svg_geometric(seed, size, bg)),
        "rings" => Ok(svg_rings(seed, size, bg)),
        "robot" => Ok(svg_robot(seed, size, bg)),
        "blockies" => Ok(svg_blockies(seed, size, bg)),
        "gradient" => Ok(svg_gradient(seed, size, bg)),
        "initials" => Ok(svg_initials(seed, size, bg)),
        "starburst" => Ok(svg_starburst(seed, size, bg)),
        "mosaic" => Ok(svg_mosaic(seed, size, bg)),
        "pixel" => Ok(svg_pixel(seed, size, bg)),
        "sunset" => Ok(svg_sunset(seed, size, bg)),
        _ => Err(format!("Unknown style: {style}")),
    }
}

/// Generate an image (dispatches to the right style).
pub fn generate_image(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>) -> Result<RgbaImage, String> {
    match style {
        "geometric" => Ok(generate_geometric(seed, size, bg)),
        "rings" => Ok(generate_rings(seed, size, bg)),
        "robot" => Ok(generate_robot(seed, size, bg)),
        "blockies" => Ok(generate_blockies(seed, size, bg)),
        "gradient" => Ok(generate_gradient(seed, size, bg)),
        "initials" => Ok(generate_initials(seed, size, bg)),
        "starburst" => Ok(generate_starburst(seed, size, bg)),
        "mosaic" => Ok(generate_mosaic(seed, size, bg)),
        "pixel" => Ok(generate_pixel(seed, size, bg)),
        "sunset" => Ok(generate_sunset(seed, size, bg)),
        _ => Err(format!("Unknown style: {style}")),
    }
}

// ── Helpers ──

fn fill_cell(img: &mut RgbaImage, col: u32, row: u32, cell_size: u32, color: (u8, u8, u8)) {
    let x0 = col * cell_size;
    let y0 = row * cell_size;
    for y in y0..y0 + cell_size {
        for x in x0..x0 + cell_size {
            if x < img.width() && y < img.height() {
                img.put_pixel(x, y, Rgba([color.0, color.1, color.2, 255]));
            }
        }
    }
}

fn fill_rect(img: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32, color: (u8, u8, u8)) {
    for y in y1..y2.min(img.height()) {
        for x in x1..x2.min(img.width()) {
            img.put_pixel(x, y, Rgba([color.0, color.1, color.2, 255]));
        }
    }
}

fn fill_circle(img: &mut RgbaImage, cx: u32, cy: u32, radius: u32, color: (u8, u8, u8)) {
    let r = radius as i64;
    let cx = cx as i64;
    let cy = cy as i64;
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r * r {
                    img.put_pixel(x as u32, y as u32, Rgba([color.0, color.1, color.2, 255]));
                }
            }
        }
    }
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    ((a as f64) * (1.0 - t) + (b as f64) * t) as u8
}

fn hex_color(c: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

// ── SVG Generators ──

fn svg_geometric(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let fg = color_from_hash(&hash, 0);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let grid_size = 5u32;
    let cell_size = size / grid_size;

    let mut rects = String::new();
    for row in 0..grid_size {
        for col in 0..grid_size.div_ceil(2) {
            let byte_idx = (row * 3 + col) as usize;
            if hash[byte_idx % 32] > 127 {
                let x = col * cell_size;
                let y = row * cell_size;
                rects.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{cell_size}" height="{cell_size}" fill="{fill}"/>"#,
                    fill = hex_color(fg)
                ));
                let mirror_x = (grid_size - 1 - col) * cell_size;
                if mirror_x != x {
                    rects.push_str(&format!(
                        r#"<rect x="{mirror_x}" y="{y}" width="{cell_size}" height="{cell_size}" fill="{fill}"/>"#,
                        fill = hex_color(fg)
                    ));
                }
            }
        }
    }

    let actual = cell_size * grid_size;
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{actual}" height="{actual}" viewBox="0 0 {actual} {actual}"><rect width="{actual}" height="{actual}" fill="{bg}"/>{rects}</svg>"#,
        bg = hex_color(bg)
    )
}

fn svg_rings(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let center = size as f64 / 2.0;
    let num_rings = 5u32;
    let ring_width = center / num_rings as f64;

    let mut circles = String::new();
    for ring in 0..num_rings {
        let color = color_from_hash(&hash, (ring as usize) * 3);
        let outer_r = center - (ring as f64 * ring_width);
        let inner_r = outer_r - ring_width * 0.7;
        let stroke_w = outer_r - inner_r;
        let mid_r = (outer_r + inner_r) / 2.0;
        circles.push_str(&format!(
            r#"<circle cx="{center}" cy="{center}" r="{mid_r:.1}" fill="none" stroke="{}" stroke-width="{stroke_w:.1}"/>"#,
            hex_color(color)
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}"/>{circles}</svg>"#,
        bg = hex_color(bg)
    )
}

fn svg_robot(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let body_color = color_from_hash(&hash, 0);
    let eye_color = color_from_hash(&hash, 3);
    let mouth_color = color_from_hash(&hash, 9);
    let accent_color = color_from_hash(&hash, 12);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let s = size as f64;

    let mut parts = String::new();

    // Head
    let hl = (s * 0.2) as u32;
    let ht = (s * 0.2) as u32;
    let hw = (s * 0.6) as u32;
    let hh = (s * 0.5) as u32;
    let hr = hl + hw; // head right edge
    parts.push_str(&format!(
        r#"<rect x="{hl}" y="{ht}" width="{hw}" height="{hh}" rx="4" fill="{}"/>"#,
        hex_color(body_color)
    ));

    // Ears
    let ear_style = hash[12] % 4;
    let ear_w = (s * 0.06) as u32;
    let ear_h = (s * 0.12) as u32;
    let ear_y = (s * 0.35) as u32;
    match ear_style {
        1 => {
            // Rectangular side ears
            parts.push_str(&format!(
                r#"<rect x="{}" y="{ear_y}" width="{ear_w}" height="{ear_h}" rx="2" fill="{}"/>"#,
                hl.saturating_sub(ear_w), hex_color(accent_color)
            ));
            parts.push_str(&format!(
                r#"<rect x="{hr}" y="{ear_y}" width="{ear_w}" height="{ear_h}" rx="2" fill="{}"/>"#,
                hex_color(accent_color)
            ));
        }
        2 => {
            // Round disc ears
            let er = (s * 0.05) as u32;
            parts.push_str(&format!(
                r#"<circle cx="{hl}" cy="{}" r="{er}" fill="{}"/>"#,
                ear_y + ear_h / 2, hex_color(accent_color)
            ));
            parts.push_str(&format!(
                r#"<circle cx="{hr}" cy="{}" r="{er}" fill="{}"/>"#,
                ear_y + ear_h / 2, hex_color(accent_color)
            ));
        }
        3 => {
            // Pointed ears (triangles)
            let tri_h = (s * 0.1) as u32;
            // Left ear
            parts.push_str(&format!(
                r#"<polygon points="{},{} {},{} {},{}" fill="{}"/>"#,
                hl, ear_y, hl.saturating_sub(ear_w), ear_y + tri_h / 2, hl, ear_y + tri_h,
                hex_color(accent_color)
            ));
            // Right ear
            parts.push_str(&format!(
                r#"<polygon points="{},{} {},{} {},{}" fill="{}"/>"#,
                hr, ear_y, hr + ear_w, ear_y + tri_h / 2, hr, ear_y + tri_h,
                hex_color(accent_color)
            ));
        }
        _ => {} // No ears
    }

    // Antenna
    let ax = size / 2;
    let at = (s * 0.05) as u32;
    let antenna_style = hash[6] % 3;
    match antenna_style {
        0 => {
            parts.push_str(&format!(
                r#"<line x1="{ax}" y1="{at}" x2="{ax}" y2="{ht}" stroke="{}" stroke-width="3"/>"#,
                hex_color(body_color)
            ));
            let br = (s * 0.04) as u32;
            parts.push_str(&format!(
                r#"<circle cx="{ax}" cy="{at}" r="{br}" fill="{}"/>"#,
                hex_color(eye_color)
            ));
        }
        1 => {
            let sp = (s * 0.08) as u32;
            parts.push_str(&format!(
                r#"<line x1="{}" y1="{at}" x2="{}" y2="{ht}" stroke="{}" stroke-width="2"/>"#,
                ax - sp, ax - sp, hex_color(body_color)
            ));
            parts.push_str(&format!(
                r#"<line x1="{}" y1="{at}" x2="{}" y2="{ht}" stroke="{}" stroke-width="2"/>"#,
                ax + sp, ax + sp, hex_color(body_color)
            ));
        }
        _ => {
            parts.push_str(&format!(
                r#"<line x1="{ax}" y1="{}" x2="{ax}" y2="{ht}" stroke="{}" stroke-width="3"/>"#,
                at + 3, hex_color(body_color)
            ));
            let bw = (s * 0.1) as u32;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{at}" width="{}" height="3" fill="{}"/>"#,
                ax - bw, bw * 2, hex_color(eye_color)
            ));
        }
    }

    // Forehead marking
    let forehead_style = hash[14] % 3;
    match forehead_style {
        1 => {
            // LED dot
            let led_y = (s * 0.26) as u32;
            let led_r = (s * 0.025) as u32;
            parts.push_str(&format!(
                r#"<circle cx="{ax}" cy="{led_y}" r="{led_r}" fill="{}"/>"#,
                hex_color(accent_color)
            ));
        }
        2 => {
            // Horizontal stripe
            let stripe_y = (s * 0.25) as u32;
            let stripe_w = (s * 0.15) as u32;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{stripe_y}" width="{}" height="2" fill="{}"/>"#,
                ax - stripe_w, stripe_w * 2, hex_color(accent_color)
            ));
        }
        _ => {}
    }

    // Visor
    let visor_style = hash[13] % 3;
    let ey = (s * 0.38) as u32;
    if visor_style > 0 {
        let visor_h = (s * 0.06) as u32;
        let visor_left = hl + (s * 0.04) as u32;
        let visor_right = hr - (s * 0.04) as u32;
        let visor_color = (
            body_color.0.saturating_sub(30),
            body_color.1.saturating_sub(30),
            body_color.2.saturating_sub(30),
        );
        if visor_style == 1 {
            parts.push_str(&format!(
                r#"<rect x="{visor_left}" y="{}" width="{}" height="{}" rx="3" fill="{}"/>"#,
                ey - visor_h, visor_right - visor_left, visor_h * 2, hex_color(visor_color)
            ));
        } else {
            let seg_w = (s * 0.14) as u32;
            let le = (s * 0.35) as u32;
            let re = (s * 0.65) as u32;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="3" fill="{}"/>"#,
                le - seg_w, ey - visor_h, seg_w * 2, visor_h * 2, hex_color(visor_color)
            ));
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="3" fill="{}"/>"#,
                re - seg_w, ey - visor_h, seg_w * 2, visor_h * 2, hex_color(visor_color)
            ));
        }
    }

    // Eyes
    let es = (s * 0.08) as u32;
    let le = (s * 0.35) as u32;
    let re = (s * 0.65) as u32;
    let eye_shape = hash[7] % 3;
    match eye_shape {
        0 => {
            parts.push_str(&format!(r#"<circle cx="{le}" cy="{ey}" r="{es}" fill="{}"/>"#, hex_color(eye_color)));
            parts.push_str(&format!(r#"<circle cx="{re}" cy="{ey}" r="{es}" fill="{}"/>"#, hex_color(eye_color)));
        }
        1 => {
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                le - es, ey - es, es * 2, es * 2, hex_color(eye_color)
            ));
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                re - es, ey - es, es * 2, es * 2, hex_color(eye_color)
            ));
        }
        _ => {
            let d = es / 2;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                le - d, ey - d, d * 2, d * 2, hex_color(eye_color)
            ));
            parts.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                re - d, ey - d, d * 2, d * 2, hex_color(eye_color)
            ));
        }
    }

    // Cheek bolts
    let bolt_style = hash[15] % 2;
    if bolt_style == 1 {
        let bolt_r = (s * 0.02) as u32;
        let bolt_y = (s * 0.48) as u32;
        let bolt_lx = (s * 0.25) as u32;
        let bolt_rx = (s * 0.75) as u32;
        parts.push_str(&format!(r#"<circle cx="{bolt_lx}" cy="{bolt_y}" r="{bolt_r}" fill="{}"/>"#, hex_color(accent_color)));
        parts.push_str(&format!(r#"<circle cx="{bolt_rx}" cy="{bolt_y}" r="{bolt_r}" fill="{}"/>"#, hex_color(accent_color)));
    }

    // Mouth
    let my = (s * 0.55) as u32;
    let mouth_style = hash[8] % 4;
    match mouth_style {
        0 => {
            let w = (s * 0.2) as u32;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{my}" width="{}" height="3" fill="{}"/>"#,
                ax - w, w * 2, hex_color(mouth_color)
            ));
        }
        1 => {
            let w = (s * 0.15) as u32;
            for row in 0..3 {
                let y = my + row * 4;
                parts.push_str(&format!(
                    r#"<rect x="{}" y="{y}" width="{}" height="2" fill="{}"/>"#,
                    ax - w, w * 2, hex_color(mouth_color)
                ));
            }
        }
        2 => {
            let mr = (s * 0.05) as u32;
            parts.push_str(&format!(r#"<circle cx="{ax}" cy="{my}" r="{mr}" fill="{}"/>"#, hex_color(mouth_color)));
        }
        _ => {
            let w = (s * 0.2) as u32;
            parts.push_str(&format!(
                r#"<rect x="{}" y="{my}" width="{}" height="3" fill="{}"/>"#,
                ax - w, w * 2, hex_color(mouth_color)
            ));
        }
    }

    // Chin plate
    let chin_style = hash[16] % 3;
    if chin_style > 0 {
        let chin_y = (s * 0.62) as u32;
        let chin_w = if chin_style == 1 { (s * 0.1) as u32 } else { (s * 0.18) as u32 };
        let chin_h = (s * 0.03) as u32;
        parts.push_str(&format!(
            r#"<rect x="{}" y="{chin_y}" width="{}" height="{chin_h}" rx="1" fill="{}"/>"#,
            ax - chin_w, chin_w * 2, hex_color(accent_color)
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}"/>{parts}</svg>"#,
        bg = hex_color(bg)
    )
}

fn svg_blockies(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let grid = 8u32;
    let cell_size = size / grid;
    let actual = cell_size * grid;

    let mut rects = String::new();
    for row in 0..grid {
        for col in 0..grid {
            let idx = (row * grid + col) as usize;
            let val = hash[idx % 32];
            let color = if val > 170 {
                Some(color1)
            } else if val > 85 {
                Some(color2)
            } else {
                None
            };
            if let Some(c) = color {
                let x = col * cell_size;
                let y = row * cell_size;
                rects.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{cell_size}" height="{cell_size}" fill="{}"/>"#,
                    hex_color(c)
                ));
            }
        }
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{actual}" height="{actual}" viewBox="0 0 {actual} {actual}"><rect width="{actual}" height="{actual}" fill="{bg}"/>{rects}</svg>"#,
        bg = hex_color(bg)
    )
}

fn svg_gradient(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let color1 = bg_override.unwrap_or_else(|| color_from_hash(&hash, 0));
    let color2 = color_from_hash(&hash, 3);
    let shape_color = color_from_hash(&hash, 6);

    let angle = (hash[9] as f64 / 255.0) * 180.0;
    let center = size as f64 / 2.0;
    let shape_size = center * 0.5;

    let shape = hash[10] % 3;
    let shape_svg = match shape {
        0 => format!(
            r#"<circle cx="{center}" cy="{center}" r="{shape_size:.1}" fill="{}" opacity="0.7"/>"#,
            hex_color(shape_color)
        ),
        1 => {
            let top = center - shape_size;
            let right = center + shape_size;
            let bottom = center + shape_size;
            let left = center - shape_size;
            let fill = hex_color(shape_color);
            format!(
                r#"<polygon points="{center},{top} {right},{center} {center},{bottom} {left},{center}" fill="{fill}" opacity="0.7"/>"#,
            )
        }
        _ => {
            // Hexagon points
            let mut points = String::new();
            for i in 0..6 {
                let a = std::f64::consts::PI / 3.0 * i as f64 - std::f64::consts::PI / 6.0;
                let px = center + shape_size * a.cos();
                let py = center + shape_size * a.sin();
                if !points.is_empty() {
                    points.push(' ');
                }
                points.push_str(&format!("{px:.1},{py:.1}"));
            }
            format!(
                r#"<polygon points="{points}" fill="{}" opacity="0.7"/>"#,
                hex_color(shape_color)
            )
        }
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><defs><linearGradient id="g" gradientTransform="rotate({angle:.0} 0.5 0.5)" gradientUnits="objectBoundingBox"><stop offset="0%" stop-color="{c1}"/><stop offset="100%" stop-color="{c2}"/></linearGradient></defs><rect width="{size}" height="{size}" fill="url(#g)"/>{shape_svg}</svg>"#,
        c1 = hex_color(color1),
        c2 = hex_color(color2)
    )
}

fn svg_initials(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| color_from_hash(&hash, 0));
    let brightness = (bg.0 as u16 + bg.1 as u16 + bg.2 as u16) / 3;
    let letter_color = if brightness > 140 {
        (40, 40, 40)
    } else {
        (240, 240, 240)
    };

    let initials = extract_initials(seed);
    let text: String = initials
        .iter()
        .map(|&idx| {
            if idx < 26 {
                (b'A' + idx as u8) as char
            } else {
                (b'0' + (idx - 26) as u8) as char
            }
        })
        .collect();

    // SVG uses actual text element — much cleaner than bitmap
    let font_size = if text.len() == 1 {
        (size as f64 * 0.6) as u32
    } else {
        (size as f64 * 0.45) as u32
    };
    let center = size as f64 / 2.0;

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}" rx="{rx}"/><text x="{cx}" y="{cy}" text-anchor="middle" dominant-baseline="central" font-family="system-ui, -apple-system, sans-serif" font-weight="700" font-size="{font_size}" fill="{fg}">{text}</text></svg>"#,
        bg = hex_color(bg),
        rx = size / 8,
        cx = center,
        cy = center,
        fg = hex_color(letter_color),
    )
}

fn svg_starburst(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let color3 = color_from_hash(&hash, 6);
    let dot_color = color_from_hash(&hash, 15);

    let center = size as f64 / 2.0;
    let num_rays = 8 + (hash[12] % 13) as u32;
    let rotation = (hash[13] as f64 / 255.0) * 360.0;
    let max_radius = center * 0.9;
    let dot_radius = size as f64 * 0.08;

    let colors = [color1, color2, color3];
    let ray_angle = 360.0 / num_rays as f64;

    let mut paths = String::new();
    for i in 0..num_rays {
        if i % 2 != 0 {
            continue;
        }
        let color = colors[(i as usize / 2) % colors.len()];
        let a1 = rotation + i as f64 * ray_angle;
        let a2 = rotation + (i + 1) as f64 * ray_angle;
        let a1_rad = a1 * std::f64::consts::PI / 180.0;
        let a2_rad = a2 * std::f64::consts::PI / 180.0;

        let x1 = center + max_radius * a1_rad.cos();
        let y1 = center + max_radius * a1_rad.sin();
        let x2 = center + max_radius * a2_rad.cos();
        let y2 = center + max_radius * a2_rad.sin();

        // Use large_arc_flag = 0 since each ray is < 180°
        let large_arc = if ray_angle > 180.0 { 1 } else { 0 };

        paths.push_str(&format!(
            r#"<path d="M{cx:.1},{cy:.1} L{x1:.1},{y1:.1} A{r:.1},{r:.1} 0 {la} 1 {x2:.1},{y2:.1} Z" fill="{fill}" opacity="0.8"/>"#,
            cx = center,
            cy = center,
            r = max_radius,
            la = large_arc,
            fill = hex_color(color),
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}"/>{paths}<circle cx="{cx}" cy="{cy}" r="{dr:.1}" fill="{dc}"/></svg>"#,
        bg = hex_color(bg),
        cx = center,
        cy = center,
        dr = dot_radius,
        dc = hex_color(dot_color),
    )
}

fn svg_mosaic(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let palette = harmonious_palette(&hash);
    let bg = bg_override.unwrap_or_else(|| {
        let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
        hsl_to_rgb(base_hue, 0.15, 0.92)
    });

    let grid = 6u32;
    let cell = size / grid;
    let margin = (cell as f64 * 0.12) as u32;

    let mut shapes = String::new();

    for row in 0..grid {
        for col in 0..grid {
            let idx = (row * grid + col) as usize;
            let h = hash[(idx * 3) % 32];
            let shape = hash[(idx * 3 + 1) % 32];
            let color = palette[(h as usize) % palette.len()];
            let hex = hex_color(color);

            let x0 = col * cell + margin;
            let y0 = row * cell + margin;
            let w = cell - margin * 2;
            let h_dim = cell - margin * 2;
            let cx = x0 + w / 2;
            let cy = y0 + h_dim / 2;
            let r = w.min(h_dim) / 2;

            match shape % 5 {
                0 => {
                    // Square
                    shapes.push_str(&format!(
                        r#"<rect x="{x0}" y="{y0}" width="{w}" height="{h_dim}" rx="2" fill="{hex}"/>"#,
                    ));
                }
                1 => {
                    // Circle
                    shapes.push_str(&format!(
                        r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{hex}"/>"#,
                    ));
                }
                2 => {
                    // Diamond
                    shapes.push_str(&format!(
                        r#"<polygon points="{cx},{y0} {},{cy} {cx},{} {x0},{cy}" fill="{hex}"/>"#,
                        x0 + w, y0 + h_dim,
                    ));
                }
                3 => {
                    // Half-filled (top)
                    shapes.push_str(&format!(
                        r#"<rect x="{x0}" y="{y0}" width="{w}" height="{}" rx="2" fill="{hex}"/>"#,
                        h_dim / 2,
                    ));
                }
                _ => {
                    // Quarter circle
                    shapes.push_str(&format!(
                        r#"<path d="M{cx},{cy} L{},{cy} A{r},{r} 0 0 1 {cx},{}" Z" fill="{hex}"/>"#,
                        cx + r, cy + r,
                    ));
                }
            }
        }
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}"/>{shapes}</svg>"#,
        bg = hex_color(bg),
    )
}

fn svg_pixel(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let color1 = color_from_hash(&hash, 0);
    let color2 = color_from_hash(&hash, 3);
    let color3 = color_from_hash(&hash, 6);
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));

    let grid = 11u32;
    let half = grid / 2 + 1;
    let cell = size / (grid + 2);
    let offset = (size - cell * grid) / 2;
    let gap = (cell / 10).clamp(1, 3);
    let pixel_size = cell - gap * 2;

    let colors = [color1, color2, color3];
    let mut rects = String::new();

    for row in 0..grid {
        for col in 0..half {
            let idx = (row * half + col) as usize;
            let byte = hash[idx % 32];

            let col_dist = (half as i32 - 1 - col as i32).unsigned_abs();
            let row_center = grid / 2;
            let row_dist = row.abs_diff(row_center);

            let threshold = 80 + col_dist * 14 + row_dist * 8;

            if (byte as u32) > threshold {
                let color_byte = hash[(idx * 3 + 11) % 32];
                let color = colors[(color_byte as usize) % 3];
                let hex = hex_color(color);

                let x = offset + col * cell + gap;
                let y = offset + row * cell + gap;
                rects.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{pixel_size}" height="{pixel_size}" fill="{hex}"/>"#,
                ));

                // Mirror
                if col < grid / 2 {
                    let mx = offset + (grid - 1 - col) * cell + gap;
                    rects.push_str(&format!(
                        r#"<rect x="{mx}" y="{y}" width="{pixel_size}" height="{pixel_size}" fill="{hex}"/>"#,
                    ));
                }
            }
        }
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}"><rect width="{size}" height="{size}" fill="{bg}"/>{rects}</svg>"#,
        bg = hex_color(bg),
    )
}

fn svg_sunset(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>) -> String {
    let hash = hash_seed(seed);
    let palette = harmonious_palette(&hash);
    let bg = bg_override.unwrap_or_else(|| {
        let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
        hsl_to_rgb(base_hue, 0.6, 0.25)
    });

    let s = size as f64;
    let num_bands = 4 + (hash[5] % 3) as usize;

    let mut band_colors: Vec<(u8, u8, u8)> = Vec::new();
    for i in 0..num_bands {
        let base = palette[i % palette.len()];
        let factor = 0.7 + (i as f64 / num_bands as f64) * 0.6;
        let r = ((base.0 as f64 * factor).min(255.0)) as u8;
        let g = ((base.1 as f64 * factor).min(255.0)) as u8;
        let b = ((base.2 as f64 * factor).min(255.0)) as u8;
        band_colors.push((r, g, b));
    }

    let band_height = s / num_bands as f64;
    let mut elements = String::new();

    // Build SVG with gradient-filled bands using linearGradients + wavy clip paths
    let mut defs = String::from("<defs>");

    for band_idx in 0..num_bands {
        let color = band_colors[band_idx];
        let next_color = if band_idx + 1 < num_bands {
            band_colors[band_idx + 1]
        } else {
            color
        };

        // Gradient for this band (vertical blend at boundary)
        let grad_id = format!("g{band_idx}");
        defs.push_str(&format!(
            r#"<linearGradient id="{grad_id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="{}"/><stop offset="70%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></linearGradient>"#,
            hex_color(color), hex_color(color), hex_color(next_color),
        ));

        let y = (band_idx as f64 * band_height) as i32;
        let h = (band_height as i32) + 2; // +2 to avoid gaps

        // Wavy top edge via clip path
        let wave_amp = s * 0.02 + (hash[(band_idx * 4 + 10) % 32] as f64 / 255.0) * s * 0.03;
        let wave_freq = 2.0 + (hash[(band_idx * 4 + 11) % 32] as f64 / 255.0) * 3.0;
        let wave_phase = (hash[(band_idx * 4 + 12) % 32] as f64 / 255.0) * std::f64::consts::PI * 2.0;

        // Build a wavy path for the top edge
        let mut path = format!("M0,{}", y + h);
        let steps = 20;
        for step in 0..=steps {
            let x = (step as f64 / steps as f64) * s;
            let wave = (x / s * wave_freq * std::f64::consts::PI + wave_phase).sin() * wave_amp;
            let py = y as f64 + wave;
            path.push_str(&format!(" L{:.1},{:.1}", x, py));
        }
        path.push_str(&format!(" L{},{} Z", size, y + h));

        let clip_id = format!("c{band_idx}");
        defs.push_str(&format!(
            r#"<clipPath id="{clip_id}"><path d="{path}"/></clipPath>"#,
        ));

        elements.push_str(&format!(
            r#"<rect x="0" y="{y}" width="{size}" height="{h}" fill="url(#{grad_id})" clip-path="url(#{clip_id})"/>"#,
        ));
    }

    // Sun/moon
    let sun_present = hash[7] % 3 != 0;
    if sun_present {
        let sun_x = s * 0.2 + (hash[8] as f64 / 255.0) * s * 0.6;
        let sun_y = s * 0.15 + (hash[9] as f64 / 255.0) * s * 0.3;
        let sun_r = s * 0.06 + (hash[10] as f64 / 255.0) * s * 0.06;

        let sun_base = palette[0];
        let sun_color = (
            ((sun_base.0 as u16 + 255) / 2) as u8,
            ((sun_base.1 as u16 + 200) / 2) as u8,
            ((sun_base.2 as u16 + 150) / 2) as u8,
        );

        // Radial gradient for glow
        let glow_r = sun_r * 1.8;
        defs.push_str(&format!(
            r#"<radialGradient id="sun"><stop offset="0%" stop-color="{}" stop-opacity="1"/><stop offset="55%" stop-color="{}" stop-opacity="0.8"/><stop offset="100%" stop-color="{}" stop-opacity="0"/></radialGradient>"#,
            hex_color(sun_color), hex_color(sun_color), hex_color(sun_color),
        ));

        elements.push_str(&format!(
            r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="url(#sun)"/>"#,
            sun_x, sun_y, glow_r,
        ));
    }

    defs.push_str("</defs>");

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">{defs}<rect width="{size}" height="{size}" fill="{bg}"/>{elements}</svg>"#,
        bg = hex_color(bg),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let img1 = generate_png("test-seed", "geometric", 256, None).unwrap();
        let img2 = generate_png("test-seed", "geometric", 256, None).unwrap();
        assert_eq!(img1, img2, "Same seed must produce identical output");
    }

    #[test]
    fn test_different_seeds_differ() {
        let img1 = generate_png("seed-a", "geometric", 256, None).unwrap();
        let img2 = generate_png("seed-b", "geometric", 256, None).unwrap();
        assert_ne!(img1, img2, "Different seeds should produce different output");
    }

    #[test]
    fn test_all_styles_png() {
        for style in &["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"] {
            let result = generate_png("test", style, 128, None);
            assert!(result.is_ok(), "Style {style} should produce valid PNG");
            assert!(!result.unwrap().is_empty(), "Style {style} PNG should not be empty");
        }
    }

    #[test]
    fn test_all_styles_svg() {
        for style in &["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"] {
            let result = generate_svg("test", style, 128, None);
            assert!(result.is_ok(), "Style {style} should produce valid SVG");
            let svg = result.unwrap();
            assert!(svg.starts_with("<svg"), "Style {style} SVG should start with <svg");
            assert!(svg.contains("</svg>"), "Style {style} SVG should end with </svg>");
        }
    }

    #[test]
    fn test_unknown_style() {
        assert!(generate_png("test", "unknown", 128, None).is_err());
        assert!(generate_svg("test", "unknown", 128, None).is_err());
    }

    #[test]
    fn test_bg_override() {
        let with_override = generate_png("test", "geometric", 128, Some((255, 0, 0))).unwrap();
        let without_override = generate_png("test", "geometric", 128, None).unwrap();
        assert_ne!(with_override, without_override, "Background override should change output");
    }

    #[test]
    fn test_various_sizes() {
        for size in &[16, 64, 128, 256, 512] {
            let result = generate_png("test", "geometric", *size, None);
            assert!(result.is_ok(), "Size {size} should work");
        }
    }

    #[test]
    fn test_hash_determinism() {
        let h1 = hash_seed("hello");
        let h2 = hash_seed("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let h1 = hash_seed("hello");
        let h2 = hash_seed("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_color_range() {
        let hash = hash_seed("test");
        let (r, g, b) = color_from_hash(&hash, 0);
        assert!(r >= 40 && r <= 220);
        assert!(g >= 40 && g <= 220);
        assert!(b >= 40 && b <= 220);
    }

    #[test]
    fn test_bg_color_pastel() {
        let hash = hash_seed("test");
        let (r, g, b) = bg_color_from_hash(&hash);
        assert!(r >= 175);
        assert!(g >= 175);
        assert!(b >= 175);
    }

    #[test]
    fn test_svg_deterministic() {
        let svg1 = generate_svg("same-seed", "geometric", 256, None).unwrap();
        let svg2 = generate_svg("same-seed", "geometric", 256, None).unwrap();
        assert_eq!(svg1, svg2);
    }

    #[test]
    fn test_empty_seed() {
        // Empty seed should still produce valid output
        let result = generate_png("", "geometric", 128, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_seed() {
        let result = generate_png("日本語テスト🤖", "geometric", 128, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_seed() {
        let seed = "a".repeat(10000);
        let result = generate_png(&seed, "geometric", 128, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_png_valid_header() {
        let png = generate_png("test", "geometric", 128, None).unwrap();
        // PNG magic bytes
        assert_eq!(&png[..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    // ── Initials style tests ──

    #[test]
    fn test_initials_deterministic() {
        let img1 = generate_png("Nanook", "initials", 256, None).unwrap();
        let img2 = generate_png("Nanook", "initials", 256, None).unwrap();
        assert_eq!(img1, img2);
    }

    #[test]
    fn test_initials_different_seeds() {
        let img1 = generate_png("Alice", "initials", 128, None).unwrap();
        let img2 = generate_png("Bob", "initials", 128, None).unwrap();
        assert_ne!(img1, img2);
    }

    #[test]
    fn test_initials_extract_letters() {
        let initials = extract_initials("Nanook");
        assert_eq!(initials.len(), 2);
        assert_eq!(initials[0], b'N' as usize - b'A' as usize); // N
        assert_eq!(initials[1], b'a' as usize - b'a' as usize); // a
    }

    #[test]
    fn test_initials_extract_numbers() {
        let initials = extract_initials("42agent");
        assert_eq!(initials.len(), 2);
        assert_eq!(initials[0], 26 + 4); // '4' → index 30
        assert_eq!(initials[1], 26 + 2); // '2' → index 28
    }

    #[test]
    fn test_initials_extract_empty() {
        // Non-alphanumeric seed should fall back to hash-derived character
        let initials = extract_initials("🤖");
        assert_eq!(initials.len(), 1);
        assert!(initials[0] < 26); // Should be A-Z range
    }

    #[test]
    fn test_initials_svg_contains_text() {
        let svg = generate_svg("Nanook", "initials", 256, None).unwrap();
        assert!(svg.contains("<text"), "SVG should contain text element");
        // Both 'N' and 'a' map to uppercase in SVG (font index → letter)
        assert!(svg.contains("NA"), "SVG should contain initials 'NA'");
    }

    #[test]
    fn test_initials_single_char() {
        let initials = extract_initials("X");
        assert_eq!(initials.len(), 1);
        assert_eq!(initials[0], b'X' as usize - b'A' as usize);
    }

    #[test]
    fn test_initials_bg_override() {
        let with = generate_png("Test", "initials", 128, Some((0, 0, 0))).unwrap();
        let without = generate_png("Test", "initials", 128, None).unwrap();
        assert_ne!(with, without);
    }

    #[test]
    fn test_initials_small_size() {
        let result = generate_png("AB", "initials", 16, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_initials_large_size() {
        let result = generate_png("AB", "initials", 512, None);
        assert!(result.is_ok());
    }

    // ── Starburst style tests ──

    #[test]
    fn test_starburst_deterministic() {
        let img1 = generate_png("star", "starburst", 256, None).unwrap();
        let img2 = generate_png("star", "starburst", 256, None).unwrap();
        assert_eq!(img1, img2);
    }

    #[test]
    fn test_starburst_different_seeds() {
        let img1 = generate_png("sun", "starburst", 128, None).unwrap();
        let img2 = generate_png("moon", "starburst", 128, None).unwrap();
        assert_ne!(img1, img2);
    }

    #[test]
    fn test_starburst_svg_has_paths() {
        let svg = generate_svg("star", "starburst", 256, None).unwrap();
        assert!(svg.contains("<path"), "SVG should contain ray paths");
        assert!(svg.contains("<circle"), "SVG should contain center dot");
    }

    #[test]
    fn test_starburst_bg_override() {
        let with = generate_png("star", "starburst", 128, Some((255, 0, 0))).unwrap();
        let without = generate_png("star", "starburst", 128, None).unwrap();
        assert_ne!(with, without);
    }

    #[test]
    fn test_starburst_small_size() {
        let result = generate_png("star", "starburst", 16, None);
        assert!(result.is_ok());
    }

    // ── Pixel art style tests ──

    #[test]
    fn test_pixel_deterministic() {
        let img1 = generate_png("creature", "pixel", 256, None).unwrap();
        let img2 = generate_png("creature", "pixel", 256, None).unwrap();
        assert_eq!(img1, img2);
    }

    #[test]
    fn test_pixel_different_seeds() {
        let img1 = generate_png("invader-a", "pixel", 128, None).unwrap();
        let img2 = generate_png("invader-b", "pixel", 128, None).unwrap();
        assert_ne!(img1, img2);
    }

    #[test]
    fn test_pixel_svg_has_rects() {
        let svg = generate_svg("creature", "pixel", 256, None).unwrap();
        assert!(svg.contains("<rect"), "Pixel SVG should contain rect elements");
        assert!(svg.contains("<svg"), "Should be valid SVG");
    }

    #[test]
    fn test_pixel_bg_override() {
        let with = generate_png("test", "pixel", 128, Some((255, 0, 0))).unwrap();
        let without = generate_png("test", "pixel", 128, None).unwrap();
        assert_ne!(with, without);
    }

    #[test]
    fn test_pixel_small_size() {
        let result = generate_png("tiny", "pixel", 16, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pixel_large_size() {
        let result = generate_png("big", "pixel", 512, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pixel_horizontal_symmetry() {
        // Verify the pixel art has horizontal symmetry by checking the generated image
        let img = generate_pixel("symmetry-test", 256, None);
        let grid = 11u32;
        let cell = 256 / (grid + 2);
        let offset = (256 - cell * grid) / 2;
        let mid = offset + cell * grid / 2; // center x

        // Sample several rows at cell centers and check left-right mirror
        for row in 0..grid {
            let y = offset + row * cell + cell / 2;
            if y >= 256 { continue; }
            for col in 0..(grid / 2) {
                let lx = offset + col * cell + cell / 2;
                let rx = offset + (grid - 1 - col) * cell + cell / 2;
                if lx >= 256 || rx >= 256 { continue; }
                let left_pixel = img.get_pixel(lx, y);
                let right_pixel = img.get_pixel(rx, y);
                assert_eq!(
                    left_pixel, right_pixel,
                    "Pixel at ({lx},{y}) should mirror ({rx},{y})"
                );
            }
        }
    }

    #[test]
    fn test_pixel_svg_deterministic() {
        let svg1 = generate_svg("pixel-test", "pixel", 256, None).unwrap();
        let svg2 = generate_svg("pixel-test", "pixel", 256, None).unwrap();
        assert_eq!(svg1, svg2);
    }

    #[test]
    fn test_pixel_unicode_seed() {
        let result = generate_png("🎮👾", "pixel", 128, None);
        assert!(result.is_ok());
    }

    // ── Bitmap font tests ──

    #[test]
    fn test_font_all_chars_nonzero() {
        let font = bitmap_font();
        for (i, &bits) in font.iter().enumerate() {
            assert!(bits > 0, "Font char at index {i} should have non-zero bits");
        }
    }

    #[test]
    fn test_char_to_font_idx_coverage() {
        // Uppercase
        assert_eq!(char_to_font_idx('A'), Some(0));
        assert_eq!(char_to_font_idx('Z'), Some(25));
        // Lowercase
        assert_eq!(char_to_font_idx('a'), Some(0));
        assert_eq!(char_to_font_idx('z'), Some(25));
        // Digits
        assert_eq!(char_to_font_idx('0'), Some(26));
        assert_eq!(char_to_font_idx('9'), Some(35));
        // Invalid
        assert_eq!(char_to_font_idx('!'), None);
        assert_eq!(char_to_font_idx(' '), None);
    }

    // ── Sunset style tests ──

    #[test]
    fn test_sunset_deterministic() {
        let img1 = generate_png("horizon", "sunset", 256, None).unwrap();
        let img2 = generate_png("horizon", "sunset", 256, None).unwrap();
        assert_eq!(img1, img2);
    }

    #[test]
    fn test_sunset_different_seeds() {
        let img1 = generate_png("dawn", "sunset", 128, None).unwrap();
        let img2 = generate_png("dusk", "sunset", 128, None).unwrap();
        assert_ne!(img1, img2);
    }

    #[test]
    fn test_sunset_svg_has_gradients() {
        let svg = generate_svg("horizon", "sunset", 256, None).unwrap();
        assert!(svg.contains("<linearGradient"), "Sunset SVG should contain gradient defs");
        assert!(svg.contains("<clipPath"), "Sunset SVG should contain clip paths for wavy edges");
    }

    #[test]
    fn test_sunset_svg_deterministic() {
        let svg1 = generate_svg("sunset-test", "sunset", 256, None).unwrap();
        let svg2 = generate_svg("sunset-test", "sunset", 256, None).unwrap();
        assert_eq!(svg1, svg2);
    }

    #[test]
    fn test_sunset_bg_override() {
        let with = generate_png("test", "sunset", 128, Some((0, 0, 50))).unwrap();
        let without = generate_png("test", "sunset", 128, None).unwrap();
        assert_ne!(with, without);
    }

    #[test]
    fn test_sunset_small_size() {
        let result = generate_png("small", "sunset", 16, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sunset_large_size() {
        let result = generate_png("big", "sunset", 512, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sunset_uses_harmony() {
        // The sunset style should use harmonious_palette, producing distinct
        // but cohesive colors. Verify it produces valid output for many seeds.
        for i in 0..20 {
            let seed = format!("sunset-harmony-{i}");
            let result = generate_png(&seed, "sunset", 128, None);
            assert!(result.is_ok(), "Sunset should work for seed: {seed}");
        }
    }

    #[test]
    fn test_sunset_unicode_seed() {
        let result = generate_png("🌅🌄", "sunset", 128, None);
        assert!(result.is_ok());
    }
}
