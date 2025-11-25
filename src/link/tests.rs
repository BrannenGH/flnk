use crate::link::link_files::link_files;
use crate::link::link_options::LinkOptions;
use crate::link::test_env::TestEnv;
use std::{fs, io};

/// Expectation: Behaves like ln for single files (hard link default)
#[test]
fn test_basic_hard_link() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src", "dest"], &["src/file1.txt"])?;

    let linked = link_files(
        test_env.sub_folders[0].to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&LinkOptions::default()),
    )?;

    assert_eq!(linked.len(), 1);
    assert!(test_env.sub_folders[1].join("file1.txt").exists());
    Ok(())
}

/// Expectation: `-s/--symbolic` creates a symbolic link
#[test]
fn test_basic_symbolic_link() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src", "dest"], &["src/file1.txt"])?;
    let src = &test_env.sub_folders[0];
    let dst = &test_env.sub_folders[1];

    let mut opts = LinkOptions::default();
    opts.symbolic = true;

    let linked = link_files(src.to_str().unwrap(), dst.to_str().unwrap(), Some(&opts))?;
    assert_eq!(linked.len(), 1);

    let md = fs::symlink_metadata(dst.join("file1.txt"))?;
    assert!(md.file_type().is_symlink());
    Ok(())
}

/// Expectation: rename-style path works when parent exists (DEST does not exist)
#[test]
fn test_rename_style_when_parent_exists() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src", "dest"], &["src/original.txt"])?;
    let src = test_env.sub_folders[0].to_str().unwrap();
    let dst = test_env.sub_folders[1].join("renamed.txt");

    let linked = link_files(src, dst.to_str().unwrap(), Some(&LinkOptions::default()))?;
    assert_eq!(linked.len(), 1);
    assert!(dst.exists());
    Ok(())
}

/// Expectation: fails when DEST parent directory is missing (single-file mode)
#[test]
fn test_fail_when_parent_dir_missing() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src"], &["src/original.txt"])?;
    let src = test_env.sub_folders[0].to_str().unwrap();
    let missing_parent = test_env._root_temp_dir.join("missing_parent").join("newname.txt");

    let res = link_files(src, missing_parent.to_str().unwrap(), Some(&LinkOptions::default()));
    assert!(res.is_err());
    Ok(())
}

/// Expectation: If DEST exists and is a directory, link goes inside (single file)
#[test]
fn test_single_file_into_existing_directory() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src", "dest"], &["src/a.txt"])?;
    let linked = link_files(
        test_env.sub_folders[0].join("a.txt").to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&LinkOptions::default()),
    )?;
    assert_eq!(linked.len(), 1);
    assert!(test_env.sub_folders[1].join("a.txt").exists());
    Ok(())
}

/// Expectation: Directory recursion preserves tree structure and creates directories as needed
#[test]
fn test_directory_recursive_hard_links() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(
        &["src", "dest"],
        &[
            "src/file1.txt",
            "src/file2.txt",
            "src/sub/file3.txt",
            "src/sub/nested/file4.txt",
        ],
    )?;

    let linked = link_files(
        test_env.sub_folders[0].to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&LinkOptions::default()),
    )?;

    assert_eq!(linked.len(), 4);
    assert!(test_env.sub_folders[1].join("file1.txt").exists());
    assert!(test_env.sub_folders[1].join("file2.txt").exists());
    assert!(test_env.sub_folders[1].join("sub/file3.txt").exists());
    assert!(test_env.sub_folders[1].join("sub/nested/file4.txt").exists());
    Ok(())
}

/// Expectation: Recursive with -s produces symlinks at all leaf files
#[test]
fn test_directory_recursive_symbolic_links() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(
        &["src", "dest"],
        &[
            "src/f1.txt",
            "src/f2.txt",
            "src/dirA/g1.txt",
        ],
    )?;
    let src = test_env.sub_folders[0].to_str().unwrap();
    let dst = test_env.sub_folders[1].to_str().unwrap();

    let mut opts = LinkOptions::default();
    opts.symbolic = true;

    let linked = link_files(src, dst, Some(&opts))?;
    assert_eq!(linked.len(), 3);
    assert!(fs::symlink_metadata(test_env.sub_folders[1].join("f1.txt"))?.file_type().is_symlink());
    assert!(fs::symlink_metadata(test_env.sub_folders[1].join("f2.txt"))?.file_type().is_symlink());
    assert!(fs::symlink_metadata(test_env.sub_folders[1].join("dirA/g1.txt"))?.file_type().is_symlink());
    Ok(())
}

/// Expectation: `-f` overwrites existing destination
#[test]
fn test_force_overwrite_existing() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(
        &["src", "dest"],
        &["src/a.txt", "dest/a.txt"],
    )?;

    let mut opts = LinkOptions::default();
    opts.force = true;

    let linked = link_files(
        test_env.sub_folders[0].join("a.txt").to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&opts),
    )?;
    assert_eq!(linked.len(), 1);
    assert!(test_env.sub_folders[1].join("a.txt").exists());
    Ok(())
}

/// Expectation: Without -f/-i, linking to existing file should error
#[test]
fn test_existing_file_without_force_errors() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(
        &["src", "dest"],
        &["src/a.txt", "dest/a.txt"],
    )?;

    let res = link_files(
        test_env.sub_folders[0].join("a.txt").to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&LinkOptions::default()),
    );
    assert!(res.is_err());
    Ok(())
}

/// Expectation: -T no-target-directory treats DEST as a file path even if directory exists
#[test]
fn test_no_target_directory_flag() -> Result<(), Box<dyn std::error::Error>> {
    let test_env = TestEnv::new(&["src", "dest"], &["src/foo.txt"])?;

    let mut opts = LinkOptions::default();
    opts.no_target_directory = true; // corresponds to -T

    // Try to treat 'dest' dir as a file path; should error like ln (can't link directory path as file)
    let res = link_files(
        test_env.sub_folders[0].join("foo.txt").to_str().unwrap(),
        test_env.sub_folders[1].to_str().unwrap(),
        Some(&opts),
    );
    assert!(res.is_err());
    Ok(())
}