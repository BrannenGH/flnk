# flnk end-to-end tests

Python-based E2E tests exercise the CLI binary from the repo root. Each test runs in its own working directory so parallel runs (via `pytest -n auto`) stay isolated.

## Setup
- `cd e2e`
- `python3 -m venv .venv && source .venv/bin/activate`
- `pip install -r requirements.txt`

## Running
- From repo root: `cd e2e && pytest` (use `-n auto` for parallel)
- Override the binary path with `FLNK_BIN=/path/to/flnk pytest` if you already built it; otherwise the harness runs `cargo build --quiet` once per session.

Tests rely on a `test_workdir` fixture that `chdir`s into a per-test temp dir, so new files are kept out of the repo tree.
