use crate::avatar::{self, hash_seed};
use image::{ImageBuffer, Rgba, RgbaImage};

/// Default animation parameters.
pub const DEFAULT_FRAMES: u16 = 12;
pub const DEFAULT_DELAY: u16 = 10; // centiseconds (100ms)
pub const MAX_FRAMES: u16 = 30;
pub const MAX_GIF_SIZE: u32 = 512;

/// Generate animated GIF bytes for a given style.
/// `delay` is in centiseconds (1/100th of a second).
pub fn generate_gif(
    seed: &str,
    style: &str,
    size: u32,
    bg: Option<(u8, u8, u8)>,
    frames: u16,
    delay: u16,
) -> Result<Vec<u8>, String> {
    let size = size.min(MAX_GIF_SIZE);
    let frames = frames.clamp(2, MAX_FRAMES);

    // Generate all animation frames
    let images = generate_frames(seed, style, size, bg, frames)?;

    // Encode as GIF
    encode_gif(&images, size, delay)
}

/// Generate N animation frames for a style.
fn generate_frames(
    seed: &str,
    style: &str,
    size: u32,
    bg: Option<(u8, u8, u8)>,
    frame_count: u16,
) -> Result<Vec<RgbaImage>, String> {
    let mut frames = Vec::with_capacity(frame_count as usize);

    for i in 0..frame_count {
        let t = i as f64 / frame_count as f64; // 0.0 to ~1.0
        let img = generate_animated_frame(seed, style, size, bg, t)?;
        frames.push(img);
    }

    Ok(frames)
}

/// Generate a single animation frame at time t (0.0 to 1.0).
fn generate_animated_frame(
    seed: &str,
    style: &str,
    size: u32,
    bg: Option<(u8, u8, u8)>,
    t: f64,
) -> Result<RgbaImage, String> {
    match style {
        "rings" => Ok(animate_rings(seed, size, bg, t)),
        "robot" => Ok(animate_robot(seed, size, bg, t)),
        "starburst" => Ok(animate_starburst(seed, size, bg, t)),
        "gradient" => Ok(animate_gradient(seed, size, bg, t)),
        "pixel" => Ok(animate_pixel(seed, size, bg, t)),
        "sunset" => Ok(animate_sunset(seed, size, bg, t)),
        // Styles without custom animation: use pulse effect
        _ => Ok(animate_pulse(seed, style, size, bg, t)),
    }
}

// ── Per-Style Animations ──

/// Rings: pulsating ring radii — rings breathe in and out.
fn animate_rings(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| avatar::bg_color_from_hash(&hash));

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let cx = size / 2;
    let cy = size / 2;
    let num_rings = 3 + (hash[10] % 4) as u32; // 3-6 rings
    let max_radius = size / 2 - 2;

    // Oscillation: each ring breathes with a phase offset
    let wave = (t * std::f64::consts::TAU).sin();

    for ring in 0..num_rings {
        let color = avatar::color_from_hash(&hash, (ring as usize) * 3);
        let base_radius = max_radius * (num_rings - ring) / num_rings;

        // Each ring has a different phase offset
        let phase_offset = ring as f64 * 0.4;
        let ring_wave = ((t * std::f64::consts::TAU + phase_offset).sin()) * 0.08;
        let radius = ((base_radius as f64) * (1.0 + ring_wave)) as u32;
        let thickness = (size / (num_rings * 3)).max(3);

        draw_ring(&mut img, cx, cy, radius, thickness, color);
    }

    // Center dot with pulse
    let dot_color = avatar::color_from_hash(&hash, 15);
    let base_dot_r = (size / 12).max(3);
    let dot_r = ((base_dot_r as f64) * (1.0 + wave * 0.15)) as u32;
    fill_circle(&mut img, cx, cy, dot_r, dot_color);

    img
}

