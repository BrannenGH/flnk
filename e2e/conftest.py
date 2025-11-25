from __future__ import annotations

import functools
import os
import subprocess
from pathlib import Path
from typing import Iterable, Mapping, Optional

import pytest


REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BIN = REPO_ROOT / "target" / "debug" / ("flnk.exe" if os.name == "nt" else "flnk")


@functools.lru_cache(maxsize=1)
def _ensure_flnk_built() -> Path:
    env_bin = os.environ.get("FLNK_BIN")
    if env_bin:
        return Path(env_bin)

    subprocess.run(
        ["cargo", "build", "--quiet"],
        cwd=REPO_ROOT,
        check=True,
    )
    return DEFAULT_BIN


@pytest.fixture(scope="session")
def flnk_bin() -> Path:
    bin_path = _ensure_flnk_built()
    if not bin_path.exists():
        raise FileNotFoundError(f"flnk binary not found at {bin_path}")
    return bin_path


@pytest.fixture
def test_workdir(tmp_path_factory: pytest.TempPathFactory, monkeypatch: pytest.MonkeyPatch, request: pytest.FixtureRequest) -> Path:
    workdir = tmp_path_factory.mktemp(f"flnk-{request.node.name}")
    monkeypatch.chdir(workdir)
    return workdir


@pytest.fixture
def run_flnk(flnk_bin: Path):
    def _run(
        args: Iterable[str],
        *,
        cwd: Optional[Path] = None,
        env: Optional[Mapping[str, str]] = None,
    ) -> subprocess.CompletedProcess[str]:
        proc_env = os.environ.copy()
        if env:
            proc_env.update(env)

        return subprocess.run(
            [str(flnk_bin), *args],
            cwd=cwd,
            env=proc_env,
            text=True,
            capture_output=True,
        )

    return _run
