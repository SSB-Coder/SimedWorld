from pathlib import Path

from ._native import FixtureWorld


def load_fixture(path: str | Path) -> FixtureWorld:
    return FixtureWorld(Path(path).read_text(encoding="utf-8"))
