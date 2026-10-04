# SPDX-License-Identifier: GPL-3.0-only
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from publish_prerelease import validate_request, write_metadata


class PrereleaseTests(unittest.TestCase):
    def test_only_matching_explicit_development_tags_are_accepted(self):
        sha = "a" * 40
        for stage in ["alpha", "beta", "rc"]:
            tag = f"v0.1.0-{stage}.1"
            self.assertEqual(validate_request({"tag": tag}, "0.1.0", sha), tag)
        for tag in ["v0.1.0", "v1.0.0-alpha.1", "v0.1.0-alpha.0", "../bad", 1]:
            with self.assertRaises(ValueError):
                validate_request({"tag": tag}, "0.1.0", sha)
        for request in [{}, {"tag": "v0.1.0-alpha.1", "force": True}]:
            with self.assertRaises(ValueError):
                validate_request(request, "0.1.0", sha)
        with self.assertRaises(ValueError):
            validate_request({"tag": "v0.1.0-alpha.1"}, "0.1.0", "main")

    def test_checksums_cover_packages_and_provenance_metadata(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp)
            (output / "test.apk").write_bytes(b"test-only fixture")
            write_metadata(output, "v0.1.0-alpha.1", "a" * 40, {})
            metadata = json.loads((output / "build-info.json").read_text())
            self.assertEqual(metadata["source_commit"], "a" * 40)
            self.assertEqual(metadata["device_validation"], [])
            checked = set()
            for line in (output / "checksums.txt").read_text().splitlines():
                checksum, name = line.split("  ")
                checked.add(name)
                self.assertEqual(checksum, hashlib.sha256((output / name).read_bytes()).hexdigest())
            self.assertEqual(checked, {"test.apk", "build-info.json", "DEVELOPMENT.txt"})
