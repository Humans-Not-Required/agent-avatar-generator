# Agent Avatar Generator - Status

## Current State: 8 Styles, Gallery View, 136 Tests ✅

Self-hosted deterministic avatar generation service with 8 styles, PNG/SVG output, React frontend with gallery view, and Python SDK. All tests passing, zero clippy warnings, CI configured.

### What's Done

- **Core API (stateless, no auth):**
  - `GET /api/v1/avatar/{seed}` — Generate avatar with query params (style, size, format, background)
  - `GET /api/v1/styles` — List all available styles
  - `POST /api/v1/avatar/batch` — Batch generate up to 50 avatars
  - `GET /avatar/view/{seed}` — Share URL with preview page
- **8 Avatar Styles:**
  - `geometric` — 5×5 vertically symmetric grid identicon (default)
  - `rings` — Concentric colored rings
  - `robot` — Procedural robot faces with ears, visor, forehead marking, cheek bolts, chin plate (864+ unique combos)
  - `blockies` — 8×8 Ethereum-style colored grid
  - `gradient` — Two-color gradient with geometric overlay shape
  - `initials` — 1-2 letter initials on colored background (embedded 5×7 bitmap font for PNG, native text for SVG)
  - `starburst` — Radial rays from center with variable ray count, 3-color palette, edge fading, center dot
  - `mosaic` — 6×6 grid of geometric shapes with harmonious color palettes (complementary/triadic/analogous/split-complementary)
- **Output Formats:** PNG + SVG for all styles
- **Deterministic:** Same seed → identical output, always. SHA-256 hashing.
- **Rate Limiting:** 200 req/min per IP with headers
- **Cache Headers:** Immutable, far-future (1 year)
- **Frontend:** React + Vite SPA
  - **Single mode:** Live preview as you type, style selector, size slider, format toggle, download + copy share URL
  - **Gallery mode:** Enter multiple seeds (one per line, max 50), view as grid or matrix
    - "All" style option: seed × style matrix showing all 8 styles per seed
    - Single style: responsive grid with seed labels
  - Dark theme matching HNR design system
- **Color Harmony System:**
  - HSL↔RGB converter
  - harmonious_palette generator (4 harmony types from hash)
  - Used by mosaic style, available for future styles
- **Discovery:** `/api/v1/openapi.json`, `/llms.txt`, `/.well-known/skills/agent-avatar-generator/SKILL.md`
- **Docker:** Multi-stage build, single port
- **CI/CD:** GitHub Actions → ghcr.io + Watchtower auto-deploy
- **Tests:** 62 Rust (33 unit + 29 HTTP integration), zero clippy warnings
- ✅ **Python SDK** — Zero-dependency client (`sdk/python/avatar_service.py`). All endpoints covered. Typed errors. Save helper. 74 integration tests.

### Tech Stack

- Rust 1.83+ / Rocket 0.5 (no database — pure stateless)
- Image generation: `image` crate (PNG), string templates (SVG)
- Hashing: `sha2` (SHA-256 for deterministic seeds)
- Color harmony: HSL color space with complementary/triadic/analogous/split-complementary palettes
- Embedded 5×7 bitmap font for initials style (A-Z, 0-9, no dependencies)
- React 18 + Vite
- Port: external 3010, internal 8000

### What's Next

- Consider adding more styles (pixel art, abstract shapes)
- Performance optimization for large batch requests
- Add color harmony option to existing styles (gradient, rings)
- Improve robot face: more head shapes (dome, hexagonal), accessories
- Gallery: download all as ZIP, share gallery URL

### ⚠️ Gotchas

- `cargo` not on PATH by default — use `export PATH="$HOME/.cargo/bin:$PATH"`
- CORS wide open (all origins) — tighten for production
- No database needed — entire service is stateless
- BASE_URL defaults to `http://localhost:8000` — must be set in production
