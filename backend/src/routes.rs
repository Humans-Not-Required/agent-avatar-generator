use crate::animation;
use crate::avatar;
use crate::rate_limit::{RateGuard, RateLimited};
use crate::styles::{self, DEFAULT_SIZE, DEFAULT_STYLE, MAX_SIZE, MIN_SIZE};
use crate::theme::{self, Theme};

use base64::Engine;
use image::ImageFormat;
use rayon::prelude::*;
use rocket::http::{ContentType, Header, Status};
use rocket::response::content::{RawHtml, RawText};
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::{get, post, Request};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// ── Response Types ──

pub struct ImageResponse {
    pub bytes: Vec<u8>,
    pub content_type: ContentType,
    pub cache_control: String,
    pub generation_ms: Option<f64>,
}

impl<'r> Responder<'r, 'static> for ImageResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(self.content_type)
            .header(Header::new("Cache-Control", self.cache_control));
        if let Some(ms) = self.generation_ms {
            builder.header(Header::new("X-Generation-Time-Ms", format!("{ms:.2}")));
        }
        builder
            .sized_body(self.bytes.len(), std::io::Cursor::new(self.bytes))
            .ok()
    }
}

pub struct SvgResponse {
    pub body: String,
    pub cache_control: String,
    pub generation_ms: Option<f64>,
}

impl<'r> Responder<'r, 'static> for SvgResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(ContentType::SVG)
            .header(Header::new("Cache-Control", self.cache_control));
        if let Some(ms) = self.generation_ms {
            builder.header(Header::new("X-Generation-Time-Ms", format!("{ms:.2}")));
        }
        builder
            .sized_body(self.body.len(), std::io::Cursor::new(self.body))
            .ok()
    }
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

// ── Query Params ──

fn parse_bg(bg: &Option<String>) -> Option<(u8, u8, u8)> {
    bg.as_ref().and_then(|s| {
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some((r, g, b))
    })
}

fn validate_params(
    style: &Option<String>,
    size: &Option<u32>,
) -> Result<(String, u32), (Status, Json<ApiError>)> {
    let style = style.as_deref().unwrap_or(DEFAULT_STYLE).to_lowercase();
    if !styles::is_valid_style(&style) {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                error: format!("Unknown style: {style}"),
                detail: Some("Valid styles: geometric, rings, robot, blockies, gradient, initials, starburst, mosaic, pixel, sunset".to_string()),
            }),
        ));
    }

    let size = size.unwrap_or(DEFAULT_SIZE);
    if !(MIN_SIZE..=MAX_SIZE).contains(&size) {
        return Err((
            Status::BadRequest,
            Json(ApiError {
                error: format!("Size must be between {MIN_SIZE} and {MAX_SIZE}"),
                detail: None,
            }),
        ));
    }

    Ok((style, size))
}

fn parse_theme(theme: &Option<String>) -> Result<Option<Theme>, (Status, Json<ApiError>)> {
    match theme {
        None => Ok(None),
        Some(name) => {
            Theme::parse(name).map(Some).ok_or_else(|| (
                Status::BadRequest,
                Json(ApiError {
                    error: format!("Unknown theme: {name}"),
                    detail: Some("Valid themes: warm, cool, ocean, forest, sunset, neon, pastel, monochrome, earth".to_string()),
                }),
            ))
        }
    }
}

/// Generate a themed PNG from an image (avoids double encode/decode).
fn themed_png(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>, t: &Theme) -> Result<Vec<u8>, String> {
    let mut img = avatar::generate_image(seed, style, size, bg)?;
    t.apply_to_image(&mut img);
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| format!("PNG encoding error: {e}"))?;
    Ok(buf)
}

/// Generate a themed SVG (remap all hex colors in the SVG string).
fn themed_svg(seed: &str, style: &str, size: u32, bg: Option<(u8, u8, u8)>, t: &Theme) -> Result<String, String> {
    let svg = avatar::generate_svg(seed, style, size, bg)?;
    Ok(t.apply_to_svg(&svg))
}

const CACHE_HEADER: &str = "public, max-age=31536000, immutable";

