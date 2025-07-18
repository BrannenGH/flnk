use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A struct containing options for controlling the linking behavior.
#[derive(Debug, Clone)]
pub struct LinkOptions {
    /// If true, creates symbolic links instead of hard links
    pub symbolic: bool,
    /// If true and creating symbolic links, creates relative symbolic links
    pub relative: bool,
    /// If true, removes existing destination files
    pub force: bool,
    /// If true, creates backups of existing files
    pub backup: bool,
    /// The suffix to use for backup files
    pub backup_suffix: String,
    /// If true, prints actions as they occur
    pub verbose: bool,
    /// When true and creating symbolic links, directories will not be symbolically linked
    pub symlink_files_only: bool,
}

/// Default implementation for LinkOptions
impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            symbolic: false,
            relative: false,
            force: false,
            backup: false,
            backup_suffix: String::from("~"),
            verbose: false,
            symlink_files_only: false,
        }
    }
}

/// Computes a relative path from the source to the target.
///
/// # Arguments
///
/// * `source` - The source path to compute the relative path from
/// * `target` - The target path to compute the relative path to
///
/// # Returns
///
/// * `io::Result<PathBuf>` - The relative path from source to target
fn make_relative(source: &Path, target: &Path) -> io::Result<PathBuf> {
    let source_abs = fs::canonicalize(source)?;
    let target_abs = fs::canonicalize(target.parent().unwrap_or(target))?;

    pathdiff::diff_paths(&source_abs, &target_abs)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not compute relative path"))
}

/// Creates a backup of a file by renaming it with a suffix.
///
/// If a file with the backup name already exists, appends a counter
/// to the backup name until a unique name is found.
///
/// # Arguments
///
/// * `dest` - The path to the file to back up
/// * `suffix` - The suffix to append to the backup file name
///
/// # Returns
///
/// * `io::Result<()>` - Success if the backup was created
fn create_backup(dest: &Path, suffix: &str) -> io::Result<()> {
    let suffix = if suffix.is_empty() { "~" } else { suffix };
    let dest_str = dest.to_string_lossy();
    let mut backup_path = PathBuf::from(format!("{}{}", dest_str, suffix));

    if backup_path.exists() {
        let mut counter = 1;
        loop {
            backup_path = PathBuf::from(format!("{}.~{}~", dest_str, counter));
            if !backup_path.exists() {
                break;
            }
            counter += 1;
        }
    }

    fs::rename(dest, backup_path)
}

/// Creates either a hard link or symbolic link based on the provided options.
///
/// # Arguments
///
/// * `source_path` - The path to the source file to link from
/// * `dest_path` - The path where the link should be created
/// * `opts` - The options controlling the link behavior
///
/// # Returns
///
/// * `io::Result<PathBuf>` - The path to the created link
fn make_link(source_path: &Path, dest_path: &Path, opts: &LinkOptions) -> io::Result<PathBuf> {
    if opts.symbolic {
        let link_target = if opts.relative {
            make_relative(source_path, dest_path)?
        } else {
            source_path.to_path_buf()
        };

        std::os::unix::fs::symlink(&link_target, dest_path)?;
        Ok(dest_path.to_path_buf())
    } else {
        fs::hard_link(source_path, dest_path)?;
        Ok(dest_path.to_path_buf())
    }
}

