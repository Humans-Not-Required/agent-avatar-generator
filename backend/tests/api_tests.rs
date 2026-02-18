use base64::Engine;
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
    for style in &["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst"] {
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
    for style in &["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst"] {
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
    assert_eq!(styles.len(), 10);
    let names: Vec<&str> = styles.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"geometric"));
    assert!(names.contains(&"rings"));
    assert!(names.contains(&"robot"));
    assert!(names.contains(&"blockies"));
    assert!(names.contains(&"gradient"));
    assert!(names.contains(&"initials"));
    assert!(names.contains(&"starburst"));
    assert!(names.contains(&"mosaic"));
    assert!(names.contains(&"pixel"));
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

// ── Initials Style ──

#[test]
fn test_initials_png() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/Nanook?style=initials")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::PNG));
    let bytes = response.into_bytes().unwrap();
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_initials_svg() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/Nanook?style=initials&format=svg")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::SVG));
    let body = response.into_string().unwrap();
    assert!(body.contains("<text"));
    assert!(body.contains("NA")); // First two alphanumeric chars uppercased
}

#[test]
fn test_initials_deterministic_http() {
    let client = client();
    let r1 = client
        .get("/api/v1/avatar/agent42?style=initials")
        .dispatch()
        .into_bytes()
        .unwrap();
    let r2 = client
        .get("/api/v1/avatar/agent42?style=initials")
        .dispatch()
        .into_bytes()
        .unwrap();
    assert_eq!(r1, r2, "Same seed should produce identical avatar");
}

#[test]
fn test_initials_with_bg() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/Test?style=initials&background=ff0000")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_initials_numeric_seed() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/42?style=initials&format=svg")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("42")); // Should show "42"
}

#[test]
fn test_initials_single_char() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/X?style=initials&format=svg")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("X")); // Single char
}

#[test]
fn test_initials_batch() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["Alice", "Bob", "Charlie"], "style": "initials", "size": 64}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert_eq!(body["avatars"].as_array().unwrap().len(), 3);
}

// ── Starburst Style ──

#[test]
fn test_starburst_png() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/star?style=starburst")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::PNG));
    let bytes = response.into_bytes().unwrap();
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_starburst_svg() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/star?style=starburst&format=svg")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::SVG));
    let body = response.into_string().unwrap();
    assert!(body.contains("<path")); // Ray paths
    assert!(body.contains("<circle")); // Center dot
}

#[test]
fn test_starburst_deterministic_http() {
    let client = client();
    let r1 = client
        .get("/api/v1/avatar/burst?style=starburst")
        .dispatch()
        .into_bytes()
        .unwrap();
    let r2 = client
        .get("/api/v1/avatar/burst?style=starburst")
        .dispatch()
        .into_bytes()
        .unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn test_starburst_with_bg() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/sun?style=starburst&background=000033")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_starburst_custom_size() {
    let client = client();
    let response = client
        .get("/api/v1/avatar/star?style=starburst&size=512")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::PNG));
}

#[test]
fn test_starburst_batch() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["sun", "moon", "star"], "style": "starburst", "size": 64, "format": "svg"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let avatars = body["avatars"].as_array().unwrap();
    assert_eq!(avatars.len(), 3);
    for a in avatars {
        assert!(a["error"].is_null());
    }
}

// ── Robot Variations ──

#[test]
fn test_robot_many_seeds_png() {
    // Generate robot avatars for many seeds to exercise all ear/visor/forehead/chin variants
    let client = client();
    let seeds = vec![
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
        "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi",
        "rho", "sigma", "tau", "upsilon", "phi", "chi", "psi", "omega",
    ];
    for seed in &seeds {
        let url = format!("/api/v1/avatar/{}?style=robot&size=128", seed);
        let response = client.get(&url).dispatch();
        assert_eq!(response.status(), Status::Ok, "Failed for seed: {}", seed);
        assert_eq!(response.content_type().unwrap(), ContentType::PNG);
        let bytes = response.into_bytes().unwrap();
        assert!(bytes.len() > 100, "Robot PNG too small for seed: {}", seed);
    }
}

