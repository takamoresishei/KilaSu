# SPDX-License-Identifier: GPL-3.0-only
from pathlib import Path
import hashlib
import struct
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import package as packaging

class PackagingTests(unittest.TestCase):
    def fixture(self, root):
        """Synthetic format fixtures are never published as Android artifacts."""
        apk = root / "test.apk"
        with zipfile.ZipFile(apk, "w") as archive:
            archive.writestr("AndroidManifest.xml", b"test-only manifest")
            archive.writestr("classes.dex", b"test-only dex")
        elf = bytearray(64)
        elf[:6] = b"\x7fELF\x02\x01"
        struct.pack_into("<HH", elf, 16, 3, 183)
        daemon = root / "daemon"
        daemon.mkdir()
        for name in ["kila", "kilasd"]:
            (daemon / name).write_bytes(elf)
        validator = root / "kila-boot"
        validator.write_bytes(elf)
        return apk, daemon, validator

    def test_packages_preserve_identity_permissions_notices_and_are_deterministic(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            apk, daemon, validator = self.fixture(root)
            for output in [root / "one", root / "two"]:
                packaging.package("v0.1.0-dev.test", apk, daemon, validator, output)
            first, second = root / "one", root / "two"
            self.assertEqual([p.name for p in sorted(first.iterdir())], [p.name for p in sorted(second.iterdir())])
            for path in first.iterdir():
                self.assertEqual(path.read_bytes(), (second / path.name).read_bytes())
            self.assertEqual((first / "KilaSU-Manager-v0.1.0-dev.test.apk").read_bytes(), apk.read_bytes())
            with zipfile.ZipFile(first / "KilaSU-Installer-v0.1.0-dev.test.zip") as archive:
                pin = archive.read("payload/manager.prop").decode()
                self.assertIn(hashlib.sha256(apk.read_bytes()).hexdigest(), pin)
                self.assertEqual(archive.getinfo("payload/kilasd").external_attr >> 16 & 0o777, 0o755)
                self.assertIn("LICENSES/Zlib.txt", archive.namelist())
            lines = (first / "checksums.txt").read_text().splitlines()
            self.assertEqual(len(lines), 4)
            for line in lines:
                digest, name = line.split("  ")
                self.assertEqual(digest, hashlib.sha256((first / name).read_bytes()).hexdigest())

    def test_wrong_architecture_and_non_executable_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            _, _, binary = self.fixture(root)
            for field, value in [(18, 62), (16, 1)]:
                header = bytearray(binary.read_bytes())
                struct.pack_into("<H", header, field, value)
                wrong = root / "wrong"
                wrong.write_bytes(header)
                with self.assertRaises(ValueError):
                    packaging.check_arm64_elf(wrong)

    def test_invalid_apk_and_version_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            apk, daemon, validator = self.fixture(root)
            with self.assertRaises(ValueError):
                packaging.package("../wrong", apk, daemon, validator, root / "out")
            with zipfile.ZipFile(apk, "w") as archive:
                archive.writestr("wrong", "not an APK")
            with self.assertRaises(ValueError):
                packaging.package("v0.1.0", apk, daemon, validator, root / "out")
