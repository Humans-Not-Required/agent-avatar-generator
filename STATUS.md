# Agent Avatar Generator - Status

## Current State: 10 Styles, 9 Color Themes, Animated GIF, Gallery ZIP ✅

Self-hosted deterministic avatar generation service with 10 styles, 9 color themes, PNG/SVG/GIF output, React frontend with gallery view, and Python SDK. All tests passing, CI configured.

### What's Done

- **Core API (stateless, no auth):**
  - `GET /api/v1/avatar/{seed}` — Generate avatar with query params (style, size, format, background)
  - `GET /api/v1/styles` — List all available styles
  - `POST /api/v1/avatar/batch` — Batch generate up to 50 avatars
  - `GET /avatar/view/{seed}` — Share URL with preview page
- **9 Avatar Styles:**
  - `geometric` — 5×5 vertically symmetric grid identicon (default)
  - `rings` — Concentric colored rings
  - `robot` — Procedural robot faces with 4 head shapes (square, dome, hexagonal, trapezoid), 6 antenna styles (straight, V, T, lightning bolt, satellite dish, coil/spring), eye glow effects (none/subtle/bright), ears, visor, forehead marking, cheek bolts, mouth, chin plate, collar (4 styles), shoulder pads (3 styles), chest emblem (4 styles) — 50,000+ unique combos
  - `blockies` — 8×8 Ethereum-style colored grid
  - `gradient` — Two-color gradient with geometric overlay shape
  - `initials` — 1-2 letter initials on colored background (embedded 5×7 bitmap font for PNG, native text for SVG)
  - `starburst` — Radial rays from center with variable ray count, 3-color palette, edge fading, center dot
  - `mosaic` — 6×6 grid of geometric shapes with harmonious color palettes (complementary/triadic/analogous/split-complementary)
  - `pixel` — Retro pixel art creatures with horizontal symmetry, 11×11 grid, 3-color palette, visible pixel gaps (space-invader inspired)
  - `sunset` — Layered horizon bands using harmonious color palette, wavy edges between bands, optional sun with glow effect
- **Output Formats:** PNG + SVG + animated GIF for all styles
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
- **Compare Mode:** Side-by-side theme comparison UI
  - Single style: shows all 10 themes (original + 9) for a seed
  - All styles: style × theme matrix (10 styles × 10 themes = 100 avatars)
  - Shareable comparison URLs (?mode=compare&seed=X&style=Y)
  - Theme selector added to Gallery mode
- **Animated GIF Avatars** — `?format=gif&frames=10&delay=8`
  - 6 custom per-style animations: rings (pulsate), robot (eye blink), starburst (rotate), gradient (angle rotate), pixel (color cycle), sunset (sun movement + color shift)
  - Generic brightness pulse for styles without custom animation
  - GIF encoding via `gif` crate with NeuQuant color quantization (256-color)
  - Configurable frames (2-30) and delay (1-100 centiseconds)
  - `X-Frame-Count` header on GIF responses
  - Deterministic: same seed + params = identical GIF
  - Frontend: GIF format selector + frames/delay sliders with live animated preview
  - SDK: `generate_gif()` + GIF params on all methods
- **Tests:** 419 Rust (140 unit × 2 binaries + 139 HTTP integration)
- ✅ **Python SDK** — Zero-dependency client (`sdk/python/avatar_service.py`). All endpoints covered. Typed errors. Save helper. `gallery_zip()` and `gallery_zip_save()` methods. `themes()` method. `generate_timed()`, `batch_timed()`, `gallery_zip_timed()` for performance monitoring. 292 integration tests.

### Tech Stack

- Rust 1.83+ / Rocket 0.5 (no database — pure stateless)
- Image generation: `image` crate (PNG), string templates (SVG), `gif` crate with `color_quant` (animated GIF)
- Hashing: `sha2` (SHA-256 for deterministic seeds)
- Color harmony: HSL color space with complementary/triadic/analogous/split-complementary palettes
- Embedded 5×7 bitmap font for initials style (A-Z, 0-9, no dependencies)
- React 18 + Vite
- Port: external 3010, internal 8000

### What's Next

- ~~Performance optimization for large batch requests~~ ✅ Done (rayon parallelization + timing headers)
- ~~Gallery: share gallery URL~~ ✅ Done (URL params encode seeds, style, size, theme)
- ~~Color themes: user-specified palette/mood presets~~ ✅ Done (9 themes)
- ~~Comparison mode: side-by-side before/after theme comparison~~ ✅ Done (Compare mode UI)
- ~~Additional robot antenna styles and eye glow effects~~ ✅ Done (6 antenna styles + 3 glow levels)
- ~~Animated avatar support (GIF/APNG)~~ ✅ Done (GIF with per-style animations)

### 🎨 Color Palette Improvement — Proposal (awaiting Jordan decision)

**Problem:** `color_from_hash()` maps raw hash bytes directly to RGB. When R/G/B bytes land close together in value, you get muddy grays and browns. Jordan flagged this 2026-02-19.

**Root cause confirmed:** The function clamps to range 40-220 but does nothing about *saturation*. Two hash bytes at 120 and 118 → muddy gray, regardless of clamping.

**What already works well:** `harmonious_palette()` (used by mosaic) picks a hue from hash, then applies a color harmony model (complementary/triadic/analogous). These avatars always look clean.

**The fix — 3 options:**

**Option A — HSL color_from_hash (1 function, drop-in)** ← *recommended quick fix*
- Replace raw-RGB extraction with HSL: hue from 2 hash bytes, saturation locked 0.60-0.85, lightness 0.40-0.60
- All 9 styles get vivid, non-muddy colors immediately — no per-style changes
- Colors within a single avatar may be unrelated in hue (different styles pick independent hash offsets)
- ~10 lines of code change

**Option B — Avatar palette (best quality, bigger change)**
- Generate a single `AvatarPalette { primary, secondary, accent, highlight, background }` per seed using `harmonious_palette()` logic
- Update each style to pick from the palette by role (e.g., robot: body=primary, eyes=accent, visor=secondary)
- Colors within each avatar are harmonically related (analogous/triadic/etc.)
- Best visual result — avatars feel "designed," not random
- ~50-100 lines across 9 styles + new palette struct

**Option C — Hue-shifted analogous (drop-in, middle ground)**
- `color_from_hash` uses the same base hue as `harmonious_palette` but shifts by 30° per `offset/3`
- All colors within an avatar share a base hue — automatically analogous
- Drop-in like A, harmony like B (but only analogous, not complementary/triadic)
- ~15 lines of code change

**Recommendation:** Option A for speed (vivid today, no breakage), Option B for quality (worth 1-2 extra sessions, best demo impact). Option C is a good middle ground if neither extreme fits.

**Waiting on:** Jordan's preference before implementing. Any option can be done in <1 session.

### ⚠️ Gotchas

- `cargo` not on PATH by default — use `export PATH="$HOME/.cargo/bin:$PATH"`
- CORS wide open (all origins) — tighten for production
- No database needed — entire service is stateless
- BASE_URL defaults to `http://localhost:8000` — must be set in production