#[test]
fn test_robot_many_seeds_svg() {
    // Generate robot SVGs for many seeds to exercise all feature variants
    let client = client();
    let seeds = vec![
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
        "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi",
    ];
    for seed in &seeds {
        let url = format!("/api/v1/avatar/{}?style=robot&size=128&format=svg", seed);
        let response = client.get(&url).dispatch();
        assert_eq!(response.status(), Status::Ok, "Failed for seed: {}", seed);
        let body = response.into_string().unwrap();
        assert!(body.contains("<svg"), "No SVG tag for seed: {}", seed);
        assert!(body.contains("</svg>"), "Unclosed SVG for seed: {}", seed);
    }
}

#[test]
fn test_robot_deterministic_features() {
    // Same seed should produce identical robot avatar bytes
    let client = client();
    let response1 = client.get("/api/v1/avatar/robot-test-42?style=robot&size=256").dispatch();
    let bytes1 = response1.into_bytes().unwrap();
    let response2 = client.get("/api/v1/avatar/robot-test-42?style=robot&size=256").dispatch();
    let bytes2 = response2.into_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "Robot avatar not deterministic");
}

#[test]
fn test_robot_batch_variety() {
    // Batch generate robots and verify they're all different
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["robot-a", "robot-b", "robot-c", "robot-d", "robot-e"], "style": "robot", "size": 64}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let avatars = body["avatars"].as_array().unwrap();
    assert_eq!(avatars.len(), 5);
    // Collect base64 data and verify they're unique
    let data: Vec<&str> = avatars.iter().map(|a| a["data"].as_str().unwrap()).collect();
    let unique: std::collections::HashSet<&str> = data.iter().copied().collect();
    assert_eq!(unique.len(), 5, "Not all robot avatars are unique");
}

#[test]
fn test_robot_small_size() {
    // Robot should render even at very small sizes without panicking
    let client = client();
    let response = client.get("/api/v1/avatar/tiny-robot?style=robot&size=16").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    assert!(bytes.len() > 50, "Robot PNG too small at 16px");
}

#[test]
fn test_robot_large_size() {
    // Robot should render at max size
    let client = client();
    let response = client.get("/api/v1/avatar/big-robot?style=robot&size=512").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    assert!(bytes.len() > 1000, "Robot PNG too small at 512px");
}

// ── Mosaic Style ──

#[test]
fn test_mosaic_png() {
    let client = client();
    let response = client.get("/api/v1/avatar/mosaic-test?style=mosaic&size=256").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::PNG);
    let bytes = response.into_bytes().unwrap();
    assert!(bytes.len() > 200);
}

#[test]
fn test_mosaic_svg() {
    let client = client();
    let response = client.get("/api/v1/avatar/mosaic-test?style=mosaic&size=256&format=svg").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().unwrap();
    assert!(body.contains("<svg"));
    assert!(body.contains("</svg>"));
}

#[test]
fn test_mosaic_deterministic() {
    let client = client();
    let response1 = client.get("/api/v1/avatar/mosaic-det?style=mosaic&size=128").dispatch();
    let bytes1 = response1.into_bytes().unwrap();
    let response2 = client.get("/api/v1/avatar/mosaic-det?style=mosaic&size=128").dispatch();
    let bytes2 = response2.into_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "Mosaic should be deterministic");
}

#[test]
fn test_mosaic_with_bg() {
    let client = client();
    let response = client.get("/api/v1/avatar/mosaic-bg?style=mosaic&size=128&background=ff0000").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_mosaic_batch() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["m1", "m2", "m3"], "style": "mosaic", "size": 64}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let avatars = body["avatars"].as_array().unwrap();
    assert_eq!(avatars.len(), 3);
    for a in avatars {
        assert!(a["error"].is_null());
    }
}

#[test]
fn test_mosaic_small() {
    let client = client();
    let response = client.get("/api/v1/avatar/tiny-mosaic?style=mosaic&size=16").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

// ── Pixel Art Style ──

#[test]
fn test_pixel_png() {
    let client = client();
    let response = client.get("/api/v1/avatar/invader?style=pixel&size=256").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::PNG);
    let bytes = response.into_bytes().unwrap();
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_pixel_svg() {
    let client = client();
    let response = client.get("/api/v1/avatar/invader?style=pixel&size=256&format=svg").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::SVG);
    let body = response.into_string().unwrap();
    assert!(body.contains("<svg"));
    assert!(body.contains("<rect")); // Should have pixel rects
}

#[test]
fn test_pixel_deterministic_http() {
    let client = client();
    let r1 = client.get("/api/v1/avatar/creature?style=pixel&size=128").dispatch().into_bytes().unwrap();
    let r2 = client.get("/api/v1/avatar/creature?style=pixel&size=128").dispatch().into_bytes().unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn test_pixel_with_bg() {
    let client = client();
    let response = client.get("/api/v1/avatar/alien?style=pixel&background=000033").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_pixel_batch() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds": ["inv1", "inv2", "inv3"], "style": "pixel", "size": 64}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let avatars = body["avatars"].as_array().unwrap();
    assert_eq!(avatars.len(), 3);
    // All should be unique
    let data: Vec<&str> = avatars.iter().map(|a| a["data"].as_str().unwrap()).collect();
    let unique: std::collections::HashSet<&str> = data.iter().copied().collect();
    assert_eq!(unique.len(), 3, "Not all pixel avatars are unique");
}

#[test]
fn test_pixel_small() {
    let client = client();
    let response = client.get("/api/v1/avatar/tiny-pixel?style=pixel&size=16").dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_pixel_custom_size() {
    let client = client();
    let response = client.get("/api/v1/avatar/big-pixel?style=pixel&size=512").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::PNG);
}

// ── Gallery ZIP ──

#[test]
fn test_gallery_zip_single_seed() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["nanook"]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::ZIP));
    let bytes = response.into_bytes().unwrap();
    // ZIP magic bytes: PK\x03\x04
    assert!(bytes.len() > 4);
    assert_eq!(&bytes[..2], b"PK");
}

#[test]
fn test_gallery_zip_multiple_seeds() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["alice","bob","charlie"],"style":"geometric"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::ZIP));
    let bytes = response.into_bytes().unwrap();
    // Verify it's a valid ZIP with 3 entries
    let reader = std::io::Cursor::new(bytes);
    let zip = zip::ZipArchive::new(reader).unwrap();
    assert_eq!(zip.len(), 3);
}

#[test]
fn test_gallery_zip_svg_format() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["nanook"],"format":"svg"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    assert_eq!(zip.len(), 1);
    // Check filename has .svg extension
    let file = zip.by_index(0).unwrap();
    assert!(file.name().ends_with(".svg"));
}

#[test]
fn test_gallery_zip_all_styles() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["nanook"],"style":"all"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    let reader = std::io::Cursor::new(bytes);
    let zip = zip::ZipArchive::new(reader).unwrap();
    // Should have 10 entries (one per style)
    assert_eq!(zip.len(), 10);
}

#[test]
fn test_gallery_zip_all_styles_filenames() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["test"],"style":"all","format":"png"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    let reader = std::io::Cursor::new(bytes);
    let zip = zip::ZipArchive::new(reader).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.name_for_index(i).unwrap().to_string())
        .collect();
    // Each file should be named seed_style.png
    for name in &names {
        assert!(name.starts_with("test_"), "Expected test_ prefix: {name}");
        assert!(name.ends_with(".png"), "Expected .png suffix: {name}");
    }
}

#[test]
fn test_gallery_zip_with_custom_size() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["nanook"],"size":256}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    // Verify the PNG inside is a valid image close to requested size
    let reader = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut file = zip.by_index(0).unwrap();
    let mut png_bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut png_bytes).unwrap();
    let img = image::load_from_memory(&png_bytes).unwrap();
    // Size may be slightly adjusted by grid alignment
    assert!(img.width() >= 250 && img.width() <= 260, "width {} out of range", img.width());
    assert!(img.height() >= 250 && img.height() <= 260, "height {} out of range", img.height());
}

#[test]
fn test_gallery_zip_with_background() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["nanook"],"background":"ff0000"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::ZIP));
}