// Type aliases to reduce complexity warnings
type ApiResult<T> = Result<T, (Status, Json<ApiError>)>;
type RatedApiResult<T> = Result<RateLimited<ApiResult<T>>, (Status, Json<ApiError>)>;

// ── Routes ──

/// GET /api/v1/avatar/<seed> — generate avatar (PNG default, also SVG and GIF)
#[get("/avatar/<seed>?<style>&<size>&<format>&<background>&<theme>&<frames>&<delay>")]
#[allow(clippy::too_many_arguments)]
pub fn generate_avatar(
    seed: &str,
    style: Option<String>,
    size: Option<u32>,
    format: Option<String>,
    background: Option<String>,
    theme: Option<String>,
    frames: Option<u16>,
    delay: Option<u16>,
    rate: RateGuard,
) -> RatedApiResult<ImageResponseOrSvg> {
    let (style, size) = validate_params(&style, &size)?;
    let bg = parse_bg(&background);
    let t = parse_theme(&theme)?;
    let fmt = format.as_deref().unwrap_or("png");
    let start = Instant::now();

    let inner = match fmt {
        "svg" => {
            let result = if let Some(ref t) = t {
                themed_svg(seed, &style, size, bg, t)
            } else {
                avatar::generate_svg(seed, &style, size, bg)
            };
            let gen_ms = start.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(svg) => Ok(ImageResponseOrSvg::Svg(SvgResponse {
                    body: svg,
                    cache_control: CACHE_HEADER.to_string(),
                    generation_ms: Some(gen_ms),
                })),
                Err(e) => Err((
                    Status::InternalServerError,
                    Json(ApiError { error: e, detail: None }),
                )),
            }
        }
        "png" => {
            let result = if let Some(ref t) = t {
                themed_png(seed, &style, size, bg, t)
            } else {
                avatar::generate_png(seed, &style, size, bg)
            };
            let gen_ms = start.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(bytes) => Ok(ImageResponseOrSvg::Png(ImageResponse {
                    bytes,
                    content_type: ContentType::PNG,
                    cache_control: CACHE_HEADER.to_string(),
                    generation_ms: Some(gen_ms),
                })),
                Err(e) => Err((
                    Status::InternalServerError,
                    Json(ApiError { error: e, detail: None }),
                )),
            }
        }
        "gif" => {
            let gif_size = size.min(animation::MAX_GIF_SIZE);
            let frame_count = frames.unwrap_or(animation::DEFAULT_FRAMES);
            let frame_delay = delay.unwrap_or(animation::DEFAULT_DELAY);
            let result = animation::generate_gif(seed, &style, gif_size, bg, frame_count, frame_delay);
            let gen_ms = start.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(bytes) => Ok(ImageResponseOrSvg::Gif(GifResponse {
                    bytes,
                    cache_control: CACHE_HEADER.to_string(),
                    generation_ms: Some(gen_ms),
                    frame_count: Some(frame_count.clamp(2, animation::MAX_FRAMES)),
                })),
                Err(e) => Err((
                    Status::InternalServerError,
                    Json(ApiError { error: e, detail: None }),
                )),
            }
        }
        _ => Err((
            Status::BadRequest,
            Json(ApiError {
                error: format!("Unknown format: {fmt}"),
                detail: Some("Valid formats: png, svg, gif".to_string()),
            }),
        )),
    };

    Ok(RateLimited {
        inner,
        limit: rate.limit,
        remaining: rate.remaining,
    })
}

/// GET /api/v1/avatar/<seed>.png
#[get("/avatar/<seed_png>", rank = 2)]
pub fn generate_avatar_png(
    seed_png: &str,
    rate: RateGuard,
) -> RatedApiResult<ImageResponse> {
    let seed = seed_png.strip_suffix(".png").unwrap_or(seed_png);

    // If it actually ends with .svg, handle that
    if seed_png.ends_with(".svg") {
        // This shouldn't match since we have a separate route, but just in case
        return Ok(RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: "Use .svg extension route".to_string(),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        });
    }

    let start = Instant::now();
    let inner = match avatar::generate_png(seed, DEFAULT_STYLE, DEFAULT_SIZE, None) {
        Ok(bytes) => {
            let gen_ms = start.elapsed().as_secs_f64() * 1000.0;
            Ok(ImageResponse {
                bytes,
                content_type: ContentType::PNG,
                cache_control: CACHE_HEADER.to_string(),
                generation_ms: Some(gen_ms),
            })
        }
        Err(e) => Err((
            Status::InternalServerError,
            Json(ApiError {
                error: e,
                detail: None,
            }),
        )),
    };

    Ok(RateLimited {
        inner,
        limit: rate.limit,
        remaining: rate.remaining,
    })
}

