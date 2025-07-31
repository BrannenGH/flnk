use crate::link::link_options::LinkOptions;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

        let dest_file = if rel_path.as_os_str().is_empty() && dest_path.is_dir() {
            dest_path.join(path.file_name().unwrap())
        } else {
            dest_path.join(rel_path)
        };
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