#[test]
fn test_gallery_zip_empty_seeds_error() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":[]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_gallery_zip_invalid_style_error() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["test"],"style":"nonexistent"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_gallery_zip_invalid_format_error() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["test"],"format":"gif"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn test_gallery_zip_content_disposition() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["test"]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let cd = response.headers().get_one("Content-Disposition").unwrap();
    assert!(cd.contains("avatars.zip"));
}

#[test]
fn test_gallery_zip_deterministic() {
    let client = client();
    let body = r#"{"seeds":["alice","bob"],"style":"robot"}"#;
    let r1 = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body)
        .dispatch();
    let bytes1 = r1.into_bytes().unwrap();
    let r2 = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body)
        .dispatch();
    let bytes2 = r2.into_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "ZIP should be deterministic for same input");
}

#[test]
fn test_gallery_zip_multi_seed_all_styles() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["alice","bob"],"style":"all"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    let reader = std::io::Cursor::new(bytes);
    let zip = zip::ZipArchive::new(reader).unwrap();
    // 2 seeds × 10 styles = 20 entries
    assert_eq!(zip.len(), 20);
}

// ── Themes ──

#[test]
fn test_list_themes() {
    let client = client();
    let response = client.get("/api/v1/themes").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let themes = body.as_array().unwrap();
    assert_eq!(themes.len(), 9);
    let names: Vec<&str> = themes.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"warm"));
    assert!(names.contains(&"cool"));
    assert!(names.contains(&"ocean"));
    assert!(names.contains(&"forest"));
    assert!(names.contains(&"sunset"));
    assert!(names.contains(&"neon"));
    assert!(names.contains(&"pastel"));
    assert!(names.contains(&"monochrome"));
    assert!(names.contains(&"earth"));
}

#[test]
fn test_themed_png_differs_from_unthemed() {
    let client = client();
    let unthemed = client.get("/api/v1/avatar/nanook?style=geometric").dispatch();
    let themed = client.get("/api/v1/avatar/nanook?style=geometric&theme=warm").dispatch();
    assert_eq!(unthemed.status(), Status::Ok);
    assert_eq!(themed.status(), Status::Ok);
    let bytes1 = unthemed.into_bytes().unwrap();
    let bytes2 = themed.into_bytes().unwrap();
    assert_ne!(bytes1, bytes2, "Themed avatar should differ from unthemed");
}

#[test]
fn test_themed_svg_differs_from_unthemed() {
    let client = client();
    let unthemed = client.get("/api/v1/avatar/nanook?style=geometric&format=svg").dispatch();
    let themed = client.get("/api/v1/avatar/nanook?style=geometric&format=svg&theme=cool").dispatch();
    assert_eq!(unthemed.status(), Status::Ok);
    assert_eq!(themed.status(), Status::Ok);
    let svg1 = unthemed.into_string().unwrap();
    let svg2 = themed.into_string().unwrap();
    assert_ne!(svg1, svg2, "Themed SVG should differ from unthemed");
    assert!(svg2.starts_with("<svg"), "Themed SVG should be valid");
}

#[test]
fn test_themed_png_deterministic() {
    let client = client();
    let r1 = client.get("/api/v1/avatar/test?theme=ocean").dispatch();
    let r2 = client.get("/api/v1/avatar/test?theme=ocean").dispatch();
    assert_eq!(r1.status(), Status::Ok);
    assert_eq!(r2.status(), Status::Ok);
    let b1 = r1.into_bytes().unwrap();
    let b2 = r2.into_bytes().unwrap();
    assert_eq!(b1, b2, "Same seed + same theme should produce identical output");
}

#[test]
fn test_different_themes_produce_different_results() {
    let client = client();
    let warm = client.get("/api/v1/avatar/test?theme=warm").dispatch();
    let cool = client.get("/api/v1/avatar/test?theme=cool").dispatch();
    assert_eq!(warm.status(), Status::Ok);
    assert_eq!(cool.status(), Status::Ok);
    let b1 = warm.into_bytes().unwrap();
    let b2 = cool.into_bytes().unwrap();
    assert_ne!(b1, b2, "Different themes should produce different output");
}