/// GET /api/v1/avatar/<seed>.svg
#[get("/avatar/<seed_svg>", rank = 3)]
pub fn generate_avatar_svg(
    seed_svg: &str,
    rate: RateGuard,
) -> RatedApiResult<SvgResponse> {
    let seed = seed_svg
        .strip_suffix(".svg")
        .unwrap_or(seed_svg);

    let start = Instant::now();
    let inner = match avatar::generate_svg(seed, DEFAULT_STYLE, DEFAULT_SIZE, None) {
        Ok(svg) => {
            let gen_ms = start.elapsed().as_secs_f64() * 1000.0;
            Ok(SvgResponse {
                body: svg,
                cache_control: CACHE_HEADER.to_string(),
                generation_ms: Some(gen_ms),
            })
        }
        Err(e) => Err((
            Status::InternalServerError,
            Json(ApiError {
                error: e,
                detail: None,
            }),
        )),
    };

    Ok(RateLimited {
        inner,
        limit: rate.limit,
        remaining: rate.remaining,
    })
}

/// GET /api/v1/styles — list available styles
#[get("/styles")]
pub fn list_styles() -> Json<Vec<styles::StyleInfo>> {
    Json(styles::available_styles())
}

/// GET /api/v1/themes — list available color themes
#[get("/themes")]
pub fn list_themes() -> Json<Vec<theme::ThemeInfo>> {
    Json(theme::available_themes())
}

