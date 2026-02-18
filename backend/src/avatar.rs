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

    // Eyes
    let eye_size = (s * 0.08) as u32;
    let eye_y = (s * 0.38) as u32;
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
    let bg = bg_override.unwrap_or_else(|| bg_color_from_hash(&hash));
    let s = size as f64;

    let mut parts = String::new();

    // Head
    let hl = (s * 0.2) as u32;
    let ht = (s * 0.2) as u32;
    let hw = (s * 0.6) as u32;
    let hh = (s * 0.5) as u32;
    parts.push_str(&format!(
        r#"<rect x="{hl}" y="{ht}" width="{hw}" height="{hh}" rx="4" fill="{}"/>"#,
        hex_color(body_color)
    ));

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

    // Eyes
    let ey = (s * 0.38) as u32;
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
        for style in &["geometric", "rings", "robot", "blockies", "gradient"] {
            let result = generate_png("test", style, 128, None);
            assert!(result.is_ok(), "Style {style} should produce valid PNG");
            assert!(!result.unwrap().is_empty(), "Style {style} PNG should not be empty");
        }
    }

    #[test]
    fn test_all_styles_svg() {
        for style in &["geometric", "rings", "robot", "blockies", "gradient"] {
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
}
