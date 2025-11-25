from pathlib import Path
import os

def test_force_overwrites_existing_file(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    src.mkdir()
    dest.mkdir()
    (src / "note.txt").write_text("new\n")
    existing = dest / "note.txt"
    existing.write_text("old\n")

    result = run_flnk(["-f", "src/note.txt", "dest"])

    assert result.returncode == 0, result.stderr
    assert existing.read_text() == "new\n"
    # hard link: same inode
    assert os.stat(src / "note.txt").st_ino == os.stat(existing).st_ino


def test_existing_without_force_fails(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    src.mkdir()
    dest.mkdir()
    (src / "a.txt").write_text("a")
    (dest / "a.txt").write_text("b")

    result = run_flnk(["src/a.txt", "dest"])

    assert result.returncode != 0
    assert "destination exists" in result.stderr.lower()


def test_missing_parent_errors(run_flnk, test_workdir: Path):
    (test_workdir / "src.txt").write_text("hi")
    result = run_flnk(["src.txt", "missing/child.txt"])

    assert result.returncode != 0
    assert "parent" in result.stderr.lower()


def test_recursive_directory_hard_links(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    (src / "sub").mkdir(parents=True)
    (src / "root.txt").write_text("root")
    (src / "sub" / "child.txt").write_text("child")

    result = run_flnk(["src", "dest"])

    assert result.returncode == 0, result.stderr
    assert (dest / "root.txt").read_text() == "root"
    assert (dest / "sub" / "child.txt").read_text() == "child"
    # ensure hard link for at least one file
    assert os.stat(src / "root.txt").st_ino == os.stat(dest / "root.txt").st_ino

def test_no_target_directory_errors_when_dest_dir_exists(run_flnk, test_workdir: Path):
    src = test_workdir / "src"
    dest = test_workdir / "dest"
    src.mkdir()
    dest.mkdir()
    (src / "file.txt").write_text("x")

    result = run_flnk(["-T", "src/file.txt", "dest"])

    assert result.returncode != 0
    assert "destination exists" in result.stderr.lower()
