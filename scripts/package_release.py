"""Assemble already-built Windows binaries and reproducible model inputs."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile


def runtime_readme(source):
    return (
        """# Windows runtime package

Use CPython 3.13 x64. Install with `scripts/install_release.ps1`, then launch
`scripts/run_release.ps1`. Add `-Headless -Decisions 300` for a saved headless run.
To reuse an environment with this exact wheel already installed, supply
`-Python C:\\path\\to\\environment\\Scripts\\python.exe`; this avoids another copy
of PyTorch. The packaged viewer needs no Rust build cache or Rust toolchain.

The source-project guide follows. Its build commands and historical artifact
links refer to the source checkout, rather than files included in this runtime.

"""
        + source
    )


def package(root, destination):
    root = Path(root).resolve()
    destination = Path(destination).resolve()
    if destination.exists():
        raise ValueError("release destination already exists")
    archive = Path(str(destination) + ".zip")
    if archive.exists():
        raise ValueError("release archive already exists")
    wheel = next((root / "artifacts/wheels").glob("world_microcosm-*.whl"))
    viewer = root / "target/debug/world_viewer.exe"
    if not viewer.is_file():
        raise FileNotFoundError("build world_viewer before packaging")
    destination.mkdir(parents=True)
    files = [
        wheel,
        viewer,
        *sorted(root.glob("*.md")),
        root / "docs/data_formats.md",
        root / "docs/resource_accounting.md",
        root / "dependencies/python-win-py313.lock",
        root / "scripts/install_release.ps1",
        root / "scripts/run_release.ps1",
        *sorted((root / "configs").glob("*.json")),
        *sorted((root / "examples").glob("*.py")),
    ]
    manifest = {
        "schema_version": 1,
        "kind": "world.windows-release",
        "python": "CPython 3.13 x64",
        "files": {},
        "controller": "independent plastic sparse RNN or custom sparse LTC; official ncps LTC/CfC are fixed-wiring experimental controls",
        "build_profile": "optimized workspace packages in dev profile",
        "dependencies": "Python packages installed separately using the bundled hash-locked requirements; no Rust toolchain needed at runtime",
    }
    for source in files:
        relative = (
            Path("wheels") / source.name
            if source == wheel
            else Path("bin/world_viewer.exe")
            if source == viewer
            else source.relative_to(root)
        )
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        if source.name == "README.md":
            target.write_text(runtime_readme(source.read_text()), encoding="utf-8")
        manifest["files"][relative.as_posix()] = hashlib.sha256(
            target.read_bytes()
        ).hexdigest()
    (destination / "manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED) as stream:
        for source in sorted(destination.rglob("*")):
            if source.is_file():
                stream.write(
                    source, Path(destination.name) / source.relative_to(destination)
                )
    return archive


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    print(package(Path(__file__).resolve().parents[1], args.output))
