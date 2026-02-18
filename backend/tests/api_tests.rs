use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;

fn client() -> Client {
    let _ = dotenvy::dotenv();
    std::env::set_var("RATE_LIMIT_WINDOW_SECS", "1");
    let rocket = agent_avatar_generator::routes::test_rocket();
    Client::tracked(rocket).expect("valid rocket instance")
}

// ── Health ──

#[test]
fn test_health() {
    let client = client();
    let response = client.get("/api/v1/health").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "agent-avatar-generator");
    assert!(body["version"].is_string());
}

// ── Generate Avatar (PNG) ──

#[test]
fn test_generate_png_default() {
    let client = client();
    let response = client.get("/api/v1/avatar/nanook").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::PNG));
    let bytes = response.into_bytes().unwrap();
    assert!(!bytes.is_empty());
    // PNG magic header
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_generate_png_explicit_format() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?format=png").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::PNG));
}

#[test]
fn test_generate_svg() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?format=svg").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::SVG));
    let body = response.into_string().unwrap();
    assert!(body.starts_with("<svg"));
    assert!(body.contains("</svg>"));
}

// ── Styles ──

#[test]
fn test_all_styles_png() {
    let client = client();
    for style in &["geometric", "rings", "robot", "blockies", "gradient"] {
        let response = client
            .get(format!("/api/v1/avatar/test?style={style}"))
            .dispatch();
        assert_eq!(
            response.status(),
            Status::Ok,
            "Style {style} should return 200"
        );
        assert_eq!(response.content_type(), Some(ContentType::PNG));
    }
}

#[test]
fn test_all_styles_svg() {
    let client = client();
    for style in &["geometric", "rings", "robot", "blockies", "gradient"] {
        let response = client
            .get(format!("/api/v1/avatar/test?style={style}&format=svg"))
            .dispatch();
        assert_eq!(
            response.status(),
            Status::Ok,
            "Style {style} SVG should return 200"
        );
        assert_eq!(response.content_type(), Some(ContentType::SVG));
    }
}

#[test]
fn test_invalid_style() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?style=invalid").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

// ── Sizes ──

#[test]
fn test_custom_size() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?size=128").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_min_size() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?size=16").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_max_size() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?size=1024").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_size_too_small() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?size=8").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_size_too_large() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?size=2048").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

// ── Background Color ──

#[test]
fn test_background_color() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/test?background=ff0000")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_background_with_hash() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/test?background=%23ff0000")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

// ── Determinism ──

#[test]
fn test_deterministic_output() {
    let client = client();
    let r1 = client.get("/api/v1/avatar/deterministic-test").dispatch();
    let b1 = r1.into_bytes().unwrap();
    let r2 = client.get("/api/v1/avatar/deterministic-test").dispatch();
    let b2 = r2.into_bytes().unwrap();
    assert_eq!(b1, b2, "Same seed must produce identical output");
}

#[test]
fn test_different_seeds_differ() {
    let client = client();
    let r1 = client.get("/api/v1/avatar/seed-alpha").dispatch();
    let b1 = r1.into_bytes().unwrap();
    let r2 = client.get("/api/v1/avatar/seed-beta").dispatch();
    let b2 = r2.into_bytes().unwrap();
    assert_ne!(b1, b2, "Different seeds should produce different output");
}

// ── Formats ──

#[test]
fn test_invalid_format() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/test?format=gif")
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

// ── Batch ──

#[test]
fn test_batch_generate() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["a", "b", "c"]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert_eq!(body["avatars"].as_array().unwrap().len(), 3);
    for item in body["avatars"].as_array().unwrap() {
        assert!(!item["data"].as_str().unwrap().is_empty());
        assert_eq!(item["format"], "png");
    }
}

#[test]
fn test_batch_svg() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["x", "y"], "format": "svg"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert_eq!(body["avatars"].as_array().unwrap().len(), 2);
    for item in body["avatars"].as_array().unwrap() {
        assert_eq!(item["format"], "svg");
    }
}

#[test]
fn test_batch_with_style() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["test"], "style": "robot", "size": 64}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_batch_too_many() {
    let client = client();
    let seeds: Vec<String> = (0..51).map(|i| format!("seed-{i}")).collect();
    let body = serde_json::json!({"seeds": seeds});
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_batch_empty() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": []}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_batch_invalid_style() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["a"], "style": "bad"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

// ── Styles Endpoint ──

#[test]
fn test_list_styles() {
    let client = client();
    let response = client.get("/api/v1/styles").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let styles = body.as_array().unwrap();
    assert_eq!(styles.len(), 5);
    let names: Vec<&str> = styles.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"geometric"));
    assert!(names.contains(&"rings"));
    assert!(names.contains(&"robot"));
    assert!(names.contains(&"blockies"));
    assert!(names.contains(&"gradient"));
}

// ── Discovery ──

#[test]
fn test_openapi() {
    let client = client();
    let response = client.get("/api/v1/openapi.json").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert_eq!(body["openapi"], "3.0.3");
    assert_eq!(body["info"]["title"], "Agent Avatar Generator");
}

#[test]
fn test_llms_txt() {
    let client = client();
    let response = client.get("/api/v1/llms.txt").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("Agent Avatar Generator"));
}

#[test]
fn test_root_llms_txt() {
    let client = client();
    let response = client.get("/llms.txt").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("Agent Avatar Generator"));
}

#[test]
fn test_skills_index() {
    let client = client();
    let response = client
        .get("/.well-known/skills/agent-avatar-generator/index.json")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert!(body["skills"].is_array());
}

#[test]
fn test_skills_skill_md() {
    let client = client();
    let response = client
        .get("/.well-known/skills/agent-avatar-generator/SKILL.md")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("Agent Avatar Generator"));
}

// ── View Page ──

#[test]
fn test_view_avatar_page() {
    let client = client();
    let response = client.get("/avatar/view/nanook").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("nanook"));
    assert!(body.contains("<img"));
    assert!(body.contains("PNG"));
    assert!(body.contains("SVG"));
}

#[test]
fn test_view_avatar_with_params() {
    let client = client();
    let response = client
        .get("/avatar/view/test?style=robot&size=512")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("robot"));
}

// ── Cache Headers ──

#[test]
fn test_cache_headers() {
    let client = client();
    let response = client.get("/api/v1/avatar/cache-test").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let cache = response
        .headers()
        .get_one("Cache-Control")
        .unwrap_or("");
    assert!(cache.contains("immutable"));
    assert!(cache.contains("max-age=31536000"));
}

// ── Rate Limit Headers ──

#[test]
fn test_rate_limit_headers() {
    let client = client();
    let response = client.get("/api/v1/avatar/rate-test").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().get_one("X-RateLimit-Limit").is_some());
    assert!(response.headers().get_one("X-RateLimit-Remaining").is_some());
}

// ── Special Seeds ──

#[test]
fn test_empty_seed() {
    // URL-encoded empty string shouldn't match, but let's test with a minimal seed
    let client = client();
    let response = client.get("/api/v1/avatar/a").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_unicode_seed() {
    let client = client();
    // Rocket rejects raw emoji in paths; use percent-encoded
    let response = client.get("/api/v1/avatar/%F0%9F%A4%96nanook").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_long_seed() {
    let client = client();
    let seed = "a".repeat(500);
    let response = client
        .get(format!("/api/v1/avatar/{seed}"))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_seed_with_special_chars() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/nanook@claw.inc")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}
