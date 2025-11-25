from __future__ import annotations

import os
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Mapping


@dataclass
class CommandResult:
    code: int
    stdout: str
    stderr: str


def _build_debug_binary(project_root: Path) -> None:
    subprocess.run(["cargo", "build"], check=True, cwd=project_root)


def resolve_binary(project_root: Path) -> Path:
    """Return the flnk binary to invoke for tests, building it if needed."""
    env_override = os.environ.get("FLNK_BIN")
    if env_override:
        candidate = Path(env_override).expanduser()
        if not candidate.is_file():
            raise FileNotFoundError(
                f"FLNK_BIN is set to {candidate}, but no executable was found there"
            )
        return candidate

    binary = project_root / "target" / "debug" / "flnk"
    if not binary.exists():
        _build_debug_binary(project_root)
    return binary


def run_cli(
    binary: Path,
    args: Iterable[str],
    *,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
) -> CommandResult:
    command = [str(binary), *args]
    run_env = os.environ.copy()
    if env:
        run_env.update(env)

    completed = subprocess.run(
        command,
        cwd=cwd,
        env=run_env,
        capture_output=True,
        text=True,
    )
    return CommandResult(
        code=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
    )
