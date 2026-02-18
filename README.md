# Agent Avatar Generator 🤖

Self-hosted deterministic avatar generation for AI agents. Same input always produces the same avatar — no accounts, no storage, no external dependencies.

Part of the [Humans-Not-Required](https://github.com/Humans-Not-Required) ecosystem.

## Quick Start

```bash
# Generate a PNG avatar
curl -o avatar.png "http://localhost:8000/api/v1/avatar/nanook?style=robot&size=256"

# Generate SVG
curl -o avatar.svg "http://localhost:8000/api/v1/avatar/nanook?format=svg"

# Batch generate
curl -X POST http://localhost:8000/api/v1/avatar/batch \
  -H "Content-Type: application/json" \
  -d '{"seeds": ["agent-1", "agent-2", "agent-3"], "style": "geometric"}'
```

## Styles

| Style | Description |
|-------|-------------|
| `geometric` | 5×5 symmetric grid identicon (default) |
| `rings` | Concentric colored rings |
| `robot` | Procedural robot faces |
| `blockies` | 8×8 Ethereum-style block grid |
| `gradient` | Two-color gradient with shape overlay |

## API

### Generate Avatar
```
GET /api/v1/avatar/{seed}?style=geometric&size=256&format=png&background=ff0000
```

### List Styles
```
GET /api/v1/styles
```

### Batch Generate
```
POST /api/v1/avatar/batch
{"seeds": [...], "style": "geometric", "size": 128, "format": "png"}
```

### Share URL
```
/avatar/view/{seed}?style=robot&size=256
```

## Properties

- **Deterministic**: Same seed → same avatar, always
- **Stateless**: No database, no storage needed
- **No auth**: All endpoints are public
- **Self-hosted**: No external dependencies
- **Cache-friendly**: Immutable responses with far-future cache headers
- **Rate limited**: 200 requests/minute per IP

## Python SDK

```python
from avatar_service import AvatarService

client = AvatarService("http://localhost:8000")

# Generate
png = client.generate_png("nanook", style="robot", size=256)
svg = client.generate_svg("nanook")

# Save to file
client.save("nanook", "avatar.png", style="robot")

# Batch
results = client.batch(["agent-1", "agent-2", "agent-3"])

# List styles
styles = client.styles()
```

## Running

### Docker (recommended)
```bash
docker compose up -d
```

### From source
```bash
cd backend
cargo run --release
```

## Discovery

- OpenAPI: `GET /api/v1/openapi.json`
- LLMs: `GET /llms.txt`
- Skills: `GET /.well-known/skills/agent-avatar-generator/SKILL.md`

## Tech Stack

- Rust (Rocket) backend — no database needed
- React + Vite frontend
- Docker deployment
- 53 tests (16 unit + 37 integration), zero clippy warnings

## License

MIT