/// Links files from a source directory to a destination directory.
///
/// Can create either hard links or symbolic links based on the options provided.
/// Handles existing files according to the backup and force options.
///
/// # Arguments
///
/// * `source` - The source directory path as a string
/// * `dest` - The destination directory path as a string
/// * `opts` - Optional link options to control the behavior
///
/// # Returns
///
/// * `io::Result<Vec<PathBuf>>` - A list of relative paths that were linked
pub fn link_files(
    source: &str,
    dest: &str,
    opts: Option<&LinkOptions>,
) -> io::Result<Vec<PathBuf>> {
    let default_opts = LinkOptions::default();
    let opts = opts.unwrap_or(&default_opts);
    let source_path = Path::new(source);
    let dest_path = Path::new(dest);
    let mut linked = Vec::new();

    for (i, entry) in WalkDir::new(source_path).into_iter().enumerate() {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        // '.' is returned as first entry, need to skip it.
        if i == 0 && metadata.is_dir() {
            continue;
        }

        // Skip non-regular files for hard links
        if !metadata.is_file() && !opts.symbolic {
            continue;
        }

        // Skip directories if symlink_files_only is true
        if metadata.is_dir() && opts.symbolic && opts.symlink_files_only {
            continue;
        }

        let rel_path = path
            .strip_prefix(source_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let dest_file = dest_path.join(rel_path);

        if let Some(parent) = dest_file.parent() {
            fs::create_dir_all(parent)?;
        }

        if metadata.is_dir() && opts.symbolic {
            make_link(path, &dest_file, opts)?;
            linked.push(rel_path.to_path_buf());
            continue;
        }

        if dest_file.exists() {
            if opts.backup {
                create_backup(&dest_file, &opts.backup_suffix)?;
            } else if opts.force {
                fs::remove_file(&dest_file)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Destination file exists",
                ));
            }
        }

        make_link(path, &dest_file, opts)?;
        linked.push(rel_path.to_path_buf());
    }

    Ok(linked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::fs::create_dir_all;
    use std::io::Write;
    use tempfile::TempDir;
    use tempfile::tempdir;

    fn create_temp_dir(name: &str) -> io::Result<(TempDir, PathBuf)> {
        let temp = tempdir()?;
        let dir_path = temp.path().join(name);
        match fs::create_dir_all(&dir_path) {
            Ok(()) => Ok((temp, dir_path)),
            Err(e) => {
                eprintln!(
                    "💥  couldn’t create dir {} → kind={:?}, os_code={:?}, msg={}",
                    &dir_path.display(),
                    e.kind(),         // high‑level category (NotFound, PermissionDenied, …)
                    e.raw_os_error(), // underlying errno if on a Unix‑like OS
                    e                 // human‑readable message
                );
                return Err(e);
            }
        }
    }

    fn setup_test_env() -> Result<((TempDir, PathBuf), (TempDir, PathBuf)), std::io::Error> {
        let src = create_temp_dir("src")?;
        let dest = create_temp_dir("dest")?;
        Ok((src, dest))
    }

    #[test]
    fn test_basic_hard_link() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let test_file = source.join("file1.txt");
        match File::create(&test_file) {
            Ok(mut f) => f.write_all(b"test content")?,
            Err(e) => {
                eprintln!(
                    "💥  couldn’t create {} → kind={:?}, os_code={:?}, msg={}",
                    &source.display(),
                    e.kind(),         // high‑level category
                    e.raw_os_error(), // underlying errno, if any
                    e                 // human‑readable message
                );
                return Err(e);
            }
        }

        let opts = LinkOptions::default();
        let linked = link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert_eq!(linked.len(), 1);
        assert!(dest.join("file1.txt").exists());
        Ok(())
    }

    #[test]
    fn test_complex_hard_link() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let test_files = [
            source.join("file1.txt"),
            source.join("file2.txt"),
            source.join("filesToLink/file3.txt"),
        ];

        for file in &test_files {
            if let Some(parent) = file.parent() {
                create_dir_all(parent)?;
            }

            match File::create(file) {
                Ok(mut f) => f.write_all(b"test content")?,
                Err(e) => {
                    eprintln!(
                        "💥  couldn’t create {} → kind={:?}, os_code={:?}, msg={}",
                        &file.display(),
                        e.kind(),         // high‑level category
                        e.raw_os_error(), // underlying errno, if any
                        e                 // human‑readable message
                    );
                    return Err(e);
                }
            }
        }

        let opts = LinkOptions::default();
        let linked = link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert_eq!(linked.len(), 3);
        assert!(dest.join("file2.txt").exists());
        assert!(dest.join("filesToLink/file3.txt").exists());
        Ok(())
    }

    #[test]
    fn test_backup_option() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let source_file = source.join("file1.txt");
        let dest_file = dest.join("file1.txt");

        File::create(&source_file)?.write_all(b"new content")?;
        File::create(&dest_file)?.write_all(b"existing content")?;

        let opts = LinkOptions {
            backup: true,
            backup_suffix: "~".to_string(),
            force: true,
            ..Default::default()
        };

        link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert!(dest_file.exists());
        assert!(dest.join("file1.txt~").exists());
        Ok(())
    }

    #[test]
    fn test_force_option() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let source_file = source.join("file1.txt");
        let dest_file = dest.join("file1.txt");

        File::create(&source_file)?.write_all(b"new content")?;
        File::create(&dest_file)?.write_all(b"existing content")?;

        let opts = LinkOptions {
            force: true,
            ..Default::default()
        };

        link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert!(dest_file.exists());
        Ok(())
    }

    #[test]
    fn test_existing_file_no_force() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let source_file = source.join("file1.txt");
        let dest_file = dest.join("file1.txt");

        File::create(&source_file)?.write_all(b"new content")?;
        File::create(&dest_file)?.write_all(b"existing content")?;

        let opts = LinkOptions::default();
        let result = link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_symbolic_link() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let test_file = source.join("file1.txt");
        File::create(&test_file)?.write_all(b"test content")?;

        let opts = LinkOptions {
            symbolic: true,
            ..Default::default()
        };

        let linked = link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert_eq!(linked.len(), 1);
        assert!(
            fs::symlink_metadata(dest.join("file1.txt"))?
                .file_type()
                .is_symlink()
        );
        Ok(())
    }

    #[test]
    fn test_relative_symbolic_link() -> io::Result<()> {
        let ((_source_dir, source), (_dest_dir, dest)) = setup_test_env()?;
        let test_file = source.join("file1.txt");
        File::create(&test_file)?.write_all(b"test content")?;

        let opts = LinkOptions {
            symbolic: true,
            relative: true,
            ..Default::default()
        };

        let linked = link_files(
            source.to_str().unwrap(),
            dest.to_str().unwrap(),
            Some(&opts),
        )?;

        assert_eq!(linked.len(), 1);
        assert!(
            fs::symlink_metadata(dest.join("file1.txt"))?
                .file_type()
                .is_symlink()
        );
        Ok(())
    }
}