/// Robot: eye blink animation — eyes close briefly.
fn animate_robot(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    // Generate base robot
    let mut img = avatar::generate_robot(seed, size, bg_override);
    let hash = hash_seed(seed);

    // Determine blink timing: eyes closed for ~15% of the cycle
    let blink_phase = (t * 2.0) % 1.0; // Two blinks per loop
    let is_blinking = blink_phase > 0.85;

    if is_blinking {
        // Draw closed eyes (horizontal lines) over the open eyes
        let head_y = size / 6;
        let head_h = size * 2 / 5;
        let eye_y = head_y + head_h / 3;
        let eye_size = size / 10;
        let eye_spacing = size / 5;
        let cx = size / 2;

        let eye_color = avatar::color_from_hash(&hash, 6);

        // Left eye - horizontal line
        let left_cx = cx - eye_spacing;
        let right_cx = cx + eye_spacing;

        let blink_w = eye_size + 2;
        let blink_h = 2.max(size / 64);

        // Fill eye areas with head color first (erase open eyes)
        let head_color = avatar::color_from_hash(&hash, 0);
        fill_rect_safe(&mut img, left_cx.saturating_sub(eye_size + 2), eye_y.saturating_sub(eye_size + 2),
                   left_cx + eye_size + 3, eye_y + eye_size + 3, head_color);
        fill_rect_safe(&mut img, right_cx.saturating_sub(eye_size + 2), eye_y.saturating_sub(eye_size + 2),
                   right_cx + eye_size + 3, eye_y + eye_size + 3, head_color);

        // Draw blink lines
        fill_rect_safe(&mut img, left_cx.saturating_sub(blink_w), eye_y, left_cx + blink_w, eye_y + blink_h, eye_color);
        fill_rect_safe(&mut img, right_cx.saturating_sub(blink_w), eye_y, right_cx + blink_w, eye_y + blink_h, eye_color);
    }

    img
}

/// Starburst: rotation animation — rays spin slowly.
fn animate_starburst(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let hash = hash_seed(seed);
    let bg = bg_override.unwrap_or_else(|| avatar::bg_color_from_hash(&hash));

    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    let cx = size as f64 / 2.0;
    let cy = size as f64 / 2.0;
    let max_r = size as f64 / 2.0 - 2.0;

    let num_rays = 8 + (hash[5] % 13) as usize; // 8-20 rays
    let colors = [
        avatar::color_from_hash(&hash, 0),
        avatar::color_from_hash(&hash, 3),
        avatar::color_from_hash(&hash, 6),
    ];

    // Rotation offset based on time
    let rotation = t * std::f64::consts::TAU;

    for i in 0..num_rays {
        let base_angle = (i as f64 / num_rays as f64) * std::f64::consts::TAU;
        let angle = base_angle + rotation + (hash[9] as f64 / 255.0) * std::f64::consts::TAU;
        let color = colors[i % 3];
        let ray_width = 0.15 + (hash[(i * 2 + 12) % 32] as f64 / 255.0) * 0.15;

        // Draw ray with fading
        for r_step in 0..(max_r as u32) {
            let r = r_step as f64;
            let fade = 1.0 - (r / max_r).powf(1.5);
            if fade <= 0.0 { break; }

            let half_w = ray_width * r;
            for w in -(half_w as i32)..=(half_w as i32) {
                let perp_angle = angle + std::f64::consts::FRAC_PI_2;
                let px = cx + r * angle.cos() + w as f64 * perp_angle.cos();
                let py = cy + r * angle.sin() + w as f64 * perp_angle.sin();
                let ix = px as u32;
                let iy = py as u32;
                if ix < size && iy < size {
                    let alpha = (fade * 255.0) as u8;
                    blend_pixel(&mut img, ix, iy, color, alpha);
                }
            }
        }
    }

    // Center dot
    let dot_color = avatar::color_from_hash(&hash, 9);
    let dot_r = (size / 15).max(3);
    fill_circle(&mut img, size / 2, size / 2, dot_r, dot_color);

    img
}

