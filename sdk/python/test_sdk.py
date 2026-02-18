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
        for style in ["geometric", "rings", "robot", "blockies", "gradient"]:
            data = self.client.generate("test", style=style)
            self.assertIsInstance(data, bytes, f"Style {style} should return bytes")
            self.assertTrue(len(data) > 0, f"Style {style} should return non-empty")

    def test_all_styles_svg(self):
        for style in ["geometric", "rings", "robot", "blockies", "gradient"]:
            data = self.client.generate_svg("test", style=style)
            self.assertIsInstance(data, str, f"Style {style} SVG should return string")
            self.assertTrue(data.startswith("<svg"), f"Style {style} SVG should start with <svg")

    def test_list_styles(self):
        styles = self.client.styles()
        self.assertIsInstance(styles, list)
        self.assertEqual(len(styles), 5)
        names = [s["name"] for s in styles]
        self.assertIn("geometric", names)
        self.assertIn("rings", names)
        self.assertIn("robot", names)
        self.assertIn("blockies", names)
        self.assertIn("gradient", names)

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
