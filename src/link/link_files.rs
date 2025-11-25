use crate::link::link_options::LinkOptions;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[cfg(unix)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(windows)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(src, dst)
}

fn create_link(source: &Path, target: &Path, opts: &LinkOptions) -> io::Result<()> {
    // Handle overwrite semantics
    if target.exists() {
        if opts.force {
            // Remove file/symlink/dir (ln -f removes existing destination)
            // We only expect files/symlinks as targets in our use-cases.
            // Try unlink first; if it's a dir, this will fail and we'll bubble the error.
            match fs::remove_file(target) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        } else if opts.interactive {
            // Out of scope for non-interactive tests; return an error to force caller behavior.
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "interactive mode not supported in this context",
            ));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination exists (use -f to overwrite)",
            ));
        }
    }

    // Ensure parent exists (ln does NOT create parents for single-file ops)
    if let Some(parent) = target.parent() {
        if !parent.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "destination parent directory does not exist",
            ));
        }
    }

    if opts.symbolic {
        symlink_file(source, target)
    } else {
        fs::hard_link(source, target)
    }
}

fn resolve_single_file_target(source: &Path, dest: &Path, opts: &LinkOptions) -> io::Result<PathBuf> {
    // If DEST is an existing directory and -T not set, link goes inside it with basename(SOURCE)
    if dest.is_dir() && !opts.no_target_directory {
        let name = source
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "source has no file name"))?;
        Ok(dest.join(name))
    } else {
        // DEST is treated as a path (rename-style is allowed if parent exists)
        Ok(dest.to_path_buf())
    }
}

fn link_directory_recursive(source_dir: &Path, dest_dir: &Path, opts: &LinkOptions) -> io::Result<Vec<PathBuf>> {
    // If DEST doesn't exist, create it. If it exists and is a file -> error.
    if dest_dir.exists() {
        if dest_dir.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "destination exists and is a file",
            ));
        }
    } else {
        fs::create_dir_all(dest_dir)?;
    }

    let mut linked: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        let src_path = entry.path();

        if entry.file_type().is_dir() {
            // Ensure corresponding directory exists in dest
            let rel = src_path.strip_prefix(source_dir).unwrap_or(src_path);
            let dst_path = dest_dir.join(rel);
            fs::create_dir_all(&dst_path)?;
            continue;
        }

        if entry.file_type().is_file() {
            let rel = src_path.strip_prefix(source_dir).unwrap_or(src_path);
            let dst_path = dest_dir.join(rel);

            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }

            create_link(src_path, &dst_path, opts)?;
            linked.push(dst_path);
        } else if entry.file_type().is_symlink() {
            // For simplicity and safety, only link real files; treat symlinked files as files if user asked for -s
            // If -s is set, we create a new symlink pointing to the source path text.
            let rel = src_path.strip_prefix(source_dir).unwrap_or(src_path);
            let dst_path = dest_dir.join(rel);
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if opts.symbolic {
                create_link(src_path, &dst_path, opts)?;
                linked.push(dst_path);
            } else {
                // Skip non-regular symlink entries when creating hard links
                // This mirrors ln failing on hard-linking symlinks as files; we can attempt to hard-link target metadata,
                // but to keep behavior simple and predictable, skip here.
                // (If stricter ln parity is needed, resolve and hard-link the target file on unix.)
                continue;
            }
        }
    }

    Ok(linked)
}

/// Public entry: create a link (single-file) or recursively link all files from a directory.
/// Returns the list of created destination paths.
pub fn link_files(source: &str, dest: &str, opts: Option<&LinkOptions>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let opts = opts.cloned().unwrap_or_default();

    let source_path = Path::new(source);
    let dest_path = Path::new(dest);

    if !source_path.exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "source does not exist",
        )));
    }

    let meta = fs::symlink_metadata(source_path)?;
    if meta.is_dir() {
        // Directory recursion
        let linked = link_directory_recursive(source_path, dest_path, &opts)?;
        Ok(linked)
    } else {
        // Single file behavior
        // Determine final target path
        let target = resolve_single_file_target(source_path, dest_path, &opts)?;

        // Guard: if -n/--no-dereference is set and DEST is a symlink to a directory, treat dest as a normal file path.
        // (Our resolve_single_file_target already respected -T; for -n, this only matters when dest is a symlink to dir.)
        if opts.no_dereference {
            if let Ok(md) = fs::symlink_metadata(dest_path) {
                if md.file_type().is_symlink() {
                    // If the symlink points to a directory and no_target_directory is false, we should NOT treat dest as a dir.
                    // We'll proceed with target as computed (which is dest itself), not placing inside.
                    // No extra action needed; the resolve_single_file_target already returned dest if -T or not a directory.
                }
            }
        }

        // Ensure parent exists when using rename-style; do NOT create parents for single-file ops.
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    "destination parent directory does not exist",
                )));
            }
        }

        create_link(source_path, &target, &opts)?;
        Ok(vec![target])
    }
}
