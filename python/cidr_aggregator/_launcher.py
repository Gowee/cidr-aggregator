"""Find and exec the platform-specific cidr-aggregator binary."""

import os
import platform
import sys
from pathlib import Path


def _platform_dir() -> str:
    system = platform.system()
    raw_machine = platform.machine()
    machine = raw_machine.lower()

    if system == "Linux":
        if machine in ("x86_64", "amd64"):
            return "linux-x64"
        if machine in ("aarch64", "arm64"):
            return "linux-arm64"
    elif system == "Darwin":
        if machine in ("x86_64", "amd64"):
            return "macos-x64"
        if machine in ("arm64", "aarch64"):
            return "macos-arm64"
    elif system == "Windows":
        # Windows on ARM64 runs windows-x64 via built-in x64 emulation
        if machine in ("amd64", "x86_64", "arm64"):
            return "windows-x64"

    print(f"Unsupported platform: {system}-{raw_machine}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    bin_dir = Path(__file__).parent / "_bin" / _platform_dir()
    ext = ".exe" if platform.system() == "Windows" else ""
    binary = bin_dir / f"cidr-aggregator{ext}"

    if not binary.exists():
        print(f"Binary not found: {binary}", file=sys.stderr)
        print(
            "The package may not include a binary for your platform.",
            file=sys.stderr,
        )
        sys.exit(1)

    args = [str(binary)] + sys.argv[1:]

    if platform.system() == "Windows":
        import subprocess

        try:
            res = subprocess.run(args)
            sys.exit(res.returncode)
        except KeyboardInterrupt:
            sys.exit(130)
    else:
        os.execvp(str(binary), args)


if __name__ == "__main__":
    main()