#[test]
fn test_all_themes_valid_png() {
    let client = client();
    for theme in &["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"] {
        let url = format!("/api/v1/avatar/test?theme={theme}");
        let response = client.get(&url).dispatch();
        assert_eq!(response.status(), Status::Ok, "Theme {theme} should produce valid response");
        let bytes = response.into_bytes().unwrap();
        assert!(!bytes.is_empty(), "Theme {theme} should produce non-empty PNG");
        // Verify PNG magic bytes
        assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47], "Theme {theme} should produce valid PNG");
    }
}

#[test]
fn test_all_themes_valid_svg() {
    let client = client();
    for theme in &["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"] {
        let url = format!("/api/v1/avatar/test?format=svg&theme={theme}");
        let response = client.get(&url).dispatch();
        assert_eq!(response.status(), Status::Ok, "Theme {theme} SVG should work");
        let svg = response.into_string().unwrap();
        assert!(svg.starts_with("<svg"), "Theme {theme} should produce valid SVG");
    }
}

#[test]
fn test_all_themes_all_styles_png() {
    let client = client();
    for theme in &["warm", "cool", "neon", "monochrome"] {
        for style in &["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"] {
            let url = format!("/api/v1/avatar/test?style={style}&theme={theme}");
            let response = client.get(&url).dispatch();
            assert_eq!(response.status(), Status::Ok, "Style {style} + theme {theme} should work");
        }
    }
}

#[test]
fn test_invalid_theme_returns_error() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?theme=rainbow").dispatch();
    assert_eq!(response.status(), Status::BadRequest);
    let body: serde_json::Value = response.into_json().unwrap();
    assert!(body["error"].as_str().unwrap().contains("Unknown theme"));
}

#[test]
fn test_theme_with_background_override() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?theme=warm&background=FF0000").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_themed_batch() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["a","b"],"theme":"ocean"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let avatars = body["avatars"].as_array().unwrap();
    assert_eq!(avatars.len(), 2);
    assert!(!avatars[0]["data"].as_str().unwrap().is_empty());
}

#[test]
fn test_themed_batch_svg() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["a"],"format":"svg","theme":"monochrome"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    let data = body["avatars"][0]["data"].as_str().unwrap();
    // Decode base64 and check it's SVG
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data).unwrap();
    let svg = String::from_utf8(decoded).unwrap();
    assert!(svg.starts_with("<svg"));
}

#[test]
fn test_themed_gallery_zip() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["test"],"theme":"forest"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    let reader = std::io::Cursor::new(bytes);
    let zip = zip::ZipArchive::new(reader).unwrap();
    assert_eq!(zip.len(), 1);
}

#[test]
fn test_neon_theme_dark_bg_png() {
    let client = client();
    let response = client.get("/api/v1/avatar/test?style=geometric&theme=neon").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let bytes = response.into_bytes().unwrap();
    // Just verify it produces a valid PNG
    assert_eq!(&bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

// ── Performance: Timing Headers ──

#[test]
fn test_png_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), "Should have X-Generation-Time-Ms header");
    let ms: f64 = header.unwrap().parse().expect("Should be a valid f64");
    assert!(ms >= 0.0, "Generation time should be non-negative");
    assert!(ms < 10000.0, "Generation time should be reasonable (<10s)");
}

#[test]
fn test_svg_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test?format=svg").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), "Should have X-Generation-Time-Ms header");
    let ms: f64 = header.unwrap().parse().expect("Should be a valid f64");
    assert!(ms >= 0.0);
}

#[test]
fn test_themed_png_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test?theme=warm").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), "Themed PNG should have timing header");
}

#[test]
fn test_themed_svg_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test?format=svg&theme=cool").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), "Themed SVG should have timing header");
}

#[test]
fn test_png_extension_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test.png").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), ".png extension route should have timing header");
}

#[test]
fn test_svg_extension_has_generation_time_header() {
    let client = client();
    let response = client.get("/api/v1/avatar/timing-test.svg").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(header.is_some(), ".svg extension route should have timing header");
}

