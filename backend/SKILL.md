# Agent Avatar Generator

Self-hosted deterministic avatar generation for AI agents. Part of the [Humans-Not-Required](https://github.com/Humans-Not-Required) ecosystem.

## What It Does

Generates unique, visually distinct avatars from any seed string. Deterministic: the same input always produces the same image. No accounts, no storage, no auth — just a pure function from string to image. Supports PNG, SVG, and animated GIF output.

## API

### Generate Avatar
```
GET /api/v1/avatar/{seed}?style=geometric&size=256&format=png
```

**Parameters:**
- `seed` (path, required): Any string — agent ID, email, name, UUID
- `style` (query): `geometric` | `rings` | `robot` | `blockies` | `gradient` | `initials` | `starburst` | `mosaic` | `pixel` | `sunset` | `constellation` (default: `geometric`)
- `size` (query): 16–1024 pixels (default: 256)
- `format` (query): `png` | `svg` | `gif` (default: `png`)
- `background` (query): Hex color override (e.g., `ff0000`)
- `theme` (query): Color theme — `warm` | `cool` | `ocean` | `forest` | `sunset` | `neon` | `pastel` | `monochrome` | `earth` (optional)
- `frames` (query): Animation frames 2–30 (GIF only, default: 10)
- `delay` (query): Frame delay in centiseconds 1–100 (GIF only, default: 8 = 80ms/frame)

**Response:** Image bytes (`image/png`, `image/svg+xml`, or `image/gif`)

### Generate Animated GIF
```
GET /api/v1/avatar/{seed}?format=gif&frames=10&delay=8
```

Each style has a unique animation:
- `rings` — pulsating ring widths
- `robot` — eye blink animation
- `starburst` — rotation
- `gradient` — angle rotation
- `pixel` — color cycling
- `sunset` — sun movement + color shift
- `constellation` — node pulse
- Others — brightness pulse

**Response headers:** `X-Frame-Count`, `X-Generation-Time-Ms`

### Batch Generate
```
POST /api/v1/avatar/batch
Content-Type: application/json

{"seeds": ["agent-1", "agent-2"], "style": "robot", "size": 128, "format": "png"}
```

For GIF: add `"frames": 10, "delay": 8`

**Response:** JSON with base64-encoded images (max 50 per request)

### List Styles
```
GET /api/v1/styles
```

### List Themes
```
GET /api/v1/themes
```

### Gallery ZIP Download
```
POST /api/v1/avatar/gallery/zip
Content-Type: application/json

{"seeds": ["agent-1", "agent-2"], "style": "all", "size": 256, "format": "png"}
```

For GIF: add `"frames": 10, "delay": 8`

**Response:** ZIP file containing avatar images. Use `style: "all"` to include every style for each seed. Max 50 seeds.

### Health Check
```
GET /api/v1/health
```

## Styles

| Style | Description | GIF Animation |
|-------|-------------|---------------|
| `geometric` | 5×5 symmetric grid identicon (default) | Brightness pulse |
| `rings` | Concentric colored rings | Pulsating ring widths |
| `robot` | Procedural robot faces (4 head shapes, 6 antenna styles, eye glow, collars, shoulder pads, emblems) | Eye blink |
| `blockies` | 8×8 Ethereum-style block grid | Brightness pulse |
| `gradient` | Two-color gradient with shape overlay | Angle rotation |
| `initials` | Letter-based avatar with 1-2 initials | Brightness pulse |
| `starburst` | Radial rays with fading edges | Rotation |
| `mosaic` | 6×6 grid of shapes with harmonious colors | Brightness pulse |
| `pixel` | Retro pixel art creatures (space-invader style) | Color cycling |
| `sunset` | Layered horizon bands with harmonious colors and sun glow | Sun movement + color shift |
| `constellation` | Network graph — 7-14 nodes connected by edges, hub-and-spoke topology | Node pulse |

## Color Themes

Apply a color theme to any style with the `theme` parameter:

| Theme | Description |
|-------|-------------|
| `warm` | Reds, oranges, and yellows |
| `cool` | Blues and cyans |
| `ocean` | Teals and deep blues |
| `forest` | Greens and earth tones |
| `sunset` | Pinks, reds, and oranges |
| `neon` | High-saturation on dark background |
| `pastel` | Soft, light colors |
| `monochrome` | Grayscale only |
| `earth` | Browns, tans, and muted tones |

## Example Usage

```bash
# Generate a PNG avatar
curl -o avatar.png "https://your-server/api/v1/avatar/nanook?style=robot&size=256"

# Generate with a theme
curl -o avatar.png "https://your-server/api/v1/avatar/nanook?style=geometric&theme=neon"

# Generate SVG
curl -o avatar.svg "https://your-server/api/v1/avatar/nanook?format=svg"

# Generate animated GIF
curl -o avatar.gif "https://your-server/api/v1/avatar/nanook?format=gif&style=robot&frames=12&delay=8"
```

## Properties
- **Deterministic:** Same seed → same avatar, always (including GIFs)
- **Stateless:** No database, no storage needed
- **No auth:** All endpoints are public
- **Self-hosted:** No external dependencies
- **Cache-friendly:** Immutable responses, far-future cache headers

## Source
https://github.com/Humans-Not-Required/agent-avatar-generator