/// POST /api/v1/avatar/batch — batch generate avatars
#[post("/avatar/batch", format = "json", data = "<req>")]
pub fn batch_generate(
    req: Json<BatchRequest>,
    rate: RateGuard,
) -> RateLimited<ApiResult<Json<BatchResponse>>> {
    let style = req.style.as_deref().unwrap_or(DEFAULT_STYLE);
    let size = req.size.unwrap_or(128);
    let format = req.format.as_deref().unwrap_or("png");

    if !styles::is_valid_style(style) {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: format!("Unknown style: {style}"),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    if !(MIN_SIZE..=MAX_SIZE).contains(&size) {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: format!("Size must be between {MIN_SIZE} and {MAX_SIZE}"),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    if req.seeds.len() > 50 {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: "Maximum 50 seeds per batch".to_string(),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    if req.seeds.is_empty() {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: "At least one seed required".to_string(),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    let bg = parse_bg(&req.background);
    let t = req.theme.as_ref().and_then(|n| Theme::parse(n));
    let start = Instant::now();

    let gif_frames = req.frames.unwrap_or(animation::DEFAULT_FRAMES);
    let gif_delay = req.delay.unwrap_or(animation::DEFAULT_DELAY);

    // Generate all avatars in parallel using rayon
    let results: Vec<BatchItem> = req.seeds
        .par_iter()
        .map(|seed| {
            let data = match format {
                "svg" => {
                    if let Some(ref t) = t {
                        themed_svg(seed, style, size, bg, t)
                            .map(|svg| base64::engine::general_purpose::STANDARD.encode(svg.as_bytes()))
                    } else {
                        avatar::generate_svg(seed, style, size, bg)
                            .map(|svg| base64::engine::general_purpose::STANDARD.encode(svg.as_bytes()))
                    }
                }
                "gif" => {
                    let gif_size = size.min(animation::MAX_GIF_SIZE);
                    animation::generate_gif(seed, style, gif_size, bg, gif_frames, gif_delay)
                        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
                }
                _ => {
                    if let Some(ref t) = t {
                        themed_png(seed, style, size, bg, t)
                            .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
                    } else {
                        avatar::generate_png(seed, style, size, bg)
                            .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes))
                    }
                }
            };

            match data {
                Ok(encoded) => BatchItem {
                    seed: seed.clone(),
                    data: encoded,
                    format: format.to_string(),
                    error: None,
                },
                Err(e) => BatchItem {
                    seed: seed.clone(),
                    data: String::new(),
                    format: format.to_string(),
                    error: Some(e),
                },
            }
        })
        .collect();

    let gen_ms = start.elapsed().as_secs_f64() * 1000.0;

    RateLimited {
        inner: Ok(Json(BatchResponse {
            avatars: results,
            generation_ms: gen_ms,
            count: req.seeds.len(),
        })),
        limit: rate.limit,
        remaining: rate.remaining,
    }
}

/// POST /api/v1/avatar/gallery/zip — download multiple avatars as a ZIP file
#[post("/avatar/gallery/zip", format = "json", data = "<req>")]
pub fn gallery_zip(
    req: Json<GalleryZipRequest>,
    rate: RateGuard,
) -> RateLimited<ApiResult<ZipResponse>> {
    let format = req.format.as_deref().unwrap_or("png");
    if format != "png" && format != "svg" && format != "gif" {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: format!("Unknown format: {format}"),
                    detail: Some("Valid formats: png, svg, gif".to_string()),
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    if req.seeds.is_empty() {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: "At least one seed required".to_string(),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    if req.seeds.len() > 50 {
        return RateLimited {
            inner: Err((
                Status::BadRequest,
                Json(ApiError {
                    error: "Maximum 50 seeds per ZIP".to_string(),
                    detail: None,
                }),
            )),
            limit: rate.limit,
            remaining: rate.remaining,
        };
    }

    let size = req.size.unwrap_or(DEFAULT_SIZE).clamp(MIN_SIZE, MAX_SIZE);
    let bg = parse_bg(&req.background);
    let t = req.theme.as_ref().and_then(|n| Theme::parse(n));

    // Determine which styles to include
    let all_mode = req.style.as_deref() == Some("all");
    let style_list: Vec<String> = if all_mode {
        styles::available_styles()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    } else {
        let s = req.style.as_deref().unwrap_or(DEFAULT_STYLE);
        if !styles::is_valid_style(s) {
            return RateLimited {
                inner: Err((
                    Status::BadRequest,
                    Json(ApiError {
                        error: format!("Unknown style: {s}"),
                        detail: Some("Valid styles: geometric, rings, robot, blockies, gradient, initials, starburst, mosaic, pixel, sunset — or 'all'".to_string()),
                    }),
                )),
                limit: rate.limit,
                remaining: rate.remaining,
            };
        }
        vec![s.to_string()]
    };

    let start = Instant::now();

    // Build list of (seed, style) pairs to generate
    let pairs: Vec<(String, String)> = req.seeds
        .iter()
        .flat_map(|seed| {
            style_list.iter().map(move |style| (seed.clone(), style.clone()))
        })
        .collect();

    let total_count = pairs.len();

    // Generate all images in parallel
    let generated: Vec<(String, Result<Vec<u8>, String>)> = pairs
        .par_iter()
        .map(|(seed, style)| {
            let filename = if all_mode {
                format!("{seed}_{style}.{format}")
            } else {
                format!("{seed}.{format}")
            };

            let data = match format {
                "svg" => {
                    if let Some(ref t) = t {
                        themed_svg(seed, style, size, bg, t).map(|s| s.into_bytes())
                    } else {
                        avatar::generate_svg(seed, style, size, bg).map(|s| s.into_bytes())
                    }
                }
                "gif" => {
                    let gif_size = size.min(animation::MAX_GIF_SIZE);
                    let gif_frames = req.frames.unwrap_or(animation::DEFAULT_FRAMES);
                    let gif_delay = req.delay.unwrap_or(animation::DEFAULT_DELAY);
                    animation::generate_gif(seed, style, gif_size, bg, gif_frames, gif_delay)
                }
                _ => {
                    if let Some(ref t) = t {
                        themed_png(seed, style, size, bg, t)
                    } else {
                        avatar::generate_png(seed, style, size, bg)
                    }
                }
            };

            (filename, data)
        })
        .collect();

    // Write to ZIP sequentially (ZipWriter is not thread-safe)
    let mut zip_buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip_writer = zip::ZipWriter::new(&mut zip_buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (filename, data) in &generated {
            if let Ok(bytes) = data {
                let _ = zip_writer.start_file(filename, options);
                let _ = std::io::Write::write_all(&mut zip_writer, bytes);
            }
        }

        let _ = zip_writer.finish();
    }

    let gen_ms = start.elapsed().as_secs_f64() * 1000.0;

    RateLimited {
        inner: Ok(ZipResponse {
            bytes: zip_buf.into_inner(),
            filename: "avatars.zip".to_string(),
            generation_ms: Some(gen_ms),
            count: Some(total_count),
        }),
        limit: rate.limit,
        remaining: rate.remaining,
    }
}

pub struct ZipResponse {
    pub bytes: Vec<u8>,
    pub filename: String,
    pub generation_ms: Option<f64>,
    pub count: Option<usize>,
}

impl<'r> Responder<'r, 'static> for ZipResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(ContentType::ZIP)
            .header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.filename),
            ));
        if let Some(ms) = self.generation_ms {
            builder.header(Header::new("X-Generation-Time-Ms", format!("{ms:.2}")));
        }
        if let Some(n) = self.count {
            builder.header(Header::new("X-Avatar-Count", n.to_string()));
        }
        builder
            .sized_body(self.bytes.len(), std::io::Cursor::new(self.bytes))
            .ok()
    }
}

