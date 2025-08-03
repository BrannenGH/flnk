use crate::link::link_files::link_files;
use crate::link::link_options::LinkOptions;
use std::{env, fs, io, path::Path, path::PathBuf};
use tempfile::{TempDir, tempdir};

/// ------------------------------------------------------------
/// helpers
/// ------------------------------------------------------------

/// A tmp dir plus a `PathBuf` pointing to a child directory we can work in.
fn create_temp_dir(name: &str) -> io::Result<(TempDir, PathBuf)> {
    let temp = tempdir()?;
    let dir_path = temp.path().join(name);
    fs::create_dir_all(&dir_path)?;
    Ok((temp, dir_path))
}

fn setup_test_env() -> io::Result<((TempDir, PathBuf), (TempDir, PathBuf))> {
    Ok((create_temp_dir("src")?, create_temp_dir("dest")?))
}

/// Create **one** file (auto-makes parent dirs).
fn create_test_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

/// Create the same `content` in *every* file from `files`.
pub fn create_test_files<I, P, C>(files: I, content: C) -> io::Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let bytes = content.as_ref(); // avoid re-calling as_ref in loop
    for p in files {
        create_test_file(p, bytes)?;
    }
    Ok(())
}

/// ------------------------------------------------------------
/// tests
/// ------------------------------------------------------------

#[test]
fn test_basic_hard_link() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([src.join("file1.txt")], b"test content")?;

    let linked = link_files(
        src.to_str().unwrap(),
        dst.to_str().unwrap(),
        Some(&LinkOptions::default()),
    )?;
    assert_eq!(linked.len(), 1);
    assert!(dst.join("file1.txt").exists());
    Ok(())
}

#[test]
fn test_relative_hard_link_with_spaces() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([src.join("myDir/file 3 to link.txt")], b"test content")?;

    let prev = env::current_dir()?;
    env::set_current_dir(&dst)?;

    let linked = link_files(
        &(src.to_str().unwrap().to_owned() + "/myDir/file 3 to link.txt"),
        ".",
        Some(&LinkOptions::default()),
    )?;

    assert_eq!(linked.len(), 1);
    assert!(dst.join("file 3 to link.txt").exists());

    env::set_current_dir(prev)?;
    Ok(())
}

#[test]
fn test_relative_hard_link_with_wildcard() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([
        src.join("myDir/file 3 to link.txt"),
        src.join("myDir/subDir/mov.mp4"),
        src.join("myDir/subDir/mov.nfo"),
    ], b"test content")?;

    let prev = env::current_dir()?;
    env::set_current_dir(&dst)?;

    let linked = link_files(
        &(src.to_str().unwrap().to_owned() + "/myDir/*"),
        ".",
        Some(&LinkOptions::default()),
    )?;

    assert_eq!(linked.len(), 3);
    assert!(dst.join("file 3 to link.txt").exists());
    assert!(dst.join("subDir/mov.mp4").exists());
    assert!(dst.join("subDir/mov.nfo").exists());

    env::set_current_dir(prev)?;
    Ok(())
}

#[test]
fn test_relative_hard_link_to_directory() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([src.join("myDir/file 3 to link.txt")], b"test content")?;

    let prev = env::current_dir()?;
    env::set_current_dir(&dst)?;

    let linked = link_files(
        &(src.to_str().unwrap().to_owned() + "/myDir"),
        ".",
        Some(&LinkOptions::default()),
    )?;

    assert_eq!(linked.len(), 1);
    assert!(dst.join("myDir/file 3 to link.txt").exists());

    env::set_current_dir(prev)?;
    Ok(())
}

#[test]
fn test_complex_hard_link() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files(
        [
            src.join("file1.txt"),
            src.join("file2.txt"),
            src.join("filesToLink/file3.txt"),
        ],
        b"test content",
    )?;

    let prev = env::current_dir()?;
    env::set_current_dir(&dst)?;

    let linked = link_files(
        src.to_str().unwrap(),
        dst.to_str().unwrap(),
        Some(&LinkOptions::default()),
    )?;
    assert_eq!(linked.len(), 3);
    assert!(dst.join("file2.txt").exists());
    assert!(dst.join("filesToLink/file3.txt").exists());

    env::set_current_dir(prev)?;
    Ok(())
}

#[test]
fn test_hard_link_to_sub_directory() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files(
        [
            src.join("myDir/file1.txt"),
            src.join("myDir/file2.txt"),
            src.join("filesToLink/file3.txt"),
            dst.join("destDir/subDir/file 10.mp4")
        ],
        b"test content",
    )?;

    let prev = env::current_dir()?;
    env::set_current_dir(&dst)?;

    let linked = link_files(
        &(src.to_str().unwrap().to_owned() + "/myDir"),
        &(dst.to_str().unwrap().to_owned() + "/destDir/subDir"),
        Some(&LinkOptions::default()),
    )?;
    assert_eq!(linked.len(), 2);
    assert!(dst.join("destDir/subDir/myDir/file1.txt").exists());
    assert!(dst.join("destDir/subDir/myDir/file2.txt").exists());

    env::set_current_dir(prev)?;
    Ok(())
}

#[test]
fn test_backup_option() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;
    let src_file = src.join("file1.txt");
    let dst_file = dst.join("file1.txt");

    create_test_files([&src_file], b"new content")?;
    create_test_files([&dst_file], b"existing content")?;

    let opts = LinkOptions {
        backup: true,
        backup_suffix: "~".into(),
        force: true,
        ..Default::default()
    };

    link_files(src.to_str().unwrap(), dst.to_str().unwrap(), Some(&opts))?;

    assert!(dst_file.exists());
    assert!(dst.join("file1.txt~").exists());
    Ok(())
}

#[test]
fn test_force_option() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;
    let src_file = src.join("file1.txt");
    let dst_file = dst.join("file1.txt");

    create_test_files([&src_file], b"new content")?;
    create_test_files([&dst_file], b"existing content")?;

    let opts = LinkOptions {
        force: true,
        ..Default::default()
    };

    link_files(src.to_str().unwrap(), dst.to_str().unwrap(), Some(&opts))?;
    assert!(dst_file.exists());
    Ok(())
}

#[test]
fn test_existing_file_no_force() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;
    let src_file = src.join("file1.txt");
    let dst_file = dst.join("file1.txt");

    create_test_files([&src_file], b"new content")?;
    create_test_files([&dst_file], b"existing content")?;

    let res = link_files(
        src.to_str().unwrap(),
        dst.to_str().unwrap(),
        Some(&LinkOptions::default()),
    );
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_symbolic_link() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([src.join("file1.txt")], b"test content")?;

    let opts = LinkOptions {
        symbolic: true,
        ..Default::default()
    };

    let linked = link_files(src.to_str().unwrap(), dst.to_str().unwrap(), Some(&opts))?;
    assert_eq!(linked.len(), 1);
    assert!(
        fs::symlink_metadata(dst.join("file1.txt"))?
            .file_type()
            .is_symlink()
    );
    Ok(())
}

#[test]
fn test_relative_symbolic_link() -> io::Result<()> {
    let ((_src_tmp, src), (_dst_tmp, dst)) = setup_test_env()?;

    create_test_files([src.join("file1.txt")], b"test content")?;

    let opts = LinkOptions {
        symbolic: true,
        relative: true,
        ..Default::default()
    };

    let linked = link_files(src.to_str().unwrap(), dst.to_str().unwrap(), Some(&opts))?;
    assert_eq!(linked.len(), 1);
    assert!(
        fs::symlink_metadata(dst.join("file1.txt"))?
            .file_type()
            .is_symlink()
    );
    Ok(())
}
