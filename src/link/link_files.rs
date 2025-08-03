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

fn has_glob(pattern: &str) -> bool {
    pattern.chars().any(|c| matches!(c, '*' | '?' | '['))
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }

    let mut parts = pattern.split('*');
    let first = parts.next().unwrap();
    if !text.starts_with(first) {
        return false;
    }
    let mut remainder = &text[first.len()..];
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if let Some(pos) = remainder.find(part) {
            remainder = &remainder[pos + part.len()..];
        } else {
            return false;
        }
    }
    pattern.ends_with('*') || remainder.is_empty()
}

fn expand_sources(pattern: &str) -> io::Result<Vec<PathBuf>> {
    if !has_glob(pattern) {
        return Ok(vec![PathBuf::from(pattern)]);
    }
    let path = Path::new(pattern);
    let dir = path.parent().unwrap_or(Path::new("."));
    let pat = path.file_name().unwrap_or_default().to_string_lossy();
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if wildcard_match(&pat, &name.to_string_lossy()) {
            out.push(entry.path());
        }
    }
    Ok(out)
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
    let dest_path = Path::new(dest);
    let dest_is_dir = dest_path.is_dir();
    let include_root = dest_path.is_relative();
    let mut linked = Vec::new();

    let sources = expand_sources(source)?;

    for source_path in sources {
        let base = if include_root && dest_is_dir {
            source_path.parent().unwrap_or(Path::new(""))
        } else {
            source_path.as_path()
        };

        for (i, entry) in WalkDir::new(&source_path).into_iter().enumerate() {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if i == 0 && metadata.is_dir() {
                continue;
            }

            if !metadata.is_file() && !opts.symbolic {
                continue;
            }

            if metadata.is_dir() && opts.symbolic && opts.symlink_files_only {
                continue;
            }

            let rel_path = path
                .strip_prefix(base)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let dest_file = if rel_path.as_os_str().is_empty() && dest_is_dir {
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
    }

    Ok(linked)
}
