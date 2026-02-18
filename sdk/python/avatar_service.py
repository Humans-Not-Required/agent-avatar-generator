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

    VALID_STYLES = {"geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst"}
    VALID_FORMATS = {"png", "svg"}

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

            if "application/json" in content_type:
                return json.loads(raw), resp.status, dict(resp.headers)
            return raw, resp.status, dict(resp.headers)

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

    def generate(self, seed, style="geometric", size=256, fmt="png", background=None):
        """Generate an avatar. Returns bytes (PNG) or string (SVG)."""
        params = {"style": style, "size": size, "format": fmt}
        if background:
            params["background"] = background
        qs = urlencode(params)
        data, status, headers = self._request("GET", f"/api/v1/avatar/{quote(seed, safe='')}?{qs}")
        return data

    def generate_png(self, seed, style="geometric", size=256, background=None):
        """Generate a PNG avatar. Returns bytes."""
        return self.generate(seed, style=style, size=size, fmt="png", background=background)

    def generate_svg(self, seed, style="geometric", size=256, background=None):
        """Generate an SVG avatar. Returns string."""
        data = self.generate(seed, style=style, size=size, fmt="svg", background=background)
        if isinstance(data, bytes):
            return data.decode("utf-8")
        return data

    def batch(self, seeds, style="geometric", size=128, fmt="png", background=None):
        """Batch generate avatars. Returns list of dicts with base64-encoded data."""
        body = {"seeds": seeds, "style": style, "size": size, "format": fmt}
        if background:
            body["background"] = background
        data, status, headers = self._request("POST", "/api/v1/avatar/batch", body=body)
        return data["avatars"]

    def styles(self):
        """List available styles."""
        data, status, headers = self._request("GET", "/api/v1/styles")
        return data

    def save(self, seed, path, style="geometric", size=256, fmt=None, background=None):
        """Generate and save avatar to file."""
        if fmt is None:
            fmt = "svg" if path.endswith(".svg") else "png"
        data = self.generate(seed, style=style, size=size, fmt=fmt, background=background)
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
