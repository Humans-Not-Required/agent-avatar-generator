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
    assert_eq!(styles.len(), 9);
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
