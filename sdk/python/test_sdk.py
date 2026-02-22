"""
Agent Avatar Generator — Python SDK Integration Tests

Run against a live server:
    AVATAR_SERVICE_URL=http://localhost:8000 python3 test_sdk.py
"""

import base64
import json
import os
import sys
import unittest

# Add SDK to path
sys.path.insert(0, os.path.dirname(__file__))
from avatar_service import AvatarService, ValidationError, RateLimitError


class TestHealth(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_health(self):
        data = self.client.health()
        self.assertEqual(data["status"], "ok")
        self.assertEqual(data["service"], "agent-avatar-generator")
        self.assertIn("version", data)


class TestGenerate(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_generate_png_default(self):
        data = self.client.generate("test-seed")
        self.assertIsInstance(data, bytes)
        self.assertTrue(len(data) > 0)
        # PNG magic header
        self.assertEqual(data[:4], b'\x89PNG')

    def test_generate_svg(self):
        data = self.client.generate_svg("test-seed")
        self.assertIsInstance(data, str)
        self.assertTrue(data.startswith("<svg"))
        self.assertIn("</svg>", data)

    def test_generate_png_explicit(self):
        data = self.client.generate_png("test-seed")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:4], b'\x89PNG')

    def test_deterministic(self):
        d1 = self.client.generate("deterministic-test")
        d2 = self.client.generate("deterministic-test")
        self.assertEqual(d1, d2)

    def test_different_seeds(self):
        d1 = self.client.generate("seed-alpha")
        d2 = self.client.generate("seed-beta")
        self.assertNotEqual(d1, d2)

    def test_custom_size(self):
        data = self.client.generate("test", size=128)
        self.assertIsInstance(data, bytes)
        self.assertTrue(len(data) > 0)

    def test_min_size(self):
        data = self.client.generate("test", size=16)
        self.assertIsInstance(data, bytes)

    def test_max_size(self):
        data = self.client.generate("test", size=1024)
        self.assertIsInstance(data, bytes)

    def test_size_too_small(self):
        with self.assertRaises(ValidationError):
            self.client.generate("test", size=8)

    def test_size_too_large(self):
        with self.assertRaises(ValidationError):
            self.client.generate("test", size=2048)

    def test_background_color(self):
        data = self.client.generate("test", background="ff0000")
        self.assertIsInstance(data, bytes)

    def test_invalid_style(self):
        with self.assertRaises(ValidationError):
            self.client.generate("test", style="invalid")

    def test_invalid_format(self):
        with self.assertRaises(ValidationError):
            self.client.generate("test", fmt="bmp")

    def test_special_chars_in_seed(self):
        data = self.client.generate("nanook@claw.inc")
        self.assertIsInstance(data, bytes)

    def test_long_seed(self):
        data = self.client.generate("a" * 500)
        self.assertIsInstance(data, bytes)


