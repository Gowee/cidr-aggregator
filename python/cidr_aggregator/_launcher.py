"""Find and exec the platform-specific cidr-aggregator binary."""

import os
import platform
import sys
from pathlib import Path


def _platform_dir() -> str:
    system = platform.system()
    machine = platform.machine()

    if system == "Linux" and machine == "x86_64":
        return "linux-x64"
    if system == "Linux" and machine == "aarch64":
        return "linux-arm64"
    if system == "Darwin" and machine == "x86_64":
        return "macos-x64"
    if system == "Darwin" and machine == "arm64":
        return "macos-arm64"
    if system == "Windows" and machine == "AMD64":
        return "windows-x64"

    print(f"Unsupported platform: {system}-{machine}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    bin_dir = Path(__file__).parent / "_bin" / _platform_dir()
    ext = ".exe" if platform.system() == "Windows" else ""
    binary = bin_dir / f"cidr-aggregator{ext}"

    if not binary.exists():
        print(f"Binary not found: {binary}", file=sys.stderr)
        print("The package may not include a binary for your platform.", file=sys.stderr)
        sys.exit(1)

    os.execvp(str(binary), [str(binary)] + sys.argv[1:])