#[test]
fn test_all_styles_have_generation_time() {
    let client = client();
    let styles = ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"];
    for style in &styles {
        let response = client.get(format!("/api/v1/avatar/perf-test?style={style}")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let header = response.headers().get_one("X-Generation-Time-Ms");
        assert!(header.is_some(), "Style {style} should have timing header");
        let ms: f64 = header.unwrap().parse().expect("Should be a valid f64");
        assert!(ms >= 0.0, "Style {style} timing should be non-negative");
    }
}

// ── Performance: Parallel Batch ──

#[test]
fn test_batch_response_has_timing() {
    let client = client();
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["a","b","c"]}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body: serde_json::Value = response.into_json().unwrap();
    assert!(body["generation_ms"].is_f64(), "Batch response should include generation_ms");
    assert!(body["generation_ms"].as_f64().unwrap() >= 0.0);
    assert_eq!(body["count"], 3, "Batch response should include count");
}

#[test]
fn test_batch_parallel_produces_same_results() {
    // Verify parallel batch produces identical results to sequential single requests
    let client = client();
    let seeds = vec!["parallel-a", "parallel-b", "parallel-c", "parallel-d", "parallel-e"];

    // Get batch results
    let batch_body = serde_json::json!({"seeds": seeds, "style": "geometric", "size": 64});
    let batch_response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(batch_body.to_string())
        .dispatch();
    assert_eq!(batch_response.status(), Status::Ok);
    let batch_json: serde_json::Value = batch_response.into_json().unwrap();
    let batch_avatars = batch_json["avatars"].as_array().unwrap();

    // Get individual results
    for (i, seed) in seeds.iter().enumerate() {
        let individual = client
            .get(format!("/api/v1/avatar/{seed}?style=geometric&size=64"))
            .dispatch();
        assert_eq!(individual.status(), Status::Ok);
        let individual_bytes = individual.into_bytes().unwrap();
        let individual_b64 = base64::engine::general_purpose::STANDARD.encode(&individual_bytes);

        assert_eq!(
            batch_avatars[i]["data"].as_str().unwrap(),
            individual_b64,
            "Batch result for seed '{seed}' should match individual generation"
        );
    }
}

#[test]
fn test_batch_50_seeds_with_timing() {
    let client = client();
    let seeds: Vec<String> = (0..50).map(|i| format!("perf-seed-{i}")).collect();
    let body = serde_json::json!({"seeds": seeds, "size": 64});
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let json: serde_json::Value = response.into_json().unwrap();
    assert_eq!(json["count"], 50);
    assert!(json["generation_ms"].as_f64().unwrap() > 0.0, "50-seed batch should take measurable time");
    assert_eq!(json["avatars"].as_array().unwrap().len(), 50);
}

#[test]
fn test_batch_themed_parallel() {
    let client = client();
    let body = serde_json::json!({"seeds": ["t1","t2","t3","t4","t5"], "theme": "ocean", "size": 64});
    let response = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let json: serde_json::Value = response.into_json().unwrap();
    assert_eq!(json["count"], 5);
    assert!(json["generation_ms"].is_f64());
    // Verify themed results differ from unthemed
    let unthemed_body = serde_json::json!({"seeds": ["t1","t2","t3","t4","t5"], "size": 64});
    let unthemed_resp = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(unthemed_body.to_string())
        .dispatch();
    let unthemed_json: serde_json::Value = unthemed_resp.into_json().unwrap();
    // At least one avatar should differ
    let themed_avatars = json["avatars"].as_array().unwrap();
    let unthemed_avatars = unthemed_json["avatars"].as_array().unwrap();
    let any_different = themed_avatars.iter().zip(unthemed_avatars.iter())
        .any(|(a, b)| a["data"] != b["data"]);
    assert!(any_different, "Themed batch should produce different results than unthemed");
}

// ── Performance: Gallery ZIP Timing ──

#[test]
fn test_gallery_zip_has_timing_headers() {
    let client = client();
    let body = serde_json::json!({"seeds": ["zip-a", "zip-b"], "size": 64});
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let gen_header = response.headers().get_one("X-Generation-Time-Ms");
    assert!(gen_header.is_some(), "ZIP should have X-Generation-Time-Ms header");
    let ms: f64 = gen_header.unwrap().parse().unwrap();
    assert!(ms >= 0.0);
    let count_header = response.headers().get_one("X-Avatar-Count");
    assert!(count_header.is_some(), "ZIP should have X-Avatar-Count header");
    assert_eq!(count_header.unwrap(), "2");
}

