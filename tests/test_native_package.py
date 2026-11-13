"""Integration checks for the actual installed Python/native package boundary."""

import importlib.machinery
import importlib.metadata
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import world
from world import _native


class NativePackageTests(unittest.TestCase):
    def test_installed_distribution_matches_linked_core_and_extension(self):
        version = importlib.metadata.version("world-microcosm")
        self.assertEqual(world.__version__, version)
        self.assertEqual(_native.__version__, version)
        self.assertEqual(world.core_version(), version)
        self.assertTrue(any(str(_native.__file__).endswith(suffix)
                            for suffix in importlib.machinery.EXTENSION_SUFFIXES))

    def test_integer_values_survive_python_rust_round_trip(self):
        for a, b in [(2, 3), (-7, 2), (2**53 + 1, 1), (2**63 - 1, 0), (-2**63, 0)]:
            with self.subTest(a=a, b=b):
                self.assertEqual(world.checked_add(a, b), a + b)

    def test_invalid_values_and_native_overflow_become_python_exceptions(self):
        for a, b in [(2**63 - 1, 1), (-2**63, -1), (2**63, 0)]:
            with self.subTest(a=a, b=b), self.assertRaises(OverflowError):
                world.checked_add(a, b)
        for invalid in [1.5, "2"]:
            with self.subTest(invalid=invalid), self.assertRaises(TypeError):
                world.checked_add(invalid, 1)

    def test_installed_import_works_outside_repository_without_pythonpath(self):
        code = (
            "import json, world; from world import _native; "
            "print(json.dumps({'value': world.checked_add(2, 3), "
            "'version': world.core_version(), 'native': _native.__file__}))"
        )
        with tempfile.TemporaryDirectory(prefix="world-package-") as directory:
            result = subprocess.run([sys.executable, "-I", "-c", code], cwd=directory,
                                    check=True, capture_output=True, text=True)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["value"], 5)
        self.assertEqual(payload["version"], world.__version__)
        self.assertTrue(Path(payload["native"]).is_file())


if __name__ == "__main__":
    unittest.main()