/// Gradient: angle rotation — gradient sweeps around.
fn animate_gradient(seed: &str, size: u32, _bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let hash = hash_seed(seed);
    let c1 = avatar::color_from_hash(&hash, 0);
    let c2 = avatar::color_from_hash(&hash, 3);

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Animate gradient angle
    let base_angle = (hash[6] as f64 / 255.0) * std::f64::consts::TAU;
    let angle = base_angle + t * std::f64::consts::TAU;
    let (dx, dy) = (angle.cos(), angle.sin());

    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 / size as f64 - 0.5;
            let ny = y as f64 / size as f64 - 0.5;
            let proj = (nx * dx + ny * dy + 0.5).clamp(0.0, 1.0);
            let r = lerp_u8(c1.0, c2.0, proj);
            let g = lerp_u8(c1.1, c2.1, proj);
            let b = lerp_u8(c1.2, c2.2, proj);
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Overlay shape from base (deterministic)
    let shape_type = hash[12] % 4;
    let shape_color = avatar::color_from_hash(&hash, 9);
    let cx = size / 2;
    let cy = size / 2;
    let shape_r = size / 4;

    match shape_type {
        0 => fill_circle(&mut img, cx, cy, shape_r, shape_color),
        1 => {
            // Diamond
            for dy_i in 0..shape_r {
                let w = shape_r - dy_i;
                fill_rect_safe(&mut img, cx.saturating_sub(w), cy.saturating_sub(shape_r) + dy_i,
                          cx + w + 1, cy.saturating_sub(shape_r) + dy_i + 1, shape_color);
                fill_rect_safe(&mut img, cx.saturating_sub(w), cy + dy_i,
                          cx + w + 1, cy + dy_i + 1, shape_color);
            }
        }
        2 => {
            // Square
            fill_rect_safe(&mut img, cx.saturating_sub(shape_r), cy.saturating_sub(shape_r),
                      cx + shape_r, cy + shape_r, shape_color);
        }
        _ => {
            // Triangle
            for row in 0..(shape_r * 2) {
                let w = row / 2;
                let y = cy.saturating_sub(shape_r) + row;
                fill_rect_safe(&mut img, cx.saturating_sub(w), y, cx + w + 1, y + 1, shape_color);
            }
        }
    }

    img
}

/// Pixel: color cycling — palette shifts through hues.
fn animate_pixel(seed: &str, size: u32, bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let hash = hash_seed(seed);
    let grid = 11u32;
    let cell = size / grid;
    let gap = (cell as f64 * 0.15).max(1.0) as u32;

    // Hue-shift the colors based on time
    let hue_shift = (t * 360.0) as i32;

    let bg = bg_override.unwrap_or_else(|| shift_hue(avatar::bg_color_from_hash(&hash), hue_shift / 3));
    let mut img: RgbaImage = ImageBuffer::new(size, size);
    for pixel in img.pixels_mut() {
        *pixel = Rgba([bg.0, bg.1, bg.2, 255]);
    }

    // Generate pixel creature grid (same logic as static, but with color shift)
    let c1 = shift_hue(avatar::color_from_hash(&hash, 0), hue_shift);
    let c2 = shift_hue(avatar::color_from_hash(&hash, 3), hue_shift);
    let c3 = shift_hue(avatar::color_from_hash(&hash, 6), hue_shift);

    let half_w = grid.div_ceil(2); // 6 columns, mirrored

    for row in 0..grid {
        for col in 0..half_w {
            let idx = row * half_w + col;
            let byte = hash[(idx as usize) % 32];

            // Center-weighted fill probability
            let dist_from_center = ((col as f64 - (half_w as f64 / 2.0)).abs()
                + (row as f64 - (grid as f64 / 2.0)).abs()) / grid as f64;
            let threshold = (80.0 + dist_from_center * 120.0) as u8;

            if byte > threshold {
                let color = match byte % 3 {
                    0 => c1,
                    1 => c2,
                    _ => c3,
                };

                // Draw pixel and its mirror
                let x0 = col * cell + gap;
                let y0 = row * cell + gap;
                let x1 = (col + 1) * cell - gap;
                let y1 = (row + 1) * cell - gap;
                fill_rect_safe(&mut img, x0, y0, x1, y1, color);

                // Mirror
                let mirror_col = grid - 1 - col;
                let mx0 = mirror_col * cell + gap;
                let mx1 = (mirror_col + 1) * cell - gap;
                fill_rect_safe(&mut img, mx0, y0, mx1, y1, color);
            }
        }
    }

    img
}