#[test]
fn test_gallery_zip_all_styles_has_correct_count() {
    let client = client();
    let body = serde_json::json!({"seeds": ["count-a", "count-b", "count-c"], "style": "all", "size": 32});
    let response = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let count_header = response.headers().get_one("X-Avatar-Count");
    assert!(count_header.is_some());
    // 3 seeds × 10 styles = 30
    assert_eq!(count_header.unwrap(), "30");
}

#[test]
fn test_gallery_zip_parallel_determinism() {
    // Generate same ZIP twice — should produce identical files
    let client = client();
    let body = serde_json::json!({"seeds": ["det-1", "det-2", "det-3"], "style": "geometric", "size": 64});

    let resp1 = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    let bytes1 = resp1.into_bytes().unwrap();

    let resp2 = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(body.to_string())
        .dispatch();
    let bytes2 = resp2.into_bytes().unwrap();

    assert_eq!(bytes1, bytes2, "Same inputs should produce identical ZIP files");
}

#[test]
fn test_monochrome_theme_desaturated_svg() {
    let client = client();
    let response = client.get("/api/v1/avatar/colorful?style=blockies&format=svg&theme=monochrome").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let svg = response.into_string().unwrap();
    assert!(svg.starts_with("<svg"));
    // Check that colors are desaturated (R≈G≈B in hex values)
    // Find hex colors and verify they're near-gray
    let mut found_color = false;
    for part in svg.split('#') {
        if part.len() >= 6 && part[..6].chars().all(|c| c.is_ascii_hexdigit()) {
            let hex = &part[..6];
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
            // Monochrome: channels should be close to each other
            let max_diff = (r as i16 - g as i16).unsigned_abs()
                .max((g as i16 - b as i16).unsigned_abs())
                .max((r as i16 - b as i16).unsigned_abs());
            assert!(max_diff < 20, "Monochrome color #{hex} should be near-gray (max channel diff: {max_diff})");
            found_color = true;
        }
    }
    assert!(found_color, "Should have found at least one color in SVG");
}

// ── Theme Comparison Tests ──

#[test]
fn test_all_themes_produce_different_pngs() {
    let client = client();
    let themes = ["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"];
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();

    // Get unthemed version
    let resp = client.get("/api/v1/avatar/compare-test?style=geometric&size=64").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    images.push(("none".to_string(), resp.into_bytes().unwrap()));

    // Get each themed version
    for theme in &themes {
        let resp = client.get(format!("/api/v1/avatar/compare-test?style=geometric&size=64&theme={theme}")).dispatch();
        assert_eq!(resp.status(), Status::Ok);
        images.push((theme.to_string(), resp.into_bytes().unwrap()));
    }

    // At least some themes should produce different images than unthemed
    let base = &images[0].1;
    let different_count = images[1..].iter().filter(|(_, img)| img != base).count();
    assert!(different_count >= 5, "At least 5 of 9 themes should differ from unthemed (got {different_count})");
}

#[test]
fn test_theme_comparison_batch_same_seed() {
    // Batch request with same seed, testing theme variations
    let client = client();
    let resp = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["compare-seed"],"style":"robot","size":64,"theme":"warm"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = resp.into_json().unwrap();
    assert_eq!(body["count"], 1);
    let warm_data = body["avatars"][0]["data"].as_str().unwrap().to_string();

    let resp = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["compare-seed"],"style":"robot","size":64,"theme":"cool"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = resp.into_json().unwrap();
    let cool_data = body["avatars"][0]["data"].as_str().unwrap().to_string();

    assert_ne!(warm_data, cool_data, "Warm and cool themes should produce different outputs");
}

#[test]
fn test_themed_svg_colors_differ_from_unthemed() {
    let client = client();
    let resp_plain = client.get("/api/v1/avatar/compare-svg?style=mosaic&format=svg").dispatch();
    let svg_plain = resp_plain.into_string().unwrap();

    let resp_neon = client.get("/api/v1/avatar/compare-svg?style=mosaic&format=svg&theme=neon").dispatch();
    let svg_neon = resp_neon.into_string().unwrap();

    assert_ne!(svg_plain, svg_neon, "Neon themed SVG should differ from plain");
    // Both should be valid SVGs
    assert!(svg_plain.starts_with("<svg"));
    assert!(svg_neon.starts_with("<svg"));
}

