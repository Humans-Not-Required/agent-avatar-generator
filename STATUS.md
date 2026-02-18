# Agent Avatar Generator - Status

## Current State: MVP Complete ✅

Self-hosted deterministic avatar generation service with 5 styles, PNG/SVG output, React frontend, and Python SDK. All tests passing, zero clippy warnings, CI configured.

### What's Done

- **Core API (stateless, no auth):**
  - `GET /api/v1/avatar/{seed}` — Generate avatar with query params (style, size, format, background)
  - `GET /api/v1/styles` — List all available styles
  - `POST /api/v1/avatar/batch` — Batch generate up to 50 avatars
  - `GET /avatar/view/{seed}` — Share URL with preview page
- **5 Avatar Styles:**
  - `geometric` — 5×5 vertically symmetric grid identicon (default)
  - `rings` — Concentric colored rings
  - `robot` — Procedural robot faces with varying features
  - `blockies` — 8×8 Ethereum-style colored grid
  - `gradient` — Two-color gradient with geometric overlay shape
- **Output Formats:** PNG + SVG for all styles
- **Deterministic:** Same seed → identical output, always. SHA-256 hashing.
- **Rate Limiting:** 200 req/min per IP with headers
- **Cache Headers:** Immutable, far-future (1 year)
- **Frontend:** React + Vite SPA
  - Live preview as you type
  - Style selector with visual previews
  - Size slider, format toggle
  - Download button + copy share URL
  - Dark theme matching HNR design system
- **Discovery:** `/api/v1/openapi.json`, `/llms.txt`, `/.well-known/skills/agent-avatar-generator/SKILL.md`
- **Docker:** Multi-stage build, single port
- **CI/CD:** GitHub Actions → ghcr.io + Watchtower auto-deploy
- **Tests:** 53 Rust (16 unit + 37 HTTP integration), zero clippy warnings
- ✅ **Python SDK** — Zero-dependency client (`sdk/python/avatar_service.py`). All endpoints covered. Typed errors. Save helper. 39 integration tests.

### Tech Stack

- Rust 1.83+ / Rocket 0.5 (no database — pure stateless)
- Image generation: `image` crate (PNG), string templates (SVG)
- Hashing: `sha2` (SHA-256 for deterministic seeds)
- React 18 + Vite
- Port: external TBD, internal 8000

### What's Next

- Deploy to staging (port assignment, docker-compose on staging server)
- Test Watchtower auto-pull
- Consider adding more styles (pixel art, initials, abstract shapes)
- Performance optimization for large batch requests
- Add more robot face variations

### ⚠️ Gotchas

- `cargo` not on PATH by default — use `export PATH="$HOME/.cargo/bin:$PATH"`
- CORS wide open (all origins) — tighten for production
- No database needed — entire service is stateless
- BASE_URL defaults to `http://localhost:8000` — must be set in production
- Rate limiter state is in-memory — resets on restart
- Raw emoji in URL paths rejected by Rocket — use percent-encoding

*Last updated: 2026-02-18 02:30 UTC. 53 Rust tests + 39 Python SDK tests = 92 total. Zero clippy warnings. CI configured.*

## Incoming Directions (Work Queue)

<!-- WORK_QUEUE_DIRECTIONS_START -->
(No directions yet)
<!-- WORK_QUEUE_DIRECTIONS_END -->
