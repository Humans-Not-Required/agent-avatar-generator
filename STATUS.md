# Agent Avatar Generator - Status

## Current State: 10 Styles, 9 Color Themes, Gallery ZIP ✅

Self-hosted deterministic avatar generation service with 10 styles, 9 color themes, PNG/SVG output, React frontend with gallery view, and Python SDK. All tests passing, CI configured.

### What's Done

- **Core API (stateless, no auth):**
  - `GET /api/v1/avatar/{seed}` — Generate avatar with query params (style, size, format, background)
  - `GET /api/v1/styles` — List all available styles
  - `POST /api/v1/avatar/batch` — Batch generate up to 50 avatars
  - `GET /avatar/view/{seed}` — Share URL with preview page
- **9 Avatar Styles:**
  - `geometric` — 5×5 vertically symmetric grid identicon (default)
  - `rings` — Concentric colored rings
  - `robot` — Procedural robot faces with 4 head shapes (square, dome, hexagonal, trapezoid), antennas, ears, visor, forehead marking, cheek bolts, mouth, chin plate, collar (4 styles), shoulder pads (3 styles), chest emblem (4 styles) — 20,000+ unique combos
  - `blockies` — 8×8 Ethereum-style colored grid
  - `gradient` — Two-color gradient with geometric overlay shape
  - `initials` — 1-2 letter initials on colored background (embedded 5×7 bitmap font for PNG, native text for SVG)
  - `starburst` — Radial rays from center with variable ray count, 3-color palette, edge fading, center dot
  - `mosaic` — 6×6 grid of geometric shapes with harmonious color palettes (complementary/triadic/analogous/split-complementary)
  - `pixel` — Retro pixel art creatures with horizontal symmetry, 11×11 grid, 3-color palette, visible pixel gaps (space-invader inspired)
  - `sunset` — Layered horizon bands using harmonious color palette, wavy edges between bands, optional sun with glow effect
- **Output Formats:** PNG + SVG for all styles
- **Deterministic:** Same seed → identical output, always. SHA-256 hashing.
- **Rate Limiting:** 200 req/min per IP with headers
- **Cache Headers:** Immutable, far-future (1 year)
- **Frontend:** React + Vite SPA
  - **Single mode:** Live preview as you type, style selector, size slider, format toggle, download + copy share URL
  - **Gallery mode:** Enter multiple seeds (one per line, max 50), view as grid or matrix
    - "All" style option: seed × style matrix showing all 9 styles per seed
    - Single style: responsive grid with seed labels
  - Dark theme matching HNR design system
- **Color Harmony System:**
  - HSL↔RGB converter
  - harmonious_palette generator (4 harmony types from hash)
  - Used by mosaic style, available for future styles
- **Discovery:** `/api/v1/openapi.json`, `/llms.txt`, `/.well-known/skills/agent-avatar-generator/SKILL.md`
- **Docker:** Multi-stage build, single port
- **CI/CD:** GitHub Actions → ghcr.io + Watchtower auto-deploy
- **Gallery ZIP Download** — `POST /api/v1/avatar/gallery/zip`
  - Download multiple avatars as a ZIP file
  - Style `"all"`: generates every style for each seed (seed × 10 styles)
  - Max 50 seeds, PNG or SVG format, custom size and background
  - Frontend: ZIP download button in gallery mode
- **Color Themes** — 9 themes applied via post-processing (works with all styles):
  - `warm`, `cool`, `ocean`, `forest`, `sunset`, `neon`, `pastel`, `monochrome`, `earth`
  - `GET /api/v1/themes` to list available themes
  - `?theme=warm` query param on generate, batch, and gallery_zip endpoints
  - HSL color remapping with intelligent background/foreground detection
  - Configurable rate limit via RATE_LIMIT_MAX env var
- **Performance:** Batch and gallery ZIP use parallel generation (rayon). X-Generation-Time-Ms timing header on all avatar endpoints. X-Avatar-Count header on ZIP responses. Batch response includes `generation_ms` and `count` fields.
- **Shareable Gallery URLs:** Gallery state (seeds, style, size, theme) encoded in URL query params. Share a link that reconstructs the exact gallery view.
- **Tests:** 315 Rust (102 unit × 2 binaries + 111 HTTP integration)
- ✅ **Python SDK** — Zero-dependency client (`sdk/python/avatar_service.py`). All endpoints covered. Typed errors. Save helper. `gallery_zip()` and `gallery_zip_save()` methods. `themes()` method. `generate_timed()`, `batch_timed()`, `gallery_zip_timed()` for performance monitoring. 154 integration tests.

### Tech Stack

- Rust 1.83+ / Rocket 0.5 (no database — pure stateless)
- Image generation: `image` crate (PNG), string templates (SVG)
- Hashing: `sha2` (SHA-256 for deterministic seeds)
- Color harmony: HSL color space with complementary/triadic/analogous/split-complementary palettes
- Embedded 5×7 bitmap font for initials style (A-Z, 0-9, no dependencies)
- React 18 + Vite
- Port: external 3010, internal 8000

### What's Next

- ~~Performance optimization for large batch requests~~ ✅ Done (rayon parallelization + timing headers)
- ~~Gallery: share gallery URL~~ ✅ Done (URL params encode seeds, style, size, theme)
- ~~Color themes: user-specified palette/mood presets~~ ✅ Done (9 themes)
- Additional robot antenna styles and eye glow effects
- Animated avatar support (GIF/APNG)
- Comparison mode: side-by-side before/after theme comparison

### ⚠️ Gotchas

- `cargo` not on PATH by default — use `export PATH="$HOME/.cargo/bin:$PATH"`
- CORS wide open (all origins) — tighten for production
- No database needed — entire service is stateless
- BASE_URL defaults to `http://localhost:8000` — must be set in production