/// Sunset: sun movement and color shift.
fn animate_sunset(seed: &str, size: u32, _bg_override: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let hash = hash_seed(seed);

    // Use harmonious palette with hue shift over time
    let hue_shift = (t * 60.0) as i32; // Subtle 60° shift over full cycle

    let base_hue = ((hash[0] as f64 * 256.0 + hash[1] as f64) / 65535.0) * 360.0;
    let saturation = 0.55 + (hash[2] as f64 / 255.0) * 0.3;
    let lightness = 0.45 + (hash[3] as f64 / 255.0) * 0.15;

    let num_bands = 4 + (hash[4] % 3) as u32; // 4-6 bands

    let mut img: RgbaImage = ImageBuffer::new(size, size);

    // Generate band colors with hue shift
    let band_colors: Vec<(u8, u8, u8)> = (0..num_bands)
        .map(|i| {
            let h = (base_hue + i as f64 * 40.0 + hue_shift as f64) % 360.0;
            let l = lightness - (i as f64 * 0.06) + 0.1;
            hsl_to_rgb(h, saturation, l.clamp(0.25, 0.75))
        })
        .collect();

    // Draw bands with wavy edges
    let band_h = size / num_bands;
    for band in 0..num_bands {
        let color = band_colors[band as usize];
        let y_start = band * band_h;
        let y_end = if band == num_bands - 1 { size } else { (band + 1) * band_h };

        for y in y_start..y_end {
            for x in 0..size {
                // Wavy edge offset
                let wave_amp = (hash[(band as usize * 2 + 5) % 32] as f64 / 255.0) * (band_h as f64 * 0.3);
                let wave = (x as f64 / size as f64 * std::f64::consts::TAU * 2.0).sin() * wave_amp;
                let adjusted_y = y as f64 - wave;

                if adjusted_y >= y_start as f64 && adjusted_y < y_end as f64
                    && x < size && y < size
                {
                    img.put_pixel(x, y, Rgba([color.0, color.1, color.2, 255]));
                }
            }
        }
    }

    // Sun with vertical oscillation
    let has_sun = !hash[7].is_multiple_of(3); // 2/3 chance
    if has_sun {
        let sun_x = size / 4 + (hash[8] as u32 * size / 2 / 255);
        let base_sun_y = size / 4 + (hash[9] as u32 * size / 6 / 255);
        let sun_oscillation = (t * std::f64::consts::TAU).sin() * (size as f64 * 0.05);
        let sun_y = (base_sun_y as f64 + sun_oscillation).max(0.0) as u32;
        let sun_r = size / 10 + (hash[10] as u32 * size / 30 / 255);

        // Glow
        let glow_color = hsl_to_rgb((base_hue + 30.0) % 360.0, 0.9, 0.85);
        for gy in sun_y.saturating_sub(sun_r * 2)..=(sun_y + sun_r * 2).min(size - 1) {
            for gx in sun_x.saturating_sub(sun_r * 2)..=(sun_x + sun_r * 2).min(size - 1) {
                let dx = gx as f64 - sun_x as f64;
                let dy = gy as f64 - sun_y as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                let glow_r = sun_r as f64 * 1.8;
                if dist < glow_r {
                    let alpha = ((1.0 - dist / glow_r) * 100.0) as u8;
                    if gx < size && gy < size {
                        blend_pixel(&mut img, gx, gy, glow_color, alpha);
                    }
                }
            }
        }

        // Sun circle
        let sun_color = hsl_to_rgb((base_hue + 20.0) % 360.0, 0.95, 0.9);
        fill_circle(&mut img, sun_x, sun_y, sun_r, sun_color);
    }

    img
}

/// Generic pulse animation for styles without custom animation.
/// Generates the static avatar and applies a subtle brightness pulse.
fn animate_pulse(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>, t: f64) -> RgbaImage {
    let mut img = avatar::generate_image(seed, style, size, bg)
        .unwrap_or_else(|_| ImageBuffer::new(size, size));

    // Subtle brightness oscillation
    let brightness = 1.0 + (t * std::f64::consts::TAU).sin() * 0.06; // ±6%

    for pixel in img.pixels_mut() {
        let Rgba([r, g, b, a]) = *pixel;
        let r = (r as f64 * brightness).clamp(0.0, 255.0) as u8;
        let g = (g as f64 * brightness).clamp(0.0, 255.0) as u8;
        let b = (b as f64 * brightness).clamp(0.0, 255.0) as u8;
        *pixel = Rgba([r, g, b, a]);
    }

    img
}

// ── GIF Encoding ──

/// Encode a sequence of RGBA images as an animated GIF.
/// Uses the actual image dimensions (which may differ from requested size due to grid snapping).
fn encode_gif(frames: &[RgbaImage], _size: u32, delay: u16) -> Result<Vec<u8>, String> {
    if frames.is_empty() {
        return Err("No frames to encode".to_string());
    }

    let width = frames[0].width() as u16;
    let height = frames[0].height() as u16;

    let mut buf = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut buf, width, height, &[])
            .map_err(|e| format!("GIF encoder init error: {e}"))?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| format!("GIF repeat error: {e}"))?;

        for img in frames {
            let w = img.width() as u16;
            let h = img.height() as u16;
            let mut rgba_data: Vec<u8> = img.as_raw().clone();

            let mut frame = gif::Frame::from_rgba_speed(
                w,
                h,
                &mut rgba_data,
                10, // quality/speed tradeoff (1=best quality, 30=fastest)
            );
            frame.delay = delay;
            frame.dispose = gif::DisposalMethod::Background;

            encoder
                .write_frame(&frame)
                .map_err(|e| format!("GIF frame write error: {e}"))?;
        }
    }
    Ok(buf)
}