/// GET /api/v1/health
#[get("/health")]
pub fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "agent-avatar-generator",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /api/v1/openapi.json
#[get("/openapi.json")]
pub fn openapi() -> (ContentType, &'static str) {
    (ContentType::JSON, include_str!("../openapi.json"))
}

/// GET /api/v1/llms.txt
#[get("/llms.txt")]
pub fn llms_txt() -> RawText<&'static str> {
    RawText(include_str!("../llms.txt"))
}

/// GET /llms.txt (root)
#[get("/llms.txt")]
pub fn root_llms_txt() -> RawText<&'static str> {
    RawText(include_str!("../llms.txt"))
}

/// GET /.well-known/skills/agent-avatar-generator/index.json
#[get("/.well-known/skills/agent-avatar-generator/index.json")]
pub fn skills_index() -> (ContentType, String) {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    (
        ContentType::JSON,
        serde_json::json!({
            "skills": [{
                "id": "agent-avatar-generator",
                "name": "Agent Avatar Generator",
                "description": "Self-hosted deterministic avatar generation for AI agents",
                "version": env!("CARGO_PKG_VERSION"),
                "skill_url": format!("{base_url}/.well-known/skills/agent-avatar-generator/SKILL.md"),
            }]
        })
        .to_string(),
    )
}

/// GET /.well-known/skills/agent-avatar-generator/SKILL.md
#[get("/.well-known/skills/agent-avatar-generator/SKILL.md")]
pub fn skills_skill_md() -> RawText<&'static str> {
    RawText(include_str!("../SKILL.md"))
}

