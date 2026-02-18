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
            self.client.generate("test", fmt="gif")

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
        for style in ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel"]:
            data = self.client.generate("test", style=style)
            self.assertIsInstance(data, bytes, f"Style {style} should return bytes")
            self.assertTrue(len(data) > 0, f"Style {style} should return non-empty")

    def test_all_styles_svg(self):
        for style in ["geometric", "rings", "robot", "blockies", "gradient", "initials", "starburst", "mosaic", "pixel"]:
            data = self.client.generate_svg("test", style=style)
            self.assertIsInstance(data, str, f"Style {style} SVG should return string")
            self.assertTrue(data.startswith("<svg"), f"Style {style} SVG should start with <svg")

    def test_list_styles(self):
        styles = self.client.styles()
        self.assertIsInstance(styles, list)
        self.assertEqual(len(styles), 9)
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