#[test]
fn test_compare_all_styles_same_seed_same_theme() {
    // Simulates the compare mode "all styles" view
    let client = client();
    let styles = ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"];
    let mut images: Vec<Vec<u8>> = Vec::new();

    for style in &styles {
        let resp = client.get(format!("/api/v1/avatar/compare-all?style={style}&size=64&theme=warm")).dispatch();
        assert_eq!(resp.status(), Status::Ok);
        images.push(resp.into_bytes().unwrap());
    }

    // All styles should be different from each other
    for i in 0..images.len() {
        for j in (i + 1)..images.len() {
            assert_ne!(images[i], images[j], "Style {} and {} should produce different images", styles[i], styles[j]);
        }
    }
}

#[test]
fn test_compare_theme_deterministic() {
    let client = client();
    // Same seed+style+theme should always produce same output
    let resp1 = client.get("/api/v1/avatar/deterministic-compare?style=pixel&size=64&theme=ocean").dispatch();
    let bytes1 = resp1.into_bytes().unwrap();

    let resp2 = client.get("/api/v1/avatar/deterministic-compare?style=pixel&size=64&theme=ocean").dispatch();
    let bytes2 = resp2.into_bytes().unwrap();

    assert_eq!(bytes1, bytes2, "Themed avatar should be deterministic");
}

#[test]
fn test_compare_zip_all_themes_for_seed() {
    // Download ZIP with all styles for a single seed — useful for comparison export
    let client = client();
    let resp = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["theme-compare"],"style":"all","size":64,"theme":"forest"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.content_type(), Some(ContentType::ZIP));

    let bytes = resp.into_bytes().unwrap();
    // ZIP header magic
    assert_eq!(&bytes[..2], &[0x50, 0x4B]);

    let reader = std::io::Cursor::new(&bytes);
    let archive = zip::ZipArchive::new(reader).unwrap();
    assert_eq!(archive.len(), 10, "Should have 10 files (1 seed × 10 styles)");
}

#[test]
fn test_gallery_zip_with_theme() {
    let client = client();
    let resp = client
        .post("/api/v1/avatar/gallery/zip")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["a","b"],"style":"geometric","size":64,"theme":"pastel"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let bytes = resp.into_bytes().unwrap();
    let reader = std::io::Cursor::new(&bytes);
    let archive = zip::ZipArchive::new(reader).unwrap();
    assert_eq!(archive.len(), 2, "Should have 2 files (2 seeds × 1 style)");
}

#[test]
fn test_theme_timing_header_present() {
    let client = client();
    let resp = client.get("/api/v1/avatar/timing-test?style=geometric&size=64&theme=earth").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let timing = resp.headers().get_one("X-Generation-Time-Ms");
    assert!(timing.is_some(), "Should have timing header");
    let ms: f64 = timing.unwrap().parse().unwrap();
    assert!(ms >= 0.0, "Timing should be non-negative");
}

#[test]
fn test_batch_multiple_seeds_same_theme() {
    let client = client();
    let resp = client
        .post("/api/v1/avatar/batch")
        .header(ContentType::JSON)
        .body(r#"{"seeds":["alice","bob","charlie","dave","eve"],"style":"rings","size":64,"theme":"sunset"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = resp.into_json().unwrap();
    assert_eq!(body["count"], 5);
    let avatars = body["avatars"].as_array().unwrap();

    // All should have data and no errors
    for avatar in avatars {
        assert!(!avatar["data"].as_str().unwrap().is_empty());
        assert!(avatar["error"].is_null());
    }

    // All should be different from each other
    let datas: Vec<&str> = avatars.iter().map(|a| a["data"].as_str().unwrap()).collect();
    for i in 0..datas.len() {
        for j in (i + 1)..datas.len() {
            assert_ne!(datas[i], datas[j], "Different seeds should produce different themed avatars");
        }
    }
}