/// GET /avatar/view/<seed> — share URL with preview
#[get("/avatar/view/<seed>?<style>&<size>")]
pub fn view_avatar(seed: &str, style: Option<String>, size: Option<u32>) -> RawHtml<String> {
    let style = style.as_deref().unwrap_or(DEFAULT_STYLE);
    let size = size.unwrap_or(DEFAULT_SIZE).clamp(MIN_SIZE, MAX_SIZE);
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

    RawHtml(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Avatar: {seed}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; margin: 0; background: #1a1a2e; color: #e0e0e0; }}
        .card {{ background: #16213e; border-radius: 16px; padding: 2rem; text-align: center; box-shadow: 0 4px 20px rgba(0,0,0,0.3); }}
        img {{ border-radius: 12px; margin: 1rem 0; }}
        h2 {{ margin: 0 0 0.5rem; color: #a0c4ff; }}
        .meta {{ color: #888; font-size: 0.9rem; }}
        .actions {{ margin-top: 1rem; display: flex; gap: 0.5rem; }}
        a {{ background: #0f3460; color: #e0e0e0; padding: 0.5rem 1rem; border-radius: 8px; text-decoration: none; font-size: 0.9rem; }}
        a:hover {{ background: #1a4a7a; }}
    </style>
</head>
<body>
    <div class="card">
        <h2>🤖 {seed}</h2>
        <p class="meta">Style: {style} · {size}×{size}px</p>
        <img src="{base_url}/api/v1/avatar/{seed}?style={style}&size={size}" alt="Avatar for {seed}" width="{size}" height="{size}">
        <div class="actions">
            <a href="{base_url}/api/v1/avatar/{seed}?style={style}&size={size}&format=png" download="{seed}.png">⬇ PNG</a>
            <a href="{base_url}/api/v1/avatar/{seed}?style={style}&size={size}&format=svg" download="{seed}.svg">⬇ SVG</a>
        </div>
    </div>
</body>
</html>"#,
    ))
}

/// Catch-all SPA fallback
#[get("/<_path..>", rank = 100)]
pub fn spa_fallback(_path: std::path::PathBuf) -> Option<rocket::fs::NamedFile> {
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "../frontend/dist".to_string());
    let index = std::path::PathBuf::from(static_dir).join("index.html");
    rocket::tokio::runtime::Handle::current().block_on(rocket::fs::NamedFile::open(index)).ok()
}

// ── Request/Response Types ──

#[derive(Debug, Deserialize)]
pub struct BatchRequest {
    pub seeds: Vec<String>,
    pub style: Option<String>,
    pub size: Option<u32>,
    pub format: Option<String>,
    pub background: Option<String>,
    pub theme: Option<String>,
    pub frames: Option<u16>,
    pub delay: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    pub avatars: Vec<BatchItem>,
    pub generation_ms: f64,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct BatchItem {
    pub seed: String,
    pub data: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GalleryZipRequest {
    pub seeds: Vec<String>,
    pub style: Option<String>,
    pub size: Option<u32>,
    pub format: Option<String>,
    pub background: Option<String>,
    pub theme: Option<String>,
    pub frames: Option<u16>,
    pub delay: Option<u16>,
}

/// Build a Rocket instance for testing.
#[allow(dead_code)]
pub fn test_rocket() -> rocket::Rocket<rocket::Build> {
    use crate::rate_limit::RateLimiter;
    use rocket::routes;
    use rocket_cors::{AllowedOrigins, CorsOptions};
    use std::time::Duration;

    let window_secs: u64 = std::env::var("RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let limiter = RateLimiter::new(Duration::from_secs(window_secs));

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("CORS configuration failed");

    rocket::build()
        .attach(cors)
        .manage(limiter)
        .mount(
            "/api/v1",
            routes![
                health,
                openapi,
                llms_txt,
                generate_avatar,
                generate_avatar_png,
                generate_avatar_svg,
                list_styles,
                list_themes,
                batch_generate,
                gallery_zip,
            ],
        )
        .mount(
            "/",
            routes![
                root_llms_txt,
                skills_index,
                skills_skill_md,
                view_avatar,
            ],
        )
}

pub struct GifResponse {
    pub bytes: Vec<u8>,
    pub cache_control: String,
    pub generation_ms: Option<f64>,
    pub frame_count: Option<u16>,
}

impl<'r> Responder<'r, 'static> for GifResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        let mut builder = Response::build();
        builder
            .header(ContentType::GIF)
            .header(Header::new("Cache-Control", self.cache_control));
        if let Some(ms) = self.generation_ms {
            builder.header(Header::new("X-Generation-Time-Ms", format!("{ms:.2}")));
        }
        if let Some(n) = self.frame_count {
            builder.header(Header::new("X-Frame-Count", n.to_string()));
        }
        builder
            .sized_body(self.bytes.len(), std::io::Cursor::new(self.bytes))
            .ok()
    }
}

// Union type for PNG/SVG/GIF response
pub enum ImageResponseOrSvg {
    Png(ImageResponse),
    Svg(SvgResponse),
    Gif(GifResponse),
}

impl<'r> Responder<'r, 'static> for ImageResponseOrSvg {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Self::Png(r) => r.respond_to(req),
            Self::Svg(r) => r.respond_to(req),
            Self::Gif(r) => r.respond_to(req),
        }
    }
}
