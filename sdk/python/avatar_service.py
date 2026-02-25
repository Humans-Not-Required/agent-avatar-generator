"""
Agent Avatar Generator — Python SDK

Zero-dependency client for the Agent Avatar Generator API.

Usage:
    from avatar_service import AvatarService

    client = AvatarService("http://localhost:8000")
    png_bytes = client.generate("nanook", style="robot", size=256)
    client.save("nanook", "avatar.png", style="robot")
"""

import json
import os
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, quote


class AvatarServiceError(Exception):
    """Base error for avatar service operations."""
    def __init__(self, message, status_code=None, detail=None):
        super().__init__(message)
        self.status_code = status_code
        self.detail = detail


class ValidationError(AvatarServiceError):
    """Invalid parameters (400)."""
    pass


class RateLimitError(AvatarServiceError):
    """Rate limit exceeded (429)."""
    def __init__(self, message, retry_after=None, **kwargs):
        super().__init__(message, **kwargs)
        self.retry_after = retry_after


class ServerError(AvatarServiceError):
    """Server error (5xx)."""
    pass


class AvatarService:
    """Client for the Agent Avatar Generator API."""

    VALID_STYLES = {"geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset", "constellation"}
    VALID_FORMATS = {"png", "svg", "gif"}
    VALID_THEMES = {"warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth", "rose", "amber", "lime", "sky", "violet", "slate"}

    def __init__(self, base_url=None, timeout=30):
        self.base_url = (base_url or os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")).rstrip("/")
        self.timeout = timeout

    def _request(self, method, path, body=None, headers=None):
        url = f"{self.base_url}{path}"
        hdrs = headers or {}
        data = None

        if body is not None:
            data = json.dumps(body).encode("utf-8")
            hdrs["Content-Type"] = "application/json"

        req = Request(url, data=data, headers=hdrs, method=method)

        try:
            resp = urlopen(req, timeout=self.timeout)
            content_type = resp.headers.get("Content-Type", "")
            raw = resp.read()

            # Normalize header keys to lowercase for case-insensitive access
            resp_headers = {k.lower(): v for k, v in resp.headers.items()}

            if "application/json" in content_type:
                return json.loads(raw), resp.status, resp_headers
            return raw, resp.status, resp_headers

        except HTTPError as e:
            body_text = e.read().decode("utf-8", errors="replace")
            try:
                err = json.loads(body_text)
            except (json.JSONDecodeError, ValueError):
                err = {"error": body_text}

            msg = err.get("error", str(e))
            detail = err.get("detail")

            if e.code == 400:
                raise ValidationError(msg, status_code=400, detail=detail)
            if e.code == 429:
                retry = err.get("retry_after_secs")
                raise RateLimitError(msg, status_code=429, retry_after=retry)
            if e.code >= 500:
                raise ServerError(msg, status_code=e.code, detail=detail)
            raise AvatarServiceError(msg, status_code=e.code, detail=detail)

    # ── Core API ──

    def generate(self, seed, style="geometric", size=256, fmt="png", background=None, theme=None,
                 frames=None, delay=None):
        """Generate an avatar. Returns bytes (PNG/GIF) or string (SVG).

        For GIF format, use frames (2-30) and delay (1-100, in centiseconds).
        """
        params = {"style": style, "size": size, "format": fmt}
        if background:
            params["background"] = background
        if theme:
            params["theme"] = theme
        if frames is not None:
            params["frames"] = frames
        if delay is not None:
            params["delay"] = delay
        qs = urlencode(params)
        data, status, headers = self._request("GET", f"/api/v1/avatar/{quote(seed, safe='')}?{qs}")
        return data

    def generate_timed(self, seed, style="geometric", size=256, fmt="png", background=None, theme=None,
                       frames=None, delay=None):
        """Generate an avatar with timing info. Returns (data, generation_ms).

        data is bytes (PNG/GIF) or string (SVG). generation_ms is float or None.
        """
        params = {"style": style, "size": size, "format": fmt}
        if background:
            params["background"] = background
        if theme:
            params["theme"] = theme
        if frames is not None:
            params["frames"] = frames
        if delay is not None:
            params["delay"] = delay
        qs = urlencode(params)
        data, status, headers = self._request("GET", f"/api/v1/avatar/{quote(seed, safe='')}?{qs}")
        gen_ms = headers.get("x-generation-time-ms")
        if gen_ms is not None:
            gen_ms = float(gen_ms)
        return data, gen_ms

    def generate_png(self, seed, style="geometric", size=256, background=None, theme=None):
        """Generate a PNG avatar. Returns bytes."""
        return self.generate(seed, style=style, size=size, fmt="png", background=background, theme=theme)

    def generate_svg(self, seed, style="geometric", size=256, background=None, theme=None):
        """Generate an SVG avatar. Returns string."""
        data = self.generate(seed, style=style, size=size, fmt="svg", background=background, theme=theme)
        if isinstance(data, bytes):
            return data.decode("utf-8")
        return data

    def generate_gif(self, seed, style="geometric", size=256, frames=10, delay=8, background=None, theme=None):
        """Generate an animated GIF avatar. Returns bytes.

        Args:
            seed: Avatar seed string.
            style: Avatar style (each has a unique animation).
            size: Avatar size in pixels (32-512).
            frames: Number of animation frames (2-30, default 10).
            delay: Frame delay in centiseconds (1-100, default 8 = 80ms).
            background: Optional hex color.
            theme: Optional color theme.
        """
        return self.generate(seed, style=style, size=size, fmt="gif", background=background,
                             theme=theme, frames=frames, delay=delay)

    def batch(self, seeds, style="geometric", size=128, fmt="png", background=None, theme=None,
              frames=None, delay=None):
        """Batch generate avatars. Returns list of dicts with base64-encoded data.

        For GIF format, use frames (2-30) and delay (1-100).
        """
        body = {"seeds": seeds, "style": style, "size": size, "format": fmt}
        if background:
            body["background"] = background
        if theme:
            body["theme"] = theme
        if frames is not None:
            body["frames"] = frames
        if delay is not None:
            body["delay"] = delay
        data, status, headers = self._request("POST", "/api/v1/avatar/batch", body=body)
        return data["avatars"]

    def batch_timed(self, seeds, style="geometric", size=128, fmt="png", background=None, theme=None,
                    frames=None, delay=None):
        """Batch generate avatars with timing info.

        Returns dict with keys: avatars, generation_ms, count.
        """
        body = {"seeds": seeds, "style": style, "size": size, "format": fmt}
        if background:
            body["background"] = background
        if theme:
            body["theme"] = theme
        if frames is not None:
            body["frames"] = frames
        if delay is not None:
            body["delay"] = delay
        data, status, headers = self._request("POST", "/api/v1/avatar/batch", body=body)
        return data

    def gallery_zip(self, seeds, style="geometric", size=256, fmt="png", background=None, theme=None,
                    frames=None, delay=None):
        """Download multiple avatars as a ZIP file. Returns bytes.

        Args:
            seeds: List of seed strings (max 50).
            style: Avatar style or "all" for all styles.
            size: Avatar size in pixels.
            fmt: "png", "svg", or "gif".
            background: Optional hex color (e.g. "ff0000").
            theme: Optional color theme name.
            frames: For GIF: number of animation frames (2-30).
            delay: For GIF: frame delay in centiseconds (1-100).

        Returns:
            bytes: ZIP file contents.
        """
        body = {"seeds": seeds, "style": style, "size": size, "format": fmt}
        if background:
            body["background"] = background
        if theme:
            body["theme"] = theme
        if frames is not None:
            body["frames"] = frames
        if delay is not None:
            body["delay"] = delay
        data, status, headers = self._request("POST", "/api/v1/avatar/gallery/zip", body=body)
        return data

    def gallery_zip_timed(self, seeds, style="geometric", size=256, fmt="png", background=None, theme=None,
                          frames=None, delay=None):
        """Download gallery ZIP with timing info.

        Returns (bytes, generation_ms, avatar_count).
        generation_ms and avatar_count are from response headers (float/int or None).
        """
        body = {"seeds": seeds, "style": style, "size": size, "format": fmt}
        if background:
            body["background"] = background
        if theme:
            body["theme"] = theme
        if frames is not None:
            body["frames"] = frames
        if delay is not None:
            body["delay"] = delay
        data, status, headers = self._request("POST", "/api/v1/avatar/gallery/zip", body=body)
        gen_ms = headers.get("x-generation-time-ms")
        if gen_ms is not None:
            gen_ms = float(gen_ms)
        count = headers.get("x-avatar-count")
        if count is not None:
            count = int(count)
        return data, gen_ms, count

    def gallery_zip_save(self, seeds, path, style="geometric", size=256, fmt="png", background=None, theme=None,
                         frames=None, delay=None):
        """Download gallery ZIP and save to file.

        Args:
            seeds: List of seed strings.
            path: Output file path.
            style: Avatar style or "all".
            size: Avatar size.
            fmt: "png", "svg", or "gif".
            background: Optional hex color.
            theme: Optional color theme name.
            frames: For GIF: animation frames (2-30).
            delay: For GIF: frame delay in centiseconds (1-100).

        Returns:
            str: Path to saved file.
        """
        data = self.gallery_zip(seeds, style=style, size=size, fmt=fmt, background=background,
                                theme=theme, frames=frames, delay=delay)
        with open(path, "wb") as f:
            f.write(data)
        return path

    def styles(self):
        """List available styles."""
        data, status, headers = self._request("GET", "/api/v1/styles")
        return data

    def themes(self):
        """List available color themes."""
        data, status, headers = self._request("GET", "/api/v1/themes")
        return data

    def save(self, seed, path, style="geometric", size=256, fmt=None, background=None, theme=None,
             frames=None, delay=None):
        """Generate and save avatar to file."""
        if fmt is None:
            if path.endswith(".svg"):
                fmt = "svg"
            elif path.endswith(".gif"):
                fmt = "gif"
            else:
                fmt = "png"
        data = self.generate(seed, style=style, size=size, fmt=fmt, background=background,
                             theme=theme, frames=frames, delay=delay)
        mode = "w" if isinstance(data, str) else "wb"
        with open(path, mode) as f:
            f.write(data)
        return path

    # ── Discovery ──

    def health(self):
        """Check service health."""
        data, status, headers = self._request("GET", "/api/v1/health")
        return data

    def openapi(self):
        """Get OpenAPI spec."""
        data, status, headers = self._request("GET", "/api/v1/openapi.json")
        return data

    def llms_txt(self):
        """Get llms.txt."""
        data, status, headers = self._request("GET", "/api/v1/llms.txt")
        if isinstance(data, bytes):
            return data.decode("utf-8")
        return data

    def skill_md(self):
        """Get SKILL.md."""
        data, status, headers = self._request("GET", "/.well-known/skills/agent-avatar-generator/SKILL.md")
        if isinstance(data, bytes):
            return data.decode("utf-8")
        return data

    def skills_index(self):
        """Get skills index."""
        data, status, headers = self._request("GET", "/.well-known/skills/agent-avatar-generator/index.json")
        return data