// ── Helper Functions ──

fn fill_rect_safe(img: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32, color: (u8, u8, u8)) {
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

fn draw_ring(img: &mut RgbaImage, cx: u32, cy: u32, radius: u32, thickness: u32, color: (u8, u8, u8)) {
    let r_outer = radius as i64;
    let r_inner = (radius.saturating_sub(thickness)) as i64;
    let cx = cx as i64;
    let cy = cy as i64;
    for y in (cy - r_outer)..=(cy + r_outer) {
        for x in (cx - r_outer)..=(cx + r_outer) {
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                let dx = x - cx;
                let dy = y - cy;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_outer * r_outer && dist_sq >= r_inner * r_inner {
                    img.put_pixel(x as u32, y as u32, Rgba([color.0, color.1, color.2, 255]));
                }
            }
        }
    }
}

fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, color: (u8, u8, u8), alpha: u8) {
    if x >= img.width() || y >= img.height() {
        return;
    }
    let existing = img.get_pixel(x, y);
    let a = alpha as f64 / 255.0;
    let r = (color.0 as f64 * a + existing[0] as f64 * (1.0 - a)) as u8;
    let g = (color.1 as f64 * a + existing[1] as f64 * (1.0 - a)) as u8;
    let b = (color.2 as f64 * a + existing[2] as f64 * (1.0 - a)) as u8;
    img.put_pixel(x, y, Rgba([r, g, b, 255]));
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).clamp(0.0, 255.0) as u8
}

/// Shift the hue of an RGB color by `degrees`.
fn shift_hue(color: (u8, u8, u8), degrees: i32) -> (u8, u8, u8) {
    let (h, s, l) = rgb_to_hsl(color);
    let new_h = ((h as i32 + degrees) % 360 + 360) as f64 % 360.0;
    hsl_to_rgb(new_h, s, l)
}

fn rgb_to_hsl(color: (u8, u8, u8)) -> (f64, f64, f64) {
    let r = color.0 as f64 / 255.0;
    let g = color.1 as f64 / 255.0;
    let b = color.2 as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 1e-10 {
        let mut h = (g - b) / d;
        if g < b { h += 6.0; }
        h * 60.0
    } else if (max - g).abs() < 1e-10 {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };
    (h, s, l)
}

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

