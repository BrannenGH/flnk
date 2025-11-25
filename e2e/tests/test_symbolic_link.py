from pathlib import Path
import os

def test_symbolic_link_file(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    src.mkdir()
    dest.mkdir()
    (src / "hello.txt").write_text("hi\n")

    result = run_flnk(["-s", "src/hello.txt", "dest"])

    assert result.returncode == 0, result.stderr
    target = dest / "hello.txt"
    assert target.is_symlink()
    assert target.read_text() == "hi\n"

def test_recursive_symbolic_links(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    (src / "nested").mkdir(parents=True)
    (src / "nested" / "file.txt").write_text("data")

    result = run_flnk(["-s", "src", "dest"])

    assert result.returncode == 0, result.stderr
    linked = dest / "nested" / "file.txt"
    assert linked.is_symlink()
    assert linked.read_text() == "data"

