# Agent Avatar Generator — Design Document

> See also: [Shared Design Principles](../humans-not-required/docs/design-principles.md)

## Overview

Self-hosted avatar generation for AI agents. Deterministic rendering from agent IDs — same input always produces the same avatar. Multiple agent-optimized styles, PNG and SVG output, simple REST API.

Agents need consistent visual identities but shouldn't depend on external services like DiceBear that could disappear, change APIs, or impose restrictions.

## Auth Model: None (Stateless)

Avatar generation is a **pure function** — it transforms an input string into an image. No state, no storage, no auth needed.

### Access Rules

| Operation | Auth Required | Rationale |
|-----------|--------------|-----------|
| Generate avatar | ❌ No | Stateless transformation |
| List styles | ❌ No | Public catalog |
| View via share URL | ❌ No | Public by design |

## Core Concept: Deterministic Generation

Every avatar is derived from a **seed string** (typically an agent ID, email, or name):

1. Hash the seed with SHA-256
2. Use hash bytes to deterministically select: colors, shapes, pattern, rotation, symmetry
3. Same seed → same avatar, always

No randomness. No storage. No database.

## Styles

### 1. `geometric` (default)
Grid-based symmetric patterns using colored shapes on a background. Think GitHub identicons but more expressive. 5×5 grid with vertical symmetry.

### 2. `rings`
Concentric rings with varying colors, widths, and gaps. Abstract and distinctive. Good for small sizes.

### 3. `robot`
Procedurally generated robot faces. Head shape, eyes, mouth, antenna, body color — all derived from the seed. Agent-themed.

### 4. `blockies`
Ethereum-style block avatars. 8×8 pixel grid with 2-3 colors. Compact and recognizable.

### 5. `gradient`
Two-color radial or linear gradient with an overlay shape (circle, diamond, hexagon). Minimal and modern.

## API Design

### Generate Avatar

```
GET /api/v1/avatar/{seed}
GET /api/v1/avatar/{seed}.png
GET /api/v1/avatar/{seed}.svg
```

Query parameters:
- `style` — `geometric` | `rings` | `robot` | `blockies` | `gradient` (default: `geometric`)
- `size` — pixel dimensions, 16-1024 (default: 256)
- `format` — `png` | `svg` (default: `png`, also inferred from extension)
- `background` — hex color override (default: derived from seed)

Response: image bytes with appropriate Content-Type.

### List Styles

```
GET /api/v1/styles
```

Returns available styles with descriptions and sample seeds.

### Batch Generate

```
POST /api/v1/avatar/batch
```

Body:
```json
{
  "seeds": ["agent-1", "agent-2", "agent-3"],
  "style": "geometric",
  "size": 128,
  "format": "png"
}
```

Returns JSON with base64-encoded images. Max 50 per request.

### Health

```
GET /api/v1/health
```

## Share URL Strategy

Share URLs are stateless and self-contained:

```
/avatar/view/{seed}?style=geometric&size=256
```

Renders a preview page with the avatar and download button. The avatar is regenerated on-the-fly.

## Rate Limiting

IP-based, 200 requests/minute. Generous because generation is CPU-bound but fast.

## Frontend

Simple React UI:
- Text input for seed
- Style selector (visual preview of each style)
- Size slider
- Format toggle (PNG/SVG)
- Live preview as you type
- Download button
- Copy share URL button
- Gallery view: enter multiple seeds, see them all

## Tech Stack

- **Backend:** Rust (Rocket), no database needed
- **Image generation:** `image` crate for PNG, string templates for SVG
- **Hashing:** `sha2` for deterministic seed processing
- **Frontend:** React + Vite
- **Deployment:** Single binary, single port

## Python SDK

Zero-dependency Python client. Methods:
- `generate(seed, style, size, format)` → bytes
- `generate_svg(seed, style, size)` → string
- `generate_png(seed, style, size)` → bytes
- `styles()` → list of available styles
- `batch(seeds, style, size, format)` → list of base64 images
- `save(seed, path, style, size, format)` → writes to file

## Discovery

- `/api/v1/openapi.json` — OpenAPI 3.0 spec
- `/llms.txt` — AI-readable service description
- `/.well-known/skills/agent-avatar-generator/SKILL.md` — Skill file
