"""Tests for platform resolution and launcher behavior."""

import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from cidr_aggregator import _launcher


class TestPlatformDir(unittest.TestCase):
    @patch("platform.system", return_value="Linux")
    @patch("platform.machine", return_value="x86_64")
    def test_linux_x86_64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "linux-x64")

    @patch("platform.system", return_value="Linux")
    @patch("platform.machine", return_value="aarch64")
    def test_linux_aarch64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "linux-arm64")

    @patch("platform.system", return_value="Linux")
    @patch("platform.machine", return_value="arm64")
    def test_linux_arm64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "linux-arm64")

    @patch("platform.system", return_value="Darwin")
    @patch("platform.machine", return_value="x86_64")
    def test_macos_x86_64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "macos-x64")

    @patch("platform.system", return_value="Darwin")
    @patch("platform.machine", return_value="arm64")
    def test_macos_arm64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "macos-arm64")

    @patch("platform.system", return_value="Windows")
    @patch("platform.machine", return_value="AMD64")
    def test_windows_amd64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "windows-x64")

    @patch("platform.system", return_value="Windows")
    @patch("platform.machine", return_value="ARM64")
    def test_windows_arm64(self, _m, _s):
        self.assertEqual(_launcher._platform_dir(), "windows-x64")

    @patch("sys.stderr")
    @patch("platform.system", return_value="FreeBSD")
    @patch("platform.machine", return_value="amd64")
    def test_unsupported_os(self, _m, _s, _err):
        with self.assertRaises(SystemExit) as cm:
            _launcher._platform_dir()
        self.assertEqual(cm.exception.code, 1)

    @patch("sys.stderr")
    @patch("platform.system", return_value="Linux")
    @patch("platform.machine", return_value="i686")
    def test_unsupported_arch(self, _m, _s, _err):
        with self.assertRaises(SystemExit) as cm:
            _launcher._platform_dir()
        self.assertEqual(cm.exception.code, 1)


class TestMain(unittest.TestCase):
    @patch("platform.system", return_value="Windows")
    @patch.object(_launcher, "_platform_dir", return_value="windows-x64")
    @patch.object(Path, "exists", return_value=True)
    @patch("subprocess.run", return_value=SimpleNamespace(returncode=42))
    def test_windows_propagates_exit_code(self, _mock_run, _mock_exists, _mock_pdir, _mock_sys):
        with self.assertRaises(SystemExit) as cm:
            _launcher.main()
        self.assertEqual(cm.exception.code, 42)

    @patch("platform.system", return_value="Windows")
    @patch.object(_launcher, "_platform_dir", return_value="windows-x64")
    @patch.object(Path, "exists", return_value=True)
    @patch("subprocess.run", side_effect=KeyboardInterrupt)
    def test_windows_keyboard_interrupt(self, _mock_run, _mock_exists, _mock_pdir, _mock_sys):
        with self.assertRaises(SystemExit) as cm:
            _launcher.main()
        self.assertEqual(cm.exception.code, 130)

    @patch("platform.system", return_value="Linux")
    @patch.object(_launcher, "_platform_dir", return_value="linux-x64")
    @patch.object(Path, "exists", return_value=True)
    @patch("os.execvp")
    def test_posix_execvp(self, mock_execvp, _mock_exists, _mock_pdir, _mock_sys):
        _launcher.main()
        mock_execvp.assert_called_once()

    @patch("sys.stderr")
    @patch("platform.system", return_value="Linux")
    @patch.object(_launcher, "_platform_dir", return_value="linux-x64")
    @patch.object(Path, "exists", return_value=False)
    def test_binary_not_found(self, _mock_exists, _mock_pdir, _mock_sys, _err):
        with self.assertRaises(SystemExit) as cm:
            _launcher.main()
        self.assertEqual(cm.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
