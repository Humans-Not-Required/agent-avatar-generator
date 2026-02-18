#[macro_use]
extern crate rocket;

mod avatar;
mod rate_limit;
mod routes;
mod styles;
mod theme;

use rocket::fs::{FileServer, Options};
use rocket_cors::{AllowedOrigins, CorsOptions};
use std::path::PathBuf;
use std::time::Duration;

#[launch]
fn rocket() -> _ {
    let _ = dotenvy::dotenv();

    let window_secs: u64 = std::env::var("RATE_LIMIT_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    let limiter = rate_limit::RateLimiter::new(Duration::from_secs(window_secs));

    let static_dir: PathBuf = std::env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../frontend/dist"));

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("CORS configuration failed");

    let mut build = rocket::build()
        .attach(cors)
        .manage(limiter)
        .mount(
            "/api/v1",
            routes![
                routes::health,
                routes::openapi,
                routes::llms_txt,
                routes::generate_avatar,
                routes::generate_avatar_png,
                routes::generate_avatar_svg,
                routes::list_styles,
                routes::list_themes,
                routes::batch_generate,
                routes::gallery_zip,
            ],
        )
        .mount(
            "/",
            routes![
                routes::root_llms_txt,
                routes::skills_index,
                routes::skills_skill_md,
                routes::view_avatar,
            ],
        );

    if static_dir.is_dir() {
        println!("📦 Serving frontend from: {}", static_dir.display());
        build = build
            .mount("/", FileServer::new(&static_dir, Options::Index))
            .mount("/", routes![routes::spa_fallback]);
    } else {
        println!(
            "⚠️  Frontend directory not found: {} (API-only mode)",
            static_dir.display()
        );
    }

    build
}