class TestStyles(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_all_styles_png(self):
        for style in ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"]:
            data = self.client.generate("test", style=style)
            self.assertIsInstance(data, bytes, f"Style {style} should return bytes")
            self.assertTrue(len(data) > 0, f"Style {style} should return non-empty")

    def test_all_styles_svg(self):
        for style in ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"]:
            data = self.client.generate_svg("test", style=style)
            self.assertIsInstance(data, str, f"Style {style} SVG should return string")
            self.assertTrue(data.startswith("<svg"), f"Style {style} SVG should start with <svg")

    def test_list_styles(self):
        styles = self.client.styles()
        self.assertIsInstance(styles, list)
        self.assertEqual(len(styles), 11)
        names = [s["name"] for s in styles]
        self.assertIn("geometric", names)
        self.assertIn("rings", names)
        self.assertIn("robot", names)
        self.assertIn("blockies", names)
        self.assertIn("gradient", names)
        self.assertIn("initials", names)
        self.assertIn("starburst", names)
        self.assertIn("mosaic", names)
        self.assertIn("pixel", names)
        self.assertIn("sunset", names)

    def test_styles_have_description(self):
        styles = self.client.styles()
        for s in styles:
            self.assertIn("description", s)
            self.assertIn("sample_seed", s)


class TestBatch(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_batch_generate(self):
        results = self.client.batch(["a", "b", "c"])
        self.assertEqual(len(results), 3)
        for item in results:
            self.assertIn("seed", item)
            self.assertIn("data", item)
            self.assertEqual(item["format"], "png")
            # Verify base64 is valid
            decoded = base64.b64decode(item["data"])
            self.assertEqual(decoded[:4], b'\x89PNG')

    def test_batch_svg(self):
        results = self.client.batch(["x", "y"], fmt="svg")
        self.assertEqual(len(results), 2)
        for item in results:
            self.assertEqual(item["format"], "svg")
            decoded = base64.b64decode(item["data"]).decode("utf-8")
            self.assertTrue(decoded.startswith("<svg"))

    def test_batch_with_style(self):
        results = self.client.batch(["test"], style="robot", size=64)
        self.assertEqual(len(results), 1)

    def test_batch_too_many(self):
        with self.assertRaises(ValidationError):
            self.client.batch([f"seed-{i}" for i in range(51)])

    def test_batch_empty(self):
        with self.assertRaises(ValidationError):
            self.client.batch([])

    def test_batch_invalid_style(self):
        with self.assertRaises(ValidationError):
            self.client.batch(["a"], style="bad")

    def test_batch_deterministic(self):
        r1 = self.client.batch(["det-test"])
        r2 = self.client.batch(["det-test"])
        self.assertEqual(r1[0]["data"], r2[0]["data"])


class TestSave(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()
        self.test_dir = "/tmp/avatar_test"
        os.makedirs(self.test_dir, exist_ok=True)

    def test_save_png(self):
        path = os.path.join(self.test_dir, "test.png")
        self.client.save("test", path)
        self.assertTrue(os.path.exists(path))
        with open(path, "rb") as f:
            self.assertEqual(f.read(4), b'\x89PNG')
        os.unlink(path)

    def test_save_svg(self):
        path = os.path.join(self.test_dir, "test.svg")
        self.client.save("test", path)
        self.assertTrue(os.path.exists(path))
        with open(path, "r") as f:
            content = f.read()
            self.assertTrue(content.startswith("<svg"))
        os.unlink(path)

    def test_save_explicit_format(self):
        path = os.path.join(self.test_dir, "test_explicit.png")
        self.client.save("test", path, fmt="png")
        self.assertTrue(os.path.exists(path))
        os.unlink(path)


class TestDiscovery(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_openapi(self):
        data = self.client.openapi()
        self.assertEqual(data["openapi"], "3.0.3")
        self.assertEqual(data["info"]["title"], "Agent Avatar Generator")

    def test_llms_txt(self):
        data = self.client.llms_txt()
        self.assertIn("Agent Avatar Generator", data)

    def test_skill_md(self):
        data = self.client.skill_md()
        self.assertIn("Agent Avatar Generator", data)

    def test_skills_index(self):
        data = self.client.skills_index()
        self.assertIn("skills", data)
        self.assertIsInstance(data["skills"], list)


class TestInitialsStyle(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_initials_png(self):
        data = self.client.generate_png("Nanook", style="initials")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_initials_svg(self):
        data = self.client.generate_svg("Nanook", style="initials")
        self.assertIn("<text", data)
        self.assertIn("NA", data)  # Initials are uppercased

    def test_initials_deterministic(self):
        d1 = self.client.generate("Agent42", style="initials")
        d2 = self.client.generate("Agent42", style="initials")
        self.assertEqual(d1, d2)

    def test_initials_different_seeds(self):
        d1 = self.client.generate("Alice", style="initials")
        d2 = self.client.generate("Bob", style="initials")
        self.assertNotEqual(d1, d2)

    def test_initials_numeric_seed(self):
        data = self.client.generate_svg("42", style="initials")
        self.assertIn("42", data)

    def test_initials_single_char(self):
        data = self.client.generate_svg("X", style="initials")
        self.assertIn("X", data)

    def test_initials_with_bg(self):
        data = self.client.generate("Test", style="initials", background="ff0000")
        self.assertIsInstance(data, bytes)

    def test_initials_small(self):
        data = self.client.generate("AB", style="initials", size=16)
        self.assertIsInstance(data, bytes)

    def test_initials_large(self):
        data = self.client.generate("AB", style="initials", size=512)
        self.assertIsInstance(data, bytes)

    def test_initials_batch(self):
        results = self.client.batch(["Alice", "Bob", "Charlie"], style="initials", size=64)
        self.assertEqual(len(results), 3)
        for item in results:
            self.assertIsNone(item.get("error"))

    def test_initials_save_png(self):
        path = "/tmp/avatar_test/initials.png"
        os.makedirs(os.path.dirname(path), exist_ok=True)
        self.client.save("Nanook", path, style="initials")
        self.assertTrue(os.path.exists(path))
        with open(path, "rb") as f:
            self.assertEqual(f.read(4), b'\x89PNG')
        os.unlink(path)

    def test_initials_save_svg(self):
        path = "/tmp/avatar_test/initials.svg"
        os.makedirs(os.path.dirname(path), exist_ok=True)
        self.client.save("Nanook", path, style="initials")
        self.assertTrue(os.path.exists(path))
        with open(path, "r") as f:
            self.assertIn("<text", f.read())
        os.unlink(path)

    def test_initials_email_seed(self):
        """Email-style seeds should extract letters."""
        data = self.client.generate_svg("nanook@claw.inc", style="initials")
        self.assertIn("<text", data)

    def test_initials_batch_svg(self):
        results = self.client.batch(["X", "Y", "Z"], style="initials", fmt="svg")
        self.assertEqual(len(results), 3)
        for item in results:
            decoded = base64.b64decode(item["data"]).decode("utf-8")
            self.assertIn("<text", decoded)


class TestStarburstStyle(unittest.TestCase):
    def setUp(self):
        self.client = AvatarService()

    def test_starburst_png(self):
        data = self.client.generate_png("star", style="starburst")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_starburst_svg(self):
        data = self.client.generate_svg("star", style="starburst")
        self.assertIn("<path", data)  # Ray paths
        self.assertIn("<circle", data)  # Center dot

    def test_starburst_deterministic(self):
        d1 = self.client.generate("burst", style="starburst")
        d2 = self.client.generate("burst", style="starburst")
        self.assertEqual(d1, d2)

    def test_starburst_different_seeds(self):
        d1 = self.client.generate("sun", style="starburst")
        d2 = self.client.generate("moon", style="starburst")
        self.assertNotEqual(d1, d2)

    def test_starburst_with_bg(self):
        data = self.client.generate("star", style="starburst", background="000033")
        self.assertIsInstance(data, bytes)

    def test_starburst_small(self):
        data = self.client.generate("star", style="starburst", size=16)
        self.assertIsInstance(data, bytes)

    def test_starburst_large(self):
        data = self.client.generate("star", style="starburst", size=512)
        self.assertIsInstance(data, bytes)

    def test_starburst_batch(self):
        results = self.client.batch(["sun", "moon", "star"], style="starburst", size=64)
        self.assertEqual(len(results), 3)
        for item in results:
            self.assertIsNone(item.get("error"))

    def test_starburst_save_png(self):
        path = "/tmp/avatar_test/starburst.png"
        os.makedirs(os.path.dirname(path), exist_ok=True)
        self.client.save("star", path, style="starburst")
        self.assertTrue(os.path.exists(path))
        os.unlink(path)

    def test_starburst_save_svg(self):
        path = "/tmp/avatar_test/starburst.svg"
        os.makedirs(os.path.dirname(path), exist_ok=True)
        self.client.save("star", path, style="starburst")
        self.assertTrue(os.path.exists(path))
        with open(path, "r") as f:
            content = f.read()
            self.assertIn("<path", content)
        os.unlink(path)

    def test_starburst_batch_svg(self):
        results = self.client.batch(["a", "b"], style="starburst", fmt="svg")
        self.assertEqual(len(results), 2)
        for item in results:
            decoded = base64.b64decode(item["data"]).decode("utf-8")
            self.assertIn("<path", decoded)

    def test_starburst_multiple_sizes(self):
        """Different sizes should all work."""
        for size in [32, 64, 128, 256]:
            data = self.client.generate("star", style="starburst", size=size)
            self.assertIsInstance(data, bytes)
            self.assertTrue(len(data) > 0)


class TestMosaicStyle(unittest.TestCase):
    def setUp(self):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        self.client = AvatarService(url)

    def test_mosaic_png(self):
        data = self.client.generate("mosaic-test", style="mosaic", size=128)
        self.assertIsInstance(data, bytes)
        self.assertTrue(len(data) > 100)

    def test_mosaic_svg(self):
        svg = self.client.generate_svg("mosaic-test", style="mosaic", size=128)
        self.assertIn("<svg", svg)
        self.assertIn("</svg>", svg)

    def test_mosaic_deterministic(self):
        a = self.client.generate("mosaic-det", style="mosaic", size=64)
        b = self.client.generate("mosaic-det", style="mosaic", size=64)
        self.assertEqual(a, b)

    def test_mosaic_different_seeds(self):
        a = self.client.generate("mosaic-a", style="mosaic", size=64)
        b = self.client.generate("mosaic-b", style="mosaic", size=64)
        self.assertNotEqual(a, b)

    def test_mosaic_multiple_sizes(self):
        for sz in [16, 64, 128, 256]:
            data = self.client.generate("mosaic-sz", style="mosaic", size=sz)
            self.assertTrue(len(data) > 0, f"Empty at size {sz}")

    def test_mosaic_with_bg(self):
        data = self.client.generate("mosaic-bg", style="mosaic", size=64, background="ff0000")
        self.assertIsInstance(data, bytes)

    def test_mosaic_batch(self):
        results = self.client.batch(["m1", "m2", "m3"], style="mosaic", size=64)
        self.assertEqual(len(results), 3)

    def test_mosaic_save_png(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".png", delete=True) as f:
            self.client.save("mosaic-save", f.name, style="mosaic", size=64)
            self.assertTrue(os.path.getsize(f.name) > 0)

    def test_mosaic_save_svg(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".svg", delete=True) as f:
            self.client.save("mosaic-save", f.name, style="mosaic", size=64, fmt="svg")
            self.assertTrue(os.path.getsize(f.name) > 0)


class TestPixelStyle(unittest.TestCase):
    def setUp(self):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        self.client = AvatarService(url)

    def test_pixel_png(self):
        data = self.client.generate_png("invader", style="pixel")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_pixel_svg(self):
        svg = self.client.generate_svg("invader", style="pixel")
        self.assertIn("<svg", svg)
        self.assertIn("<rect", svg)  # Should have pixel rects

    def test_pixel_deterministic(self):
        a = self.client.generate("creature", style="pixel", size=128)
        b = self.client.generate("creature", style="pixel", size=128)
        self.assertEqual(a, b)

    def test_pixel_different_seeds(self):
        a = self.client.generate("alien-a", style="pixel", size=64)
        b = self.client.generate("alien-b", style="pixel", size=64)
        self.assertNotEqual(a, b)

    def test_pixel_multiple_sizes(self):
        for sz in [16, 64, 128, 256, 512]:
            data = self.client.generate("pixel-sz", style="pixel", size=sz)
            self.assertTrue(len(data) > 0, f"Empty at size {sz}")

    def test_pixel_with_bg(self):
        data = self.client.generate("pixel-bg", style="pixel", background="000033")
        self.assertIsInstance(data, bytes)

    def test_pixel_batch(self):
        results = self.client.batch(["inv1", "inv2", "inv3"], style="pixel", size=64)
        self.assertEqual(len(results), 3)
        for item in results:
            self.assertIsNone(item.get("error"))

    def test_pixel_save_png(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".png", delete=True) as f:
            self.client.save("pixel-save", f.name, style="pixel", size=64)
            self.assertTrue(os.path.getsize(f.name) > 0)

    def test_pixel_save_svg(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".svg", delete=True) as f:
            self.client.save("pixel-save", f.name, style="pixel", size=64, fmt="svg")
            self.assertTrue(os.path.getsize(f.name) > 0)

    def test_pixel_batch_svg(self):
        results = self.client.batch(["p1", "p2"], style="pixel", fmt="svg")
        self.assertEqual(len(results), 2)
        for item in results:
            decoded = base64.b64decode(item["data"]).decode("utf-8")
            self.assertIn("<rect", decoded)


class TestSunsetStyle(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        cls.client = AvatarService(url)

    def test_sunset_png(self):
        data = self.client.generate_png("horizon", style="sunset")
        self.assertTrue(data[:4] == b'\x89PNG')

    def test_sunset_svg(self):
        svg = self.client.generate_svg("horizon", style="sunset")
        self.assertIn("<svg", svg)
        self.assertIn("<linearGradient", svg)

    def test_sunset_deterministic(self):
        a = self.client.generate("dawn", style="sunset", size=128)
        b = self.client.generate("dawn", style="sunset", size=128)
        self.assertEqual(a, b)

    def test_sunset_different_seeds(self):
        a = self.client.generate("dawn", style="sunset", size=128)
        b = self.client.generate("dusk", style="sunset", size=128)
        self.assertNotEqual(a, b)

    def test_sunset_multiple_sizes(self):
        for sz in [16, 64, 128, 256, 512]:
            data = self.client.generate("sunset-size", style="sunset", size=sz)
            self.assertIsNotNone(data)

    def test_sunset_with_bg(self):
        data = self.client.generate("sunset-bg", style="sunset", background="001133")
        self.assertIsNotNone(data)

    def test_sunset_batch(self):
        results = self.client.batch(["s1", "s2", "s3"], style="sunset")
        self.assertEqual(len(results), 3)
        for item in results:
            self.assertIn("seed", item)
            self.assertIn("data", item)

    def test_sunset_save_png(self):
        path = "/tmp/test_sunset.png"
        self.client.save("sunset-save", path, style="sunset")
        self.assertTrue(os.path.exists(path))
        with open(path, "rb") as f:
            self.assertTrue(f.read(4) == b'\x89PNG')
        os.unlink(path)

    def test_sunset_save_svg(self):
        path = "/tmp/test_sunset.svg"
        self.client.save("sunset-save", path, style="sunset", fmt="svg")
        self.assertTrue(os.path.exists(path))
        with open(path, "r") as f:
            content = f.read()
            self.assertIn("<svg", content)
        os.unlink(path)

    def test_sunset_batch_svg(self):
        results = self.client.batch(["s1", "s2"], style="sunset", fmt="svg")
        self.assertEqual(len(results), 2)
        for item in results:
            decoded = base64.b64decode(item["data"]).decode("utf-8")
            self.assertIn("<svg", decoded)

    def test_sunset_harmony_colors(self):
        """Sunset should produce varied but valid output for many seeds."""
        for i in range(10):
            data = self.client.generate_png(f"harmony-{i}", style="sunset")
            self.assertTrue(len(data) > 100)



class TestConstellationStyle(unittest.TestCase):
    """Tests for the constellation (network graph) avatar style."""

    @classmethod
    def setUpClass(cls):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        cls.client = AvatarService(url)

    def test_constellation_png(self):
        """Constellation style produces valid PNG bytes."""
        data = self.client.generate_png("nanook", style="constellation")
        self.assertTrue(data[:4] == b'\x89PNG', "Should be a valid PNG")
        self.assertGreater(len(data), 500)

    def test_constellation_svg(self):
        """Constellation SVG contains node and edge elements."""
        svg = self.client.generate_svg("nanook", style="constellation")
        self.assertIn("<svg", svg)
        self.assertIn("<circle", svg, "Should have node circles")
        self.assertIn("<line", svg, "Should have edge lines")

    def test_constellation_deterministic(self):
        """Same seed always produces identical output."""
        a = self.client.generate("agent-42", style="constellation", size=128)
        b = self.client.generate("agent-42", style="constellation", size=128)
        self.assertEqual(a, b, "Constellation must be deterministic")

    def test_constellation_different_seeds(self):
        """Different seeds produce different avatars."""
        a = self.client.generate("alpha-node", style="constellation", size=128)
        b = self.client.generate("beta-node", style="constellation", size=128)
        self.assertNotEqual(a, b, "Different seeds should differ")

    def test_constellation_multiple_sizes(self):
        """Constellation renders at all valid sizes."""
        for sz in [16, 64, 128, 256, 512]:
            data = self.client.generate("size-test", style="constellation", size=sz)
            self.assertIsNotNone(data)
            self.assertGreater(len(data), 0)

    def test_constellation_with_background(self):
        """Background color override works."""
        dark = self.client.generate("bg-test", style="constellation", background="0a0a1a")
        light = self.client.generate("bg-test", style="constellation", background="f0f0ff")
        self.assertIsNotNone(dark)
        self.assertIsNotNone(light)
        self.assertNotEqual(dark, light, "Different backgrounds should differ")

    def test_constellation_batch(self):
        """Batch generation works for constellation."""
        seeds = ["node-a", "node-b", "node-c", "hub-1"]
        results = self.client.batch(seeds, style="constellation", size=64)
        self.assertEqual(len(results), 4)
        for item in results:
            self.assertIn("seed", item)
            self.assertIn("data", item)
            self.assertGreater(len(item["data"]), 100)

    def test_constellation_gif(self):
        """Constellation GIF animation works (pulse fallback)."""
        data = self.client.generate_gif("pulse-test", style="constellation", size=64)
        # GIF magic bytes: GIF89a
        self.assertTrue(data[:3] == b'GIF', "Should be a valid GIF")

    def test_constellation_save_png(self):
        """Save constellation avatar to file."""
        path = "/tmp/test_constellation.png"
        self.client.save("save-test", path, style="constellation")
        self.assertTrue(os.path.exists(path))
        with open(path, "rb") as f:
            self.assertEqual(f.read(4), b'\x89PNG')
        os.unlink(path)

    def test_constellation_in_gallery_zip(self):
        """Constellation style included in gallery ZIP."""
        import zipfile, io
        data = self.client.gallery_zip(["net-agent"], style="constellation", size=64)
        z = zipfile.ZipFile(io.BytesIO(data))
        names = z.namelist()
        self.assertEqual(len(names), 1)
        # ZIP entries are named {seed}.png — the seed "net-agent" is the key identifier
        self.assertIn("net-agent", names[0], "ZIP entry should be named after the seed")

    def test_constellation_svg_hub_glow(self):
        """SVG hub node has glow circle (opacity < 1 circle element)."""
        svg = self.client.generate_svg("hub-glow-test", style="constellation", size=256)
        # Hub glow is rendered as a circle with opacity="0.25"
        self.assertIn('opacity="0.25"', svg, "Hub glow circle should be present")


class TestConstructor(unittest.TestCase):
    def test_default_url(self):
        client = AvatarService()
        self.assertTrue(client.base_url.startswith("http"))

    def test_custom_url(self):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        client = AvatarService(url)
        self.assertEqual(client.base_url, url)

    def test_trailing_slash(self):
        url = os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        client = AvatarService(url + "/")
        self.assertEqual(client.base_url, url)

    def test_custom_timeout(self):
        client = AvatarService(timeout=5)
        self.assertEqual(client.timeout, 5)

    def test_env_var_fallback(self):
        client = AvatarService()
        self.assertIsNotNone(client.base_url)


class TestRobotHeadShapes(unittest.TestCase):
    """Tests for robot head shapes and accessories."""

    def setUp(self):
        self.client = AvatarService()

    def test_robot_dome_head_png(self):
        """Dome-headed robot generates valid PNG."""
        data = self.client.generate("dome-robot-1", style="robot", size=128)
        self.assertTrue(len(data) > 100)
        self.assertTrue(data[:8] == b'\x89PNG\r\n\x1a\n')

    def test_robot_dome_head_svg(self):
        """Dome-headed robot SVG contains path for arc."""
        svg = self.client.generate_svg("dome-robot-1", style="robot", size=128)
        self.assertIn("<svg", svg)
        self.assertIn("</svg>", svg)

    def test_robot_hex_head_png(self):
        """Hexagonal-headed robot generates valid PNG."""
        data = self.client.generate("hex-robot-1", style="robot", size=128)
        self.assertTrue(len(data) > 100)

    def test_robot_hex_head_svg(self):
        """Hexagonal-headed robot SVG is valid."""
        svg = self.client.generate_svg("hex-robot-1", style="robot", size=128)
        self.assertIn("<svg", svg)

    def test_robot_trapezoid_head_png(self):
        """Trapezoid-headed robot generates valid PNG."""
        data = self.client.generate("trap-robot-1", style="robot", size=128)
        self.assertTrue(len(data) > 100)

    def test_robot_different_heads_differ(self):
        """Different robot seeds produce different PNGs (head variety)."""
        imgs = set()
        for i in range(20):
            data = self.client.generate(f"robot-head-variety-{i}", style="robot", size=128)
            imgs.add(data)
        self.assertGreater(len(imgs), 15, "20 robot seeds should produce mostly unique images")

    def test_robot_accessories_dont_break_small_size(self):
        """Robot with accessories works at small sizes (16px)."""
        data = self.client.generate("small-robot-accessories", style="robot", size=16)
        self.assertTrue(len(data) > 50)

    def test_robot_accessories_large_size(self):
        """Robot with accessories works at large sizes (512px)."""
        data = self.client.generate("large-robot-accessories", style="robot", size=512)
        self.assertTrue(len(data) > 1000)

    def test_robot_batch_with_head_variety(self):
        """Batch robot generation works with new head shapes."""
        seeds = [f"batch-robot-head-{i}" for i in range(10)]
        result = self.client.batch(seeds, style="robot", size=64)
        self.assertEqual(len(result), 10)
        for item in result:
            self.assertIn("data", item)
            self.assertTrue(len(item["data"]) > 10)

    def test_robot_bg_override_with_head_shapes(self):
        """Background override works with new head shapes."""
        normal = self.client.generate("robot-bg-test", style="robot", size=128)
        override = self.client.generate("robot-bg-test", style="robot", size=128, background="ff0000")
        self.assertNotEqual(normal, override)

    def test_robot_svg_bg_override_with_head_shapes(self):
        """SVG background override works with new head shapes."""
        svg = self.client.generate_svg("robot-svg-bg-test", style="robot", size=128, background="00ff00")
        self.assertIn("#00ff00", svg)

    def test_robot_determinism_with_accessories(self):
        """Robot with all accessories is deterministic."""
        img1 = self.client.generate("determinism-robot-accessories", style="robot", size=256)
        img2 = self.client.generate("determinism-robot-accessories", style="robot", size=256)
        self.assertEqual(img1, img2)

    def test_robot_svg_determinism_with_accessories(self):
        """Robot SVG with all accessories is deterministic."""
        svg1 = self.client.generate_svg("svg-robot-acc", style="robot", size=256)
        svg2 = self.client.generate_svg("svg-robot-acc", style="robot", size=256)
        self.assertEqual(svg1, svg2)


class TestRobotAntennaExpansion(unittest.TestCase):
    """Tests for expanded robot antenna styles (6 types) and eye glow effects."""

    def setUp(self):
        self.client = AvatarService()

    def test_robot_antenna_variety_png(self):
        """30 different seeds should exercise multiple antenna styles."""
        imgs = set()
        for i in range(30):
            data = self.client.generate(f"antenna-py-{i}", style="robot", size=64)
            self.assertTrue(len(data) > 50)
            imgs.add(data)
        self.assertGreater(len(imgs), 20, "30 seeds should produce mostly unique robot avatars")

    def test_robot_antenna_variety_svg(self):
        """30 different seeds should exercise multiple SVG antenna styles."""
        svgs = set()
        for i in range(30):
            svg = self.client.generate_svg(f"antenna-svg-py-{i}", style="robot", size=64)
            self.assertIn("<svg", svg)
            svgs.add(svg)
        self.assertGreater(len(svgs), 20, "30 seeds should produce mostly unique robot SVGs")

    def test_robot_antenna_deterministic(self):
        """Each antenna seed produces identical output on re-request."""
        for i in range(6):
            seed = f"antenna-det-{i}"
            img1 = self.client.generate(seed, style="robot", size=128)
            img2 = self.client.generate(seed, style="robot", size=128)
            self.assertEqual(img1, img2, f"Seed {seed} not deterministic")

    def test_robot_antenna_svg_deterministic(self):
        """Each antenna SVG seed produces identical output."""
        for i in range(6):
            seed = f"antenna-svg-det-{i}"
            svg1 = self.client.generate_svg(seed, style="robot", size=128)
            svg2 = self.client.generate_svg(seed, style="robot", size=128)
            self.assertEqual(svg1, svg2, f"SVG seed {seed} not deterministic")

    def test_robot_antenna_small_size(self):
        """New antenna styles work at minimum size."""
        for i in range(10):
            data = self.client.generate(f"antenna-small-{i}", style="robot", size=16)
            self.assertTrue(len(data) > 30)

    def test_robot_antenna_large_size(self):
        """New antenna styles work at large size."""
        for i in range(6):
            data = self.client.generate(f"antenna-large-{i}", style="robot", size=512)
            self.assertTrue(len(data) > 1000)

    def test_robot_glow_variety_png(self):
        """Eye glow should produce variety across 20 seeds."""
        imgs = set()
        for i in range(20):
            data = self.client.generate(f"glow-py-{i}", style="robot", size=64)
            self.assertTrue(len(data) > 50)
            imgs.add(data)
        self.assertGreater(len(imgs), 15, "20 seeds should produce mostly unique glow variations")

    def test_robot_glow_svg_has_opacity(self):
        """At least some robot SVGs should contain opacity (from glow circles)."""
        found_opacity = False
        for i in range(20):
            svg = self.client.generate_svg(f"glow-opacity-{i}", style="robot", size=128)
            if 'opacity=' in svg:
                found_opacity = True
                break
        self.assertTrue(found_opacity, "Expected at least one SVG with opacity from eye glow")

    def test_robot_glow_batch(self):
        """Batch robot with glow works."""
        seeds = [f"glow-batch-{i}" for i in range(10)]
        result = self.client.batch(seeds, style="robot", size=64)
        self.assertEqual(len(result), 10)
        for item in result:
            self.assertTrue(len(item["data"]) > 10)

    def test_robot_themed_with_glow(self):
        """Themes apply on top of glow without errors."""
        themes = ["warm", "cool", "neon", "pastel", "ocean"]
        for theme in themes:
            data = self.client.generate("themed-glow-robot", style="robot", size=128, theme=theme)
            self.assertTrue(len(data) > 100, f"Failed for theme {theme}")

    def test_robot_themed_svg_with_glow(self):
        """SVG themes apply on top of glow without errors."""
        themes = ["forest", "monochrome", "earth"]
        for theme in themes:
            svg = self.client.generate_svg("themed-glow-svg-robot", style="robot", size=128, theme=theme)
            self.assertIn("<svg", svg, f"Failed for theme {theme}")

    def test_robot_antenna_bg_override(self):
        """New antenna styles work with background override."""
        normal = self.client.generate("antenna-bg-test", style="robot", size=128)
        override = self.client.generate("antenna-bg-test", style="robot", size=128, background="ff0000")
        self.assertNotEqual(normal, override)

    def test_robot_antenna_gallery_zip(self):
        """Gallery ZIP with robot style works with expanded features."""
        import io, zipfile
        seeds = ["ant-zip-1", "ant-zip-2", "ant-zip-3"]
        data = self.client.gallery_zip(seeds, style="robot", size=64)
        self.assertTrue(len(data) > 100)
        zf = zipfile.ZipFile(io.BytesIO(data))
        self.assertEqual(len(zf.namelist()), 3)

    def test_robot_antenna_timed(self):
        """Timed generation works with new antenna styles."""
        data, ms = self.client.generate_timed("antenna-timed", style="robot", size=128)
        self.assertTrue(len(data) > 100)
        self.assertGreater(ms, 0)


class TestGalleryZip(unittest.TestCase):
    """Tests for gallery ZIP download."""

    def setUp(self):
        self.client = AvatarService(os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000"))

    def test_gallery_zip_single_seed(self):
        """Download ZIP with a single seed."""
        data = self.client.gallery_zip(["nanook"])
        self.assertIsInstance(data, bytes)
        self.assertTrue(len(data) > 0)
        # ZIP magic bytes
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_multiple_seeds(self):
        """Download ZIP with multiple seeds."""
        data = self.client.gallery_zip(["alice", "bob", "charlie"])
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_svg_format(self):
        """Download ZIP with SVG format."""
        data = self.client.gallery_zip(["nanook"], fmt="svg")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_all_styles(self):
        """Download ZIP with all styles."""
        data = self.client.gallery_zip(["nanook"], style="all")
        self.assertIsInstance(data, bytes)
        self.assertTrue(len(data) > 1000)  # Should be larger with all styles

    def test_gallery_zip_custom_style(self):
        """Download ZIP with specific style."""
        data = self.client.gallery_zip(["test"], style="robot")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_custom_size(self):
        """Download ZIP with custom size."""
        data = self.client.gallery_zip(["test"], size=512)
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_with_background(self):
        """Download ZIP with background color."""
        data = self.client.gallery_zip(["test"], background="ff0000")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_gallery_zip_save(self):
        """Save gallery ZIP to file."""
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as f:
            path = f.name
        try:
            result = self.client.gallery_zip_save(["alice", "bob"], path)
            self.assertEqual(result, path)
            self.assertTrue(os.path.exists(path))
            self.assertTrue(os.path.getsize(path) > 0)
        finally:
            if os.path.exists(path):
                os.unlink(path)

    def test_gallery_zip_deterministic(self):
        """Same inputs produce same ZIP."""
        data1 = self.client.gallery_zip(["alice", "bob"], style="rings")
        data2 = self.client.gallery_zip(["alice", "bob"], style="rings")
        self.assertEqual(data1, data2)

    def test_gallery_zip_empty_seeds_error(self):
        """Empty seeds list raises error."""
        with self.assertRaises(ValidationError):
            self.client.gallery_zip([])

    def test_gallery_zip_invalid_style_error(self):
        """Invalid style raises error."""
        with self.assertRaises(ValidationError):
            self.client.gallery_zip(["test"], style="nonexistent")

    def test_gallery_zip_invalid_format_error(self):
        """Invalid format raises error."""
        with self.assertRaises(ValidationError):
            self.client.gallery_zip(["test"], fmt="bmp")

    def test_gallery_zip_all_styles_larger_than_single(self):
        """All styles produces larger ZIP than single style."""
        single = self.client.gallery_zip(["test"], style="geometric")
        all_styles = self.client.gallery_zip(["test"], style="all")
        self.assertGreater(len(all_styles), len(single))

    def test_gallery_zip_svg_all_styles(self):
        """SVG format works with all styles."""
        data = self.client.gallery_zip(["test"], style="all", fmt="svg")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")


class TestThemes(unittest.TestCase):
    """Tests for color theme support."""

    def setUp(self):
        self.client = AvatarService()

    def test_list_themes(self):
        """GET /api/v1/themes returns 15 themes (9 original + 6 Tailwind palettes added 2026-02-21)."""
        themes = self.client.themes()
        self.assertEqual(len(themes), 15)
        names = [t["name"] for t in themes]
        # Original themes
        self.assertIn("warm", names)
        self.assertIn("cool", names)
        self.assertIn("neon", names)
        self.assertIn("monochrome", names)
        # Tailwind palettes
        self.assertIn("rose", names)
        self.assertIn("amber", names)
        self.assertIn("sky", names)
        self.assertIn("violet", names)

    def test_themes_have_descriptions(self):
        """Each theme has a name and description."""
        themes = self.client.themes()
        for t in themes:
            self.assertIn("name", t)
            self.assertIn("description", t)
            self.assertIsInstance(t["name"], str)
            self.assertIsInstance(t["description"], str)
            self.assertGreater(len(t["description"]), 0)

    def test_themed_png_differs(self):
        """Themed PNG should differ from unthemed."""
        normal = self.client.generate_png("test")
        themed = self.client.generate_png("test", theme="warm")
        self.assertNotEqual(normal, themed)

    def test_themed_svg_differs(self):
        """Themed SVG should differ from unthemed."""
        normal = self.client.generate_svg("test")
        themed = self.client.generate_svg("test", theme="cool")
        self.assertNotEqual(normal, themed)
        self.assertTrue(themed.startswith("<svg"))

    def test_themed_deterministic(self):
        """Same seed + same theme = same output."""
        a = self.client.generate_png("test", theme="ocean")
        b = self.client.generate_png("test", theme="ocean")
        self.assertEqual(a, b)

    def test_different_themes_differ(self):
        """Different themes produce different output."""
        warm = self.client.generate_png("test", theme="warm")
        cool = self.client.generate_png("test", theme="cool")
        self.assertNotEqual(warm, cool)

    def test_all_themes_png(self):
        """All 9 themes produce valid PNGs."""
        for name in ["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"]:
            data = self.client.generate_png("test", theme=name)
            self.assertIsInstance(data, bytes, f"Theme {name} should produce bytes")
            self.assertEqual(data[:4], b"\x89PNG", f"Theme {name} should produce valid PNG")

    def test_all_themes_svg(self):
        """All 9 themes produce valid SVGs."""
        for name in ["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"]:
            data = self.client.generate_svg("test", theme=name)
            self.assertTrue(data.startswith("<svg"), f"Theme {name} should produce valid SVG")

    def test_themes_with_all_styles(self):
        """Themes work with every style."""
        for theme in ["warm", "neon", "monochrome"]:
            for style in ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel", "sunset"]:
                data = self.client.generate_png("test", style=style, theme=theme)
                self.assertIsInstance(data, bytes, f"{style}+{theme} should work")

    def test_invalid_theme(self):
        """Invalid theme name returns error."""
        with self.assertRaises(ValidationError):
            self.client.generate_png("test", theme="rainbow")

    def test_themed_batch(self):
        """Batch with theme works."""
        results = self.client.batch(["a", "b"], theme="ocean")
        self.assertEqual(len(results), 2)
        for r in results:
            self.assertGreater(len(r["data"]), 0)

    def test_themed_batch_svg(self):
        """Batch SVG with theme works."""
        results = self.client.batch(["a"], fmt="svg", theme="monochrome")
        self.assertEqual(len(results), 1)
        import base64
        decoded = base64.b64decode(results[0]["data"]).decode("utf-8")
        self.assertTrue(decoded.startswith("<svg"))

    def test_themed_gallery_zip(self):
        """Gallery ZIP with theme works."""
        data = self.client.gallery_zip(["test"], theme="forest")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_themed_gallery_zip_all_styles(self):
        """Gallery ZIP all styles + theme works."""
        data = self.client.gallery_zip(["test"], style="all", theme="pastel")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:2], b"PK")

    def test_theme_with_background(self):
        """Theme works with background override."""
        data = self.client.generate_png("test", theme="warm", background="FF0000")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:4], b"\x89PNG")

    def test_themed_save(self):
        """Save with theme works."""
        import tempfile, os
        with tempfile.NamedTemporaryFile(suffix=".png", delete=False) as f:
            path = f.name
        try:
            self.client.save("test", path, theme="earth")
            self.assertTrue(os.path.exists(path))
            with open(path, "rb") as f:
                self.assertEqual(f.read(4), b"\x89PNG")
        finally:
            os.unlink(path)


class TestTimingAPI(unittest.TestCase):
    """Tests for performance timing features."""

    def setUp(self):
        self.client = AvatarService(
            os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        )

    # ── generate_timed ──

    def test_generate_timed_returns_tuple(self):
        """generate_timed returns (data, generation_ms)."""
        data, gen_ms = self.client.generate_timed("timing-sdk-test")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:4], b"\x89PNG")
        self.assertIsInstance(gen_ms, float)
        self.assertGreaterEqual(gen_ms, 0.0)

    def test_generate_timed_svg(self):
        """generate_timed returns SVG with timing."""
        data, gen_ms = self.client.generate_timed("timing-svg", fmt="svg")
        # SVG is returned as bytes from _request (Content-Type: image/svg+xml)
        text = data.decode("utf-8") if isinstance(data, bytes) else data
        self.assertTrue(text.startswith("<svg"))
        self.assertIsInstance(gen_ms, float)
        self.assertGreaterEqual(gen_ms, 0.0)

    def test_generate_timed_with_theme(self):
        """generate_timed works with themes."""
        data, gen_ms = self.client.generate_timed("timing-themed", theme="warm")
        self.assertIsInstance(data, bytes)
        self.assertIsInstance(gen_ms, float)

    def test_generate_timed_with_style(self):
        """generate_timed works with different styles."""
        data, gen_ms = self.client.generate_timed("timing-robot", style="robot")
        self.assertIsInstance(data, bytes)
        self.assertIsInstance(gen_ms, float)

    def test_generate_timed_all_styles(self):
        """All styles return timing info."""
        styles = ["geometric", "rings", "robot", "blockies", "gradient",
                  "initials", "starburst", "mosaic", "pixel", "sunset"]
        for style in styles:
            data, gen_ms = self.client.generate_timed(f"timing-{style}", style=style, size=64)
            self.assertIsInstance(gen_ms, float, f"Style {style} should return timing")
            self.assertGreaterEqual(gen_ms, 0.0)

    # ── batch_timed ──

    def test_batch_timed_returns_full_response(self):
        """batch_timed returns dict with avatars, generation_ms, count."""
        result = self.client.batch_timed(["bt-a", "bt-b", "bt-c"], size=64)
        self.assertIn("avatars", result)
        self.assertIn("generation_ms", result)
        self.assertIn("count", result)
        self.assertEqual(result["count"], 3)
        self.assertEqual(len(result["avatars"]), 3)
        self.assertIsInstance(result["generation_ms"], float)
        self.assertGreaterEqual(result["generation_ms"], 0.0)

    def test_batch_timed_with_theme(self):
        """batch_timed works with themes."""
        result = self.client.batch_timed(["btt-1", "btt-2"], theme="ocean", size=64)
        self.assertEqual(result["count"], 2)
        self.assertIsInstance(result["generation_ms"], float)

    def test_batch_timed_50_seeds(self):
        """batch_timed handles max seeds with timing."""
        seeds = [f"perf-{i}" for i in range(50)]
        result = self.client.batch_timed(seeds, size=32)
        self.assertEqual(result["count"], 50)
        self.assertEqual(len(result["avatars"]), 50)
        self.assertGreater(result["generation_ms"], 0.0)

    def test_batch_vs_batch_timed_consistency(self):
        """batch and batch_timed produce same avatar data."""
        seeds = ["cons-a", "cons-b", "cons-c"]
        avatars = self.client.batch(seeds, style="rings", size=64)
        timed = self.client.batch_timed(seeds, style="rings", size=64)
        for i, av in enumerate(avatars):
            self.assertEqual(av["data"], timed["avatars"][i]["data"])

    # ── gallery_zip_timed ──

    def test_gallery_zip_timed_returns_tuple(self):
        """gallery_zip_timed returns (bytes, generation_ms, count)."""
        data, gen_ms, count = self.client.gallery_zip_timed(["gzt-a", "gzt-b"], size=32)
        self.assertIsInstance(data, bytes)
        # ZIP magic number
        self.assertEqual(data[:2], b"PK")
        self.assertIsInstance(gen_ms, float)
        self.assertGreaterEqual(gen_ms, 0.0)
        self.assertEqual(count, 2)

    def test_gallery_zip_timed_all_styles(self):
        """gallery_zip_timed with style='all' returns correct count."""
        data, gen_ms, count = self.client.gallery_zip_timed(
            ["gzt-all-1", "gzt-all-2"], style="all", size=32
        )
        # 2 seeds × 11 styles = 22
        self.assertEqual(count, 22)
        self.assertIsInstance(gen_ms, float)

    def test_gallery_zip_timed_with_theme(self):
        """gallery_zip_timed works with themes."""
        data, gen_ms, count = self.client.gallery_zip_timed(
            ["gzt-theme"], theme="neon", size=32
        )
        self.assertIsInstance(gen_ms, float)
        self.assertEqual(count, 1)

    def test_gallery_zip_vs_timed_consistency(self):
        """gallery_zip and gallery_zip_timed produce same ZIP data."""
        seeds = ["gzc-1", "gzc-2"]
        plain = self.client.gallery_zip(seeds, style="blockies", size=32)
        timed_data, _, _ = self.client.gallery_zip_timed(seeds, style="blockies", size=32)
        self.assertEqual(plain, timed_data)


class TestParallelDeterminism(unittest.TestCase):
    """Tests to verify parallel generation produces deterministic results."""

    def setUp(self):
        self.client = AvatarService(
            os.environ.get("AVATAR_SERVICE_URL", "http://localhost:8000")
        )

    def test_batch_deterministic_across_calls(self):
        """Multiple batch calls produce identical results (parallel safety)."""
        seeds = [f"det-{i}" for i in range(10)]
        result1 = self.client.batch(seeds, style="geometric", size=64)
        result2 = self.client.batch(seeds, style="geometric", size=64)
        for i in range(10):
            self.assertEqual(result1[i]["data"], result2[i]["data"],
                           f"Seed det-{i} should be deterministic across calls")

    def test_batch_matches_individual(self):
        """Batch-generated avatars match individually generated ones."""
        seeds = ["match-a", "match-b", "match-c"]
        batch_result = self.client.batch(seeds, style="robot", size=64)
        import base64
        for i, seed in enumerate(seeds):
            individual = self.client.generate_png(seed, style="robot", size=64)
            individual_b64 = base64.b64encode(individual).decode()
            self.assertEqual(batch_result[i]["data"], individual_b64,
                           f"Batch vs individual mismatch for seed '{seed}'")

    def test_gallery_zip_deterministic(self):
        """Gallery ZIP is deterministic across calls."""
        seeds = ["gd-1", "gd-2", "gd-3"]
        zip1 = self.client.gallery_zip(seeds, style="mosaic", size=32)
        zip2 = self.client.gallery_zip(seeds, style="mosaic", size=32)
        self.assertEqual(zip1, zip2)


class TestThemeComparison(unittest.TestCase):
    """Tests for theme comparison workflows (supports Compare mode UI)."""

    def setUp(self):
        self.client = AvatarService()

    def test_all_themes_produce_different_images(self):
        """Each theme should produce visually different output for the same seed."""
        themes = ["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"]
        seed = "theme-compare-test"
        images = {}
        for theme in themes:
            img = self.client.generate(seed, style="geometric", size=64, theme=theme)
            images[theme] = img

        # Get unthemed version
        plain = self.client.generate(seed, style="geometric", size=64)
        different_from_plain = sum(1 for img in images.values() if img != plain)
        self.assertGreaterEqual(different_from_plain, 5,
                                f"At least 5 of 9 themes should differ from plain (got {different_from_plain})")

    def test_theme_comparison_deterministic(self):
        """Themed avatars should be deterministic."""
        for theme in ["warm", "neon", "earth"]:
            img1 = self.client.generate("determ-test", style="pixel", size=64, theme=theme)
            img2 = self.client.generate("determ-test", style="pixel", size=64, theme=theme)
            self.assertEqual(img1, img2, f"Theme '{theme}' should be deterministic")

    def test_all_styles_with_one_theme(self):
        """Compare mode: one seed, all styles, one theme."""
        styles = ["geometric", "rings", "robot", "blockies", "gradient",
                  "initials", "starburst", "mosaic", "pixel", "sunset"]
        images = {}
        for style in styles:
            img = self.client.generate("style-compare", style=style, size=64, theme="cool")
            images[style] = img
            self.assertTrue(len(img) > 0)

        # All styles should produce different images
        keys = list(images.keys())
        for i in range(len(keys)):
            for j in range(i + 1, len(keys)):
                self.assertNotEqual(images[keys[i]], images[keys[j]],
                                    f"{keys[i]} and {keys[j]} should differ")

    def test_compare_warm_vs_cool(self):
        """Warm and cool themes should produce noticeably different output."""
        warm = self.client.generate("compare-wc", style="mosaic", size=64, theme="warm")
        cool = self.client.generate("compare-wc", style="mosaic", size=64, theme="cool")
        self.assertNotEqual(warm, cool)

    def test_compare_svg_themes(self):
        """SVG theme comparison."""
        plain = self.client.generate_svg("svg-compare", style="starburst", size=64)
        neon = self.client.generate_svg("svg-compare", style="starburst", size=64, theme="neon")
        pastel = self.client.generate_svg("svg-compare", style="starburst", size=64, theme="pastel")
        self.assertNotEqual(plain, neon)
        self.assertNotEqual(plain, pastel)
        self.assertNotEqual(neon, pastel)
        # All should be valid SVGs
        self.assertTrue(plain.startswith("<svg"))
        self.assertTrue(neon.startswith("<svg"))
        self.assertTrue(pastel.startswith("<svg"))

    def test_batch_theme_comparison(self):
        """Batch with same seeds but different themes should differ."""
        seeds = ["a", "b", "c"]
        warm_batch = self.client.batch(seeds, style="rings", size=64, theme="warm")
        ocean_batch = self.client.batch(seeds, style="rings", size=64, theme="ocean")

        self.assertEqual(len(warm_batch), 3)
        self.assertEqual(len(ocean_batch), 3)

        # At least some should differ
        different = sum(1 for w, o in zip(warm_batch, ocean_batch) if w["data"] != o["data"])
        self.assertGreaterEqual(different, 2)

    def test_gallery_zip_themed(self):
        """Gallery ZIP with theme applied."""
        seeds = ["zip-compare-1", "zip-compare-2"]
        zip_plain = self.client.gallery_zip(seeds, style="geometric", size=64)
        zip_warm = self.client.gallery_zip(seeds, style="geometric", size=64, theme="warm")
        self.assertNotEqual(zip_plain, zip_warm, "Themed ZIP should differ from plain")

    def test_timed_themed_generation(self):
        """Timed generation with theme."""
        data, ms = self.client.generate_timed("timed-theme", style="robot", size=64, theme="forest")
        self.assertTrue(len(data) > 0)
        self.assertGreaterEqual(ms, 0)

    def test_all_themes_all_styles_matrix(self):
        """Full compare matrix: every combination works without errors."""
        themes = ["warm", "cool", "ocean", "forest", "sunset", "neon", "pastel", "monochrome", "earth"]
        styles = ["geometric", "rings", "robot", "blockies", "gradient",
                  "initials", "starburst", "mosaic", "pixel", "sunset"]
        # Test a subset (3 themes × 3 styles) to keep test fast
        for theme in themes[:3]:
            for style in styles[:3]:
                img = self.client.generate("matrix-test", style=style, size=32, theme=theme)
                self.assertTrue(len(img) > 0, f"Failed for {style}/{theme}")

    def test_monochrome_different_from_earth(self):
        """Monochrome and earth should produce different results."""
        mono = self.client.generate("mono-earth", style="gradient", size=64, theme="monochrome")
        earth = self.client.generate("mono-earth", style="gradient", size=64, theme="earth")
        self.assertNotEqual(mono, earth)


class TestAnimatedGif(unittest.TestCase):
    """Tests for animated GIF generation."""
    def setUp(self):
        self.client = AvatarService()

    def test_generate_gif_default(self):
        data = self.client.generate_gif("gif-test")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:6], b'GIF89a')

    def test_generate_gif_via_generate(self):
        data = self.client.generate("gif-test2", fmt="gif")
        self.assertIsInstance(data, bytes)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_deterministic(self):
        d1 = self.client.generate_gif("gif-det")
        d2 = self.client.generate_gif("gif-det")
        self.assertEqual(d1, d2)

    def test_gif_different_seeds_differ(self):
        d1 = self.client.generate_gif("gif-a")
        d2 = self.client.generate_gif("gif-b")
        self.assertNotEqual(d1, d2)

    def test_gif_all_styles(self):
        for style in AvatarService.VALID_STYLES:
            data = self.client.generate_gif(f"gif-{style}", style=style, size=64, frames=3)
            self.assertEqual(data[:6], b'GIF89a', f"Style {style} didn't produce valid GIF")
            self.assertGreater(len(data), 100, f"Style {style} GIF too small")

    def test_gif_custom_frames(self):
        small = self.client.generate_gif("gif-frames", frames=2, size=64)
        large = self.client.generate_gif("gif-frames", frames=20, size=64)
        self.assertEqual(small[:6], b'GIF89a')
        self.assertEqual(large[:6], b'GIF89a')
        self.assertGreater(len(large), len(small))

    def test_gif_custom_delay(self):
        fast = self.client.generate_gif("gif-delay", delay=1, size=64, frames=3)
        slow = self.client.generate_gif("gif-delay", delay=50, size=64, frames=3)
        self.assertEqual(fast[:6], b'GIF89a')
        self.assertEqual(slow[:6], b'GIF89a')

    def test_gif_min_frames(self):
        data = self.client.generate_gif("gif-min", frames=2, size=64)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_max_frames(self):
        data = self.client.generate_gif("gif-max", frames=30, size=64)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_with_theme(self):
        """Theme param accepted on GIF endpoint (GIF themes are a no-op currently)."""
        data = self.client.generate_gif("gif-theme", theme="neon", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')
        self.assertGreater(len(data), 100)

    def test_gif_with_background(self):
        data = self.client.generate_gif("gif-bg", background="FF0000", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_custom_size(self):
        data = self.client.generate_gif("gif-size", size=128, frames=3)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_robot_style_animation(self):
        """Robot style has eye blink animation."""
        data = self.client.generate_gif("gif-robot", style="robot", size=64, frames=5)
        self.assertEqual(data[:6], b'GIF89a')
        self.assertGreater(len(data), 200)

    def test_gif_starburst_style_animation(self):
        """Starburst style has rotation animation."""
        data = self.client.generate_gif("gif-star", style="starburst", size=64, frames=5)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_sunset_style_animation(self):
        """Sunset style has sun movement animation."""
        data = self.client.generate_gif("gif-sunset", style="sunset", size=64, frames=5)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_pixel_style_animation(self):
        """Pixel style has color cycle animation."""
        data = self.client.generate_gif("gif-pixel", style="pixel", size=64, frames=5)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gif_timed(self):
        data, ms = self.client.generate_timed("gif-timed", fmt="gif", frames=3, size=64)
        self.assertEqual(data[:6], b'GIF89a')
        self.assertIsInstance(ms, float)
        self.assertGreater(ms, 0)

    def test_gif_batch(self):
        result = self.client.batch(["gif-b1", "gif-b2", "gif-b3"], fmt="gif", size=64, frames=3)
        self.assertEqual(len(result), 3)
        for item in result:
            raw = base64.b64decode(item["data"])
            self.assertEqual(raw[:6], b'GIF89a')

    def test_gif_gallery_zip(self):
        data = self.client.gallery_zip(["gif-z1", "gif-z2"], fmt="gif", size=64, frames=3)
        self.assertIsInstance(data, bytes)
        # ZIP magic bytes
        self.assertEqual(data[:2], b'PK')


class TestGeometricStyle(unittest.TestCase):
    """Dedicated tests for geometric (default) style."""
    def setUp(self):
        self.client = AvatarService()

    def test_geometric_is_default(self):
        default = self.client.generate("geo-default")
        explicit = self.client.generate("geo-default", style="geometric")
        self.assertEqual(default, explicit)

    def test_geometric_png(self):
        data = self.client.generate("geo-test", style="geometric")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_geometric_svg(self):
        data = self.client.generate_svg("geo-test", style="geometric")
        self.assertIn("<svg", data)
        self.assertIn("rect", data)

    def test_geometric_deterministic(self):
        d1 = self.client.generate("geo-det", style="geometric")
        d2 = self.client.generate("geo-det", style="geometric")
        self.assertEqual(d1, d2)

    def test_geometric_different_seeds(self):
        d1 = self.client.generate("geo-one", style="geometric")
        d2 = self.client.generate("geo-two", style="geometric")
        self.assertNotEqual(d1, d2)

    def test_geometric_small(self):
        data = self.client.generate("geo-small", style="geometric", size=32)
        self.assertEqual(data[:4], b'\x89PNG')

    def test_geometric_with_theme(self):
        plain = self.client.generate("geo-theme", style="geometric", size=64)
        warm = self.client.generate("geo-theme", style="geometric", size=64, theme="warm")
        self.assertNotEqual(plain, warm)

    def test_geometric_batch(self):
        result = self.client.batch(["geo-b1", "geo-b2"], style="geometric", size=64)
        self.assertEqual(len(result), 2)

    def test_geometric_gif(self):
        data = self.client.generate_gif("geo-gif", style="geometric", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')


class TestRingsStyle(unittest.TestCase):
    """Dedicated tests for rings style."""
    def setUp(self):
        self.client = AvatarService()

    def test_rings_png(self):
        data = self.client.generate("rings-test", style="rings")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_rings_svg(self):
        data = self.client.generate_svg("rings-test", style="rings")
        self.assertIn("<svg", data)
        self.assertIn("circle", data)

    def test_rings_deterministic(self):
        d1 = self.client.generate("rings-det", style="rings")
        d2 = self.client.generate("rings-det", style="rings")
        self.assertEqual(d1, d2)

    def test_rings_different_seeds(self):
        d1 = self.client.generate("rings-a", style="rings")
        d2 = self.client.generate("rings-b", style="rings")
        self.assertNotEqual(d1, d2)

    def test_rings_multiple_sizes(self):
        for size in [32, 64, 128, 256]:
            data = self.client.generate("rings-size", style="rings", size=size)
            self.assertEqual(data[:4], b'\x89PNG')

    def test_rings_with_bg(self):
        data = self.client.generate("rings-bg", style="rings", background="000000")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_rings_themed(self):
        data = self.client.generate("rings-theme", style="rings", size=64, theme="ocean")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_rings_gif(self):
        data = self.client.generate_gif("rings-gif", style="rings", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')

    def test_rings_batch(self):
        result = self.client.batch(["rings-b1", "rings-b2", "rings-b3"], style="rings", size=64)
        self.assertEqual(len(result), 3)


class TestBlockiesStyle(unittest.TestCase):
    """Dedicated tests for blockies style."""
    def setUp(self):
        self.client = AvatarService()

    def test_blockies_png(self):
        data = self.client.generate("blockies-test", style="blockies")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_blockies_svg(self):
        data = self.client.generate_svg("blockies-test", style="blockies")
        self.assertIn("<svg", data)
        self.assertIn("rect", data)

    def test_blockies_deterministic(self):
        d1 = self.client.generate("blockies-det", style="blockies")
        d2 = self.client.generate("blockies-det", style="blockies")
        self.assertEqual(d1, d2)

    def test_blockies_different_seeds(self):
        d1 = self.client.generate("blockies-a", style="blockies")
        d2 = self.client.generate("blockies-b", style="blockies")
        self.assertNotEqual(d1, d2)

    def test_blockies_small(self):
        data = self.client.generate("blockies-sm", style="blockies", size=32)
        self.assertEqual(data[:4], b'\x89PNG')

    def test_blockies_themed_all(self):
        for theme in ["warm", "cool", "neon"]:
            data = self.client.generate("blockies-th", style="blockies", size=64, theme=theme)
            self.assertEqual(data[:4], b'\x89PNG')

    def test_blockies_gif(self):
        data = self.client.generate_gif("blockies-gif", style="blockies", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')


class TestGradientStyle(unittest.TestCase):
    """Dedicated tests for gradient style."""
    def setUp(self):
        self.client = AvatarService()

    def test_gradient_png(self):
        data = self.client.generate("grad-test", style="gradient")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_gradient_svg(self):
        data = self.client.generate_svg("grad-test", style="gradient")
        self.assertIn("<svg", data)
        self.assertIn("linearGradient", data.lower().replace("lineargradient", "linearGradient"))

    def test_gradient_deterministic(self):
        d1 = self.client.generate("grad-det", style="gradient")
        d2 = self.client.generate("grad-det", style="gradient")
        self.assertEqual(d1, d2)

    def test_gradient_different_seeds(self):
        d1 = self.client.generate("grad-a", style="gradient")
        d2 = self.client.generate("grad-b", style="gradient")
        self.assertNotEqual(d1, d2)

    def test_gradient_with_bg(self):
        data = self.client.generate("grad-bg", style="gradient", background="FFFFFF")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_gradient_themed(self):
        plain = self.client.generate("grad-themed", style="gradient", size=64)
        pastel = self.client.generate("grad-themed", style="gradient", size=64, theme="pastel")
        self.assertNotEqual(plain, pastel)

    def test_gradient_gif(self):
        data = self.client.generate_gif("grad-gif", style="gradient", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')

    def test_gradient_multiple_sizes(self):
        sizes = self.client.generate("grad-64", style="gradient", size=64)
        sizel = self.client.generate("grad-64", style="gradient", size=256)
        self.assertNotEqual(len(sizes), len(sizel))


class TestResponseHeaders(unittest.TestCase):
    """Test response headers (cache, timing, rate limits)."""
    def setUp(self):
        self.client = AvatarService()

    def test_timing_header_on_generate(self):
        _, ms = self.client.generate_timed("hdr-test", size=64)
        self.assertIsNotNone(ms)
        self.assertGreater(ms, 0)

    def test_timing_header_on_batch(self):
        result = self.client.batch_timed(["hdr-b1", "hdr-b2"], size=64)
        self.assertIn("generation_ms", result)
        self.assertIn("count", result)
        self.assertEqual(result["count"], 2)
        self.assertGreater(result["generation_ms"], 0)

    def test_timing_header_on_gallery_zip(self):
        data, ms, count = self.client.gallery_zip_timed(["hdr-z1"], size=64)
        self.assertIsNotNone(ms)
        self.assertGreater(ms, 0)
        self.assertIsNotNone(count)

    def test_timing_on_gif(self):
        data, ms = self.client.generate_timed("hdr-gif", fmt="gif", frames=3, size=64)
        self.assertIsNotNone(ms)
        self.assertGreater(ms, 0)

    def test_batch_count_matches_seeds(self):
        result = self.client.batch_timed(["cnt-1", "cnt-2", "cnt-3", "cnt-4", "cnt-5"], size=64)
        self.assertEqual(result["count"], 5)

    def test_timed_svg(self):
        data, ms = self.client.generate_timed("hdr-svg", fmt="svg", size=64)
        self.assertIsNotNone(ms)
        if isinstance(data, bytes):
            data = data.decode("utf-8")
        self.assertIn("<svg", data)


class TestDualDiscovery(unittest.TestCase):
    """Test both discovery paths return consistent content."""
    def setUp(self):
        self.client = AvatarService()

    def test_skill_md_exists(self):
        data = self.client.skill_md()
        self.assertIn("avatar", data.lower())

    def test_llms_txt_exists(self):
        data = self.client.llms_txt()
        self.assertIn("avatar", data.lower())

    def test_openapi_structure(self):
        data = self.client.openapi()
        self.assertIn("openapi", data)
        self.assertIn("paths", data)
        self.assertIn("info", data)
        self.assertIn("title", data["info"])

    def test_openapi_has_avatar_endpoint(self):
        data = self.client.openapi()
        paths = data["paths"]
        avatar_paths = [p for p in paths if "avatar" in p]
        self.assertGreater(len(avatar_paths), 0)

    def test_skills_index_structure(self):
        data = self.client.skills_index()
        self.assertIn("skills", data)
        self.assertGreater(len(data["skills"]), 0)

    def test_llms_txt_mentions_endpoints(self):
        data = self.client.llms_txt()
        self.assertIn("/api/v1/avatar", data)

    def test_skill_md_mentions_styles(self):
        data = self.client.skill_md()
        self.assertIn("style", data.lower())

    def test_openapi_version(self):
        data = self.client.openapi()
        self.assertTrue(data["openapi"].startswith("3."))


class TestErrorHandling(unittest.TestCase):
    """Test error handling edge cases."""
    def setUp(self):
        self.client = AvatarService()

    def test_invalid_style_error(self):
        with self.assertRaises(ValidationError) as ctx:
            self.client.generate("err-style", style="nonexistent")
        self.assertEqual(ctx.exception.status_code, 400)

    def test_invalid_format_error(self):
        with self.assertRaises(ValidationError) as ctx:
            self.client.generate("err-fmt", fmt="bmp")
        self.assertEqual(ctx.exception.status_code, 400)

    def test_size_below_minimum(self):
        with self.assertRaises(ValidationError):
            self.client.generate("err-size", size=8)

    def test_size_above_maximum(self):
        with self.assertRaises(ValidationError):
            self.client.generate("err-size", size=10000)

    def test_batch_too_many_seeds(self):
        seeds = [f"err-{i}" for i in range(51)]
        with self.assertRaises(ValidationError):
            self.client.batch(seeds)

    def test_batch_empty_seeds(self):
        with self.assertRaises(ValidationError):
            self.client.batch([])

    def test_invalid_theme_error(self):
        with self.assertRaises(ValidationError):
            self.client.generate("err-theme", theme="nonexistent")

    def test_gallery_zip_empty_seeds(self):
        with self.assertRaises(ValidationError):
            self.client.gallery_zip([])

    def test_gallery_zip_too_many_seeds(self):
        seeds = [f"err-gz-{i}" for i in range(51)]
        with self.assertRaises(ValidationError):
            self.client.gallery_zip(seeds)


class TestCrossFeatInteractions(unittest.TestCase):
    """Test combinations of features working together."""
    def setUp(self):
        self.client = AvatarService()

    def test_gif_with_all_themes(self):
        for theme in AvatarService.VALID_THEMES:
            data = self.client.generate_gif(f"cross-{theme}", theme=theme, size=64, frames=3)
            self.assertEqual(data[:6], b'GIF89a', f"Theme {theme} GIF failed")

    def test_themed_gif_accepted(self):
        """Theme param accepted on GIF (themes are no-op for GIF currently)."""
        data = self.client.generate_gif("cross-plain", theme="warm", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')
        self.assertGreater(len(data), 100)

    def test_batch_gif_themed(self):
        result = self.client.batch(["cross-bg1", "cross-bg2"], fmt="gif", theme="cool", size=64, frames=3)
        self.assertEqual(len(result), 2)
        for item in result:
            raw = base64.b64decode(item["data"])
            self.assertEqual(raw[:6], b'GIF89a')

    def test_all_formats_same_seed(self):
        png = self.client.generate("cross-fmt", size=64)
        svg = self.client.generate_svg("cross-fmt", size=64)
        gif = self.client.generate_gif("cross-fmt", size=64, frames=3)
        self.assertEqual(png[:4], b'\x89PNG')
        self.assertIn("<svg", svg)
        self.assertEqual(gif[:6], b'GIF89a')

    def test_theme_plus_bg_override(self):
        # Theme + bg override should produce valid output
        data = self.client.generate("cross-tb", style="robot", size=64, theme="neon", background="000000")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_gallery_zip_gif_format(self):
        data = self.client.gallery_zip(["cross-zg1", "cross-zg2"], fmt="gif", size=64, frames=3)
        self.assertEqual(data[:2], b'PK')

    def test_save_gif(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".gif", delete=False) as f:
            path = f.name
        try:
            self.client.save("cross-save-gif", path, fmt="gif", size=64, frames=3)
            with open(path, "rb") as f:
                data = f.read()
            self.assertEqual(data[:6], b'GIF89a')
        finally:
            os.unlink(path)

    def test_gallery_zip_themed_gif(self):
        data = self.client.gallery_zip(["cross-ztg1"], style="robot", fmt="gif", theme="warm", size=64, frames=3)
        self.assertEqual(data[:2], b'PK')

    def test_timed_gif_themed(self):
        data, ms = self.client.generate_timed("cross-tgt", fmt="gif", theme="forest", size=64, frames=3)
        self.assertEqual(data[:6], b'GIF89a')
        self.assertIsNotNone(ms)
        self.assertGreater(ms, 0)


class TestUnicodeSeeds(unittest.TestCase):
    """Test Unicode and special character handling in seeds."""
    def setUp(self):
        self.client = AvatarService()

    def test_cjk_seed(self):
        data = self.client.generate("你好世界")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_emoji_seed(self):
        data = self.client.generate("🤖🔥❄️")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_arabic_seed(self):
        data = self.client.generate("مرحبا")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_cyrillic_seed(self):
        data = self.client.generate("Привет мир")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_mixed_unicode(self):
        data = self.client.generate("Hello你好مرحبا🤖")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_unicode_deterministic(self):
        d1 = self.client.generate("🎭")
        d2 = self.client.generate("🎭")
        self.assertEqual(d1, d2)

    def test_unicode_different_scripts_differ(self):
        d1 = self.client.generate("hello")
        d2 = self.client.generate("你好")
        self.assertNotEqual(d1, d2)

    def test_url_encoded_seed(self):
        data = self.client.generate("hello world/test")
        self.assertEqual(data[:4], b'\x89PNG')

    def test_empty_like_seed(self):
        data = self.client.generate(" ")
        self.assertEqual(data[:4], b'\x89PNG')


class TestBatchAdvanced(unittest.TestCase):
    """Advanced batch generation tests."""
    def setUp(self):
        self.client = AvatarService()

    def test_batch_single_seed(self):
        result = self.client.batch(["single"], size=64)
        self.assertEqual(len(result), 1)

    def test_batch_max_seeds(self):
        seeds = [f"max-{i}" for i in range(50)]
        result = self.client.batch(seeds, size=32)
        self.assertEqual(len(result), 50)

    def test_batch_matches_individual(self):
        """Batch results should match individual generation."""
        seeds = ["match-a", "match-b"]
        batch_result = self.client.batch(seeds, style="rings", size=64)
        for i, seed in enumerate(seeds):
            individual = self.client.generate(seed, style="rings", size=64)
            batch_data = base64.b64decode(batch_result[i]["data"])
            self.assertEqual(individual, batch_data, f"Mismatch for seed {seed}")

    def test_batch_all_styles(self):
        for style in AvatarService.VALID_STYLES:
            result = self.client.batch(["batch-style-test"], style=style, size=64)
            self.assertEqual(len(result), 1, f"Style {style} batch failed")

    def test_batch_timed_structure(self):
        result = self.client.batch_timed(["bt-1", "bt-2"], size=64)
        self.assertIn("avatars", result)
        self.assertIn("count", result)
        self.assertIn("generation_ms", result)
        self.assertIsInstance(result["avatars"], list)
        self.assertEqual(result["count"], 2)

    def test_batch_svg_content(self):
        result = self.client.batch(["svg-batch"], fmt="svg", size=64)
        self.assertEqual(len(result), 1)
        svg_text = base64.b64decode(result[0]["data"]).decode("utf-8")
        self.assertIn("<svg", svg_text)

    def test_batch_with_bg_and_theme(self):
        result = self.client.batch(["bg-th-1", "bg-th-2"], size=64, background="112233", theme="pastel")
        self.assertEqual(len(result), 2)


class TestGalleryZipAdvanced(unittest.TestCase):
    """Advanced gallery ZIP tests."""
    def setUp(self):
        self.client = AvatarService()

    def test_gallery_zip_all_styles_count(self):
        """style='all' generates all 10 styles per seed."""
        data = self.client.gallery_zip(["gz-all-1"], style="all", size=32)
        self.assertEqual(data[:2], b'PK')

    def test_gallery_zip_timed_structure(self):
        data, ms, count = self.client.gallery_zip_timed(["gz-t1", "gz-t2"], size=32)
        self.assertEqual(data[:2], b'PK')
        self.assertGreater(ms, 0)
        self.assertIsNotNone(count)

    def test_gallery_zip_save_roundtrip(self):
        import tempfile
        with tempfile.NamedTemporaryFile(suffix=".zip", delete=False) as f:
            path = f.name
        try:
            self.client.gallery_zip_save(["gz-save"], path, size=32)
            with open(path, "rb") as f:
                data = f.read()
            self.assertEqual(data[:2], b'PK')
            self.assertGreater(len(data), 100)
        finally:
            os.unlink(path)

    def test_gallery_zip_themed_all_styles(self):
        data = self.client.gallery_zip(["gz-themed"], style="all", theme="earth", size=32)
        self.assertEqual(data[:2], b'PK')

    def test_gallery_zip_multiple_seeds_svg(self):
        data = self.client.gallery_zip(["gz-s1", "gz-s2", "gz-s3"], fmt="svg", size=64)
        self.assertEqual(data[:2], b'PK')

    def test_gallery_zip_deterministic(self):
        d1 = self.client.gallery_zip(["gz-det"], size=32)
        d2 = self.client.gallery_zip(["gz-det"], size=32)
        self.assertEqual(d1, d2)


class TestHealthAdvanced(unittest.TestCase):
    """Advanced health endpoint tests."""
    def setUp(self):
        self.client = AvatarService()

    def test_health_fields(self):
        data = self.client.health()
        self.assertIn("service", data)
        self.assertIn("status", data)
        self.assertIn("version", data)
        self.assertEqual(data["service"], "agent-avatar-generator")
        self.assertEqual(data["status"], "ok")

    def test_health_version_format(self):
        data = self.client.health()
        version = data["version"]
        parts = version.split(".")
        self.assertEqual(len(parts), 3, f"Version {version} not semver")


class TestStylesAdvanced(unittest.TestCase):
    """Advanced styles endpoint tests."""
    def setUp(self):
        self.client = AvatarService()

    def test_all_10_styles_listed(self):
        data = self.client.styles()
        style_names = {s["name"] for s in data}
        self.assertEqual(style_names, AvatarService.VALID_STYLES)

    def test_each_style_has_description(self):
        data = self.client.styles()
        for style in data:
            self.assertIn("name", style)
            self.assertIn("description", style)
            self.assertGreater(len(style["description"]), 0)

    def test_styles_count(self):
        data = self.client.styles()
        self.assertEqual(len(data), 11)


class TestThemesAdvanced(unittest.TestCase):
    """Advanced themes endpoint tests."""
    def setUp(self):
        self.client = AvatarService()

    def test_all_9_themes_listed(self):
        data = self.client.themes()
        theme_names = {t["name"] for t in data}
        self.assertEqual(theme_names, AvatarService.VALID_THEMES)

    def test_each_theme_has_description(self):
        data = self.client.themes()
        for theme in data:
            self.assertIn("name", theme)
            self.assertIn("description", theme)
            self.assertGreater(len(theme["description"]), 0)

    def test_themes_count(self):
        data = self.client.themes()
        self.assertEqual(len(data), 9)


if __name__ == "__main__":
    # Count tests
    loader = unittest.TestLoader()
    suite = loader.loadTestsFromModule(sys.modules[__name__])
    total = suite.countTestCases()
    print(f"\n{'='*60}")
    print(f"Agent Avatar Generator SDK Tests — {total} tests")
    print(f"Server: {os.environ.get('AVATAR_SERVICE_URL', 'http://localhost:8000')}")
    print(f"{'='*60}\n")

    result = unittest.main(exit=False, verbosity=2)
    
    print(f"\n{'='*60}")
    print(f"Results: {total - len(result.result.failures) - len(result.result.errors)} passed, "
          f"{len(result.result.failures)} failed, {len(result.result.errors)} errors")
    print(f"{'='*60}")
    
    sys.exit(0 if result.result.wasSuccessful() else 1)
