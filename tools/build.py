#!/usr/bin/env python3
"""Build OPGAME and assemble a portable runtime directory."""

from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "data"
MODELS = ROOT / "models"
DIST = ROOT / "dist"


def main() -> int:
    cargo = "cargo"
    result = subprocess.run([cargo, "build", "--release"], cwd=ROOT)
    if result.returncode != 0:
        return result.returncode

    binary_name = "opgame.exe" if sys.platform == "win32" else "opgame"
    binary = ROOT / "target" / "release" / binary_name
    if not binary.exists():
        print(f"Build succeeded, but binary was not found: {binary}", file=sys.stderr)
        return 1

    # Keep user settings across builds when a packaged runtime already exists.
    runtime_settings = DIST / "data" / "config" / "settings.toml"
    saved_settings = runtime_settings.read_bytes() if runtime_settings.exists() else None

    if DIST.exists():
        shutil.rmtree(DIST)
    DIST.mkdir(parents=True)

    output_binary = DIST / binary_name
    shutil.copy2(binary, output_binary)

    # The packaged runtime mirrors the source tree instead of adding another
    # assets/ wrapper directory:
    #
    # dist/
    #   opgame.exe
    #   models/
    #   data/
    shutil.copytree(MODELS, DIST / "models")
    shutil.copytree(DATA, DIST / "data")

    if saved_settings is not None:
        settings_path = DIST / "data" / "config" / "settings.toml"
        settings_path.parent.mkdir(parents=True, exist_ok=True)
        settings_path.write_bytes(saved_settings)

    print(f"Runtime assembled in: {DIST}")
    print(f"  binary: {output_binary.name}")
    print(f"  models: {DIST / 'models'}")
    print(f"  data:   {DIST / 'data'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
