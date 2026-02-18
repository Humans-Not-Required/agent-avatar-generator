# Agent Avatar Generator - Status

## Current State: 7 Styles, Tests Expanding ✅

Self-hosted deterministic avatar generation service with 7 styles, PNG/SVG output, React frontend, and Python SDK. All tests passing, zero clippy warnings, CI configured.

### What's Done

- **Core API (stateless, no auth):**
  - `GET /api/v1/avatar/{seed}` — Generate avatar with query params (style, size, format, background)
  - `GET /api/v1/styles` — List all available styles
  - `POST /api/v1/avatar/batch` — Batch generate up to 50 avatars
  - `GET /avatar/view/{seed}` — Share URL with preview page
- **7 Avatar Styles:**
  - `geometric` — 5×5 vertically symmetric grid identicon (default)
  - `rings` — Concentric colored rings
  - `robot` — Procedural robot faces with varying features
  - `blockies` — 8×8 Ethereum-style colored grid
  - `gradient` — Two-color gradient with geometric overlay shape
  - `initials` — 1-2 letter initials on colored background (embedded 5×7 bitmap font for PNG, native text for SVG)
  - `starburst` — Radial rays from center with variable ray count, 3-color palette, edge fading, center dot
- **Output Formats:** PNG + SVG for all styles
- **Deterministic:** Same seed → identical output, always. SHA-256 hashing.
- **Rate Limiting:** 200 req/min per IP with headers
- **Cache Headers:** Immutable, far-future (1 year)
- **Frontend:** React + Vite SPA
  - Live preview as you type
  - Style selector with visual previews (all 7 styles)
  - Size slider, format toggle
  - Download button + copy share URL
  - Dark theme matching HNR design system
- **Discovery:** `/api/v1/openapi.json`, `/llms.txt`, `/.well-known/skills/agent-avatar-generator/SKILL.md`
- **Docker:** Multi-stage build, single port
- **CI/CD:** GitHub Actions → ghcr.io + Watchtower auto-deploy
- **Tests:** 83 Rust (33 unit + 50 HTTP integration), zero clippy warnings
- ✅ **Python SDK** — Zero-dependency client (`sdk/python/avatar_service.py`). All endpoints covered. Typed errors. Save helper. 65 integration tests.

### Tech Stack

- Rust 1.83+ / Rocket 0.5 (no database — pure stateless)
- Image generation: `image` crate (PNG), string templates (SVG)
- Hashing: `sha2` (SHA-256 for deterministic seeds)
- Embedded 5×7 bitmap font for initials style (A-Z, 0-9, no dependencies)
- React 18 + Vite
- Port: external 3010, internal 8000

### What's Next

- Consider adding more styles (pixel art, mosaic, abstract shapes)
- Performance optimization for large batch requests
- Add more robot face variations (ears, accessories, body shapes)
- Gallery view in frontend (enter multiple seeds, see all at once)
- Color harmony improvements (complementary/triadic selection)

### ⚠️ Gotchas

- `cargo` not on PATH by default — use `export PATH="$HOME/.cargo/bin:$PATH"`
- CORS wide open (all origins) — tighten for production
- No database needed — entire service is stateless
- BASE_URL defaults to `http://localhost:8000` — must be set in production
- Rate limiter state is in-memory — resets on restart
- Raw emoji in URL paths rejected by Rocket — use percent-encoding
- Initials SVG uses system fonts (text element) — may render differently across systems

*Last updated: 2026-02-18 02:55 UTC. 83 Rust + 65 Python SDK = 148 total tests. Zero clippy warnings. CI green.*

## Incoming Directions (Work Queue)

<!-- WORK_QUEUE_DIRECTIONS_START -->
(No directions yet)
<!-- WORK_QUEUE_DIRECTIONS_END -->