// ── Unit Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_deterministic() {
        let gif1 = generate_gif("test", "rings", 64, None, 4, 10).unwrap();
        let gif2 = generate_gif("test", "rings", 64, None, 4, 10).unwrap();
        assert_eq!(gif1, gif2, "Same seed should produce identical GIF");
    }

    #[test]
    fn test_gif_different_seeds() {
        let gif1 = generate_gif("alice", "robot", 64, None, 4, 10).unwrap();
        let gif2 = generate_gif("bob", "robot", 64, None, 4, 10).unwrap();
        assert_ne!(gif1, gif2, "Different seeds should produce different GIFs");
    }

    #[test]
    fn test_gif_valid_header() {
        let gif = generate_gif("test", "geometric", 64, None, 4, 10).unwrap();
        assert!(gif.starts_with(b"GIF89a"), "Should be a valid GIF89a file");
    }

    #[test]
    fn test_gif_all_styles() {
        let styles = ["geometric", "rings", "robot", "blockies", "gradient",
                      "initials", "starburst", "mosaic", "pixel", "sunset"];
        for style in &styles {
            let gif = generate_gif("test", style, 64, None, 4, 10).unwrap();
            assert!(gif.starts_with(b"GIF89a"), "Style {style} should produce valid GIF");
            assert!(gif.len() > 100, "Style {style} GIF should have content");
        }
    }

    #[test]
    fn test_gif_frame_count_bounds() {
        // Min frames (clamped to 2)
        let gif = generate_gif("test", "rings", 64, None, 1, 10).unwrap();
        assert!(gif.starts_with(b"GIF89a"));

        // Max frames (clamped to 30)
        let gif = generate_gif("test", "rings", 64, None, 50, 10).unwrap();
        assert!(gif.starts_with(b"GIF89a"));
    }

    #[test]
    fn test_gif_size_clamped() {
        // Over MAX_GIF_SIZE should be clamped
        let gif = generate_gif("test", "rings", 2048, None, 4, 10).unwrap();
        assert!(gif.starts_with(b"GIF89a"));
    }

    #[test]
    fn test_gif_with_background() {
        let gif = generate_gif("test", "rings", 64, Some((255, 0, 0)), 4, 10).unwrap();
        assert!(gif.starts_with(b"GIF89a"));
    }

    #[test]
    fn test_animate_rings_frames() {
        let frames = generate_frames("test", "rings", 64, None, 6).unwrap();
        assert_eq!(frames.len(), 6);
        // Frames should differ (animation)
        assert_ne!(frames[0].as_raw(), frames[3].as_raw(), "Animated frames should differ");
    }

    #[test]
    fn test_animate_robot_blink() {
        // Frame at t=0.0 (eyes open) vs t=0.93 (eyes closed)
        let open = generate_animated_frame("test", "robot", 128, None, 0.0).unwrap();
        let closed = generate_animated_frame("test", "robot", 128, None, 0.93).unwrap();
        assert_ne!(open.as_raw(), closed.as_raw(), "Robot blink should change frame");
    }

    #[test]
    fn test_animate_starburst_rotation() {
        let f1 = generate_animated_frame("test", "starburst", 64, None, 0.0).unwrap();
        let f2 = generate_animated_frame("test", "starburst", 64, None, 0.5).unwrap();
        assert_ne!(f1.as_raw(), f2.as_raw(), "Starburst should rotate between frames");
    }

    #[test]
    fn test_animate_gradient_rotation() {
        let f1 = generate_animated_frame("test", "gradient", 64, None, 0.0).unwrap();
        let f2 = generate_animated_frame("test", "gradient", 64, None, 0.5).unwrap();
        assert_ne!(f1.as_raw(), f2.as_raw(), "Gradient should rotate between frames");
    }

    #[test]
    fn test_animate_pixel_color_cycle() {
        let f1 = generate_animated_frame("test", "pixel", 64, None, 0.0).unwrap();
        let f2 = generate_animated_frame("test", "pixel", 64, None, 0.5).unwrap();
        assert_ne!(f1.as_raw(), f2.as_raw(), "Pixel colors should cycle");
    }

    #[test]
    fn test_animate_sunset_movement() {
        let f1 = generate_animated_frame("test", "sunset", 64, None, 0.0).unwrap();
        let f2 = generate_animated_frame("test", "sunset", 64, None, 0.5).unwrap();
        assert_ne!(f1.as_raw(), f2.as_raw(), "Sunset should animate");
    }

    #[test]
    fn test_animate_pulse_generic() {
        let f1 = generate_animated_frame("test", "geometric", 64, None, 0.0).unwrap();
        let f2 = generate_animated_frame("test", "geometric", 64, None, 0.25).unwrap();
        assert_ne!(f1.as_raw(), f2.as_raw(), "Pulse should change brightness");
    }

    #[test]
    fn test_shift_hue_basics() {
        let red = (255, 0, 0);
        let shifted = shift_hue(red, 120);
        // Red shifted 120° should be roughly green
        assert!(shifted.1 > shifted.0, "120° from red should be greenish");
    }

    #[test]
    fn test_shift_hue_full_circle() {
        let color = (100, 150, 200);
        let shifted = shift_hue(color, 360);
        // 360° shift should return roughly the same color (may have tiny rounding diffs)
        assert!((color.0 as i16 - shifted.0 as i16).unsigned_abs() <= 1);
        assert!((color.1 as i16 - shifted.1 as i16).unsigned_abs() <= 1);
        assert!((color.2 as i16 - shifted.2 as i16).unsigned_abs() <= 1);
    }

    #[test]
    fn test_rgb_hsl_roundtrip() {
        let original = (128, 64, 200);
        let (h, s, l) = rgb_to_hsl(original);
        let back = hsl_to_rgb(h, s, l);
        assert!((original.0 as i16 - back.0 as i16).unsigned_abs() <= 1);
        assert!((original.1 as i16 - back.1 as i16).unsigned_abs() <= 1);
        assert!((original.2 as i16 - back.2 as i16).unsigned_abs() <= 1);
    }
}
