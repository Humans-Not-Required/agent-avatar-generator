# Stage 1: Build frontend
FROM node:22-slim AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1-slim-bookworm AS backend-builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY backend/ ./

# Build dependencies first (cacheable layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/agent-avatar-generator /usr/local/bin/agent-avatar-generator

# Stage 3: Runtime
FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/Humans-Not-Required/agent-avatar-generator"
LABEL org.opencontainers.image.description="Self-hosted avatar generation for AI agents"
LABEL org.opencontainers.image.licenses="MIT"

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash appuser
WORKDIR /app

COPY --from=backend-builder /usr/local/bin/agent-avatar-generator /app/agent-avatar-generator
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000
ENV STATIC_DIR=/app/frontend/dist

USER appuser
EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:8000/api/v1/health || exit 1

CMD ["./agent-avatar-generator"]
