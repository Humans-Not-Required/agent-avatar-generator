use crate::avatar;
use crate::rate_limit::{RateGuard, RateLimited};
use crate::styles::{self, DEFAULT_SIZE, DEFAULT_STYLE, MAX_SIZE, MIN_SIZE};

use base64::Engine;
use rocket::http::{ContentType, Header, Status};
use rocket::response::content::{RawHtml, RawText};
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use rocket::{get, post, Request};
use serde::{Deserialize, Serialize};

// ── Response Types ──

pub struct ImageResponse {
    pub bytes: Vec<u8>,
    pub content_type: ContentType,
    pub cache_control: String,
}

impl<'r> Responder<'r, 'static> for ImageResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(self.content_type)
            .header(Header::new("Cache-Control", self.cache_control))
            .sized_body(self.bytes.len(), std::io::Cursor::new(self.bytes))
            .ok()
    }
}

pub struct SvgResponse {
    pub body: String,
    pub cache_control: String,
}

impl<'r> Responder<'r, 'static> for SvgResponse {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::SVG)
            .header(Header::new("Cache-Control", self.cache_control))
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
                detail: Some("Valid styles: geometric, rings, robot, blockies, gradient".to_string()),
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

const CACHE_HEADER: &str = "public, max-age=31536000, immutable";

// Type aliases to reduce complexity warnings
type ApiResult<T> = Result<T, (Status, Json<ApiError>)>;
type RatedApiResult<T> = Result<RateLimited<ApiResult<T>>, (Status, Json<ApiError>)>;

// ── Routes ──

/// GET /api/v1/avatar/<seed> — generate avatar (PNG default)
#[get("/avatar/<seed>?<style>&<size>&<format>&<background>")]
pub fn generate_avatar(
    seed: &str,
    style: Option<String>,
    size: Option<u32>,
    format: Option<String>,
    background: Option<String>,
    rate: RateGuard,
) -> RatedApiResult<ImageResponseOrSvg> {
    let (style, size) = validate_params(&style, &size)?;
    let bg = parse_bg(&background);
    let fmt = format.as_deref().unwrap_or("png");

    let inner = match fmt {
        "svg" => match avatar::generate_svg(seed, &style, size, bg) {
            Ok(svg) => Ok(ImageResponseOrSvg::Svg(SvgResponse {
                body: svg,
                cache_control: CACHE_HEADER.to_string(),
            })),
            Err(e) => Err((
                Status::InternalServerError,
                Json(ApiError {
                    error: e,
                    detail: None,
                }),
            )),
        },
        "png" => match avatar::generate_png(seed, &style, size, bg) {
            Ok(bytes) => Ok(ImageResponseOrSvg::Png(ImageResponse {
                bytes,
                content_type: ContentType::PNG,
                cache_control: CACHE_HEADER.to_string(),
            })),
            Err(e) => Err((
                Status::InternalServerError,
                Json(ApiError {
                    error: e,
                    detail: None,
                }),
            )),
        },
        _ => Err((
            Status::BadRequest,
            Json(ApiError {
                error: format!("Unknown format: {fmt}"),
                detail: Some("Valid formats: png, svg".to_string()),
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

    let inner = match avatar::generate_png(seed, DEFAULT_STYLE, DEFAULT_SIZE, None) {
        Ok(bytes) => Ok(ImageResponse {
            bytes,
            content_type: ContentType::PNG,
            cache_control: CACHE_HEADER.to_string(),
        }),
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

    let inner = match avatar::generate_svg(seed, DEFAULT_STYLE, DEFAULT_SIZE, None) {
        Ok(svg) => Ok(SvgResponse {
            body: svg,
            cache_control: CACHE_HEADER.to_string(),
        }),
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

    let mut results = Vec::new();
    for seed in &req.seeds {
        let data = match format {
            "svg" => avatar::generate_svg(seed, style, size, bg)
                .map(|svg| base64::engine::general_purpose::STANDARD.encode(svg.as_bytes())),
            _ => avatar::generate_png(seed, style, size, bg)
                .map(|bytes| base64::engine::general_purpose::STANDARD.encode(&bytes)),
        };

        match data {
            Ok(encoded) => results.push(BatchItem {
                seed: seed.clone(),
                data: encoded,
                format: format.to_string(),
                error: None,
            }),
            Err(e) => results.push(BatchItem {
                seed: seed.clone(),
                data: String::new(),
                format: format.to_string(),
                error: Some(e),
            }),
        }
    }

    RateLimited {
        inner: Ok(Json(BatchResponse { avatars: results })),
        limit: rate.limit,
        remaining: rate.remaining,
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
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    pub avatars: Vec<BatchItem>,
}

#[derive(Debug, Serialize)]
pub struct BatchItem {
    pub seed: String,
    pub data: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
                batch_generate,
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

// Union type for PNG/SVG response
pub enum ImageResponseOrSvg {
    Png(ImageResponse),
    Svg(SvgResponse),
}

impl<'r> Responder<'r, 'static> for ImageResponseOrSvg {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Self::Png(r) => r.respond_to(req),
            Self::Svg(r) => r.respond_to(req),
        }
    }
}
