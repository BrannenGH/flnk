from pathlib import Path

def test_basic_hard_link(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    src.mkdir()
    dest.mkdir()

    (src / "hello.txt").write_text("hi\n")

    result = run_flnk(["src/hello.txt", "dest"])

    assert result.returncode == 0, result.stderr
    target = dest / "hello.txt"
    assert target.exists()
    assert target.read_text() == "hi\n"
