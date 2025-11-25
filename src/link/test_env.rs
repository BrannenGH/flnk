#[cfg(test)]
use std::{env, fs, io, path::Path, path::PathBuf};
#[cfg(test)]
use tempfile::{TempDir, tempdir};

#[cfg(test)]
pub struct TestEnv {
    prev_dir: PathBuf,
    pub _root_temp_dir: TempDir,
    pub sub_folders: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}

#[cfg(test)]
impl TestEnv {
    pub fn new(directories: &[&str], files: &[&str]) -> io::Result<Self> {
        let temp = tempdir()?;
        let mut sub_folders = Vec::new();
        let mut created_files = Vec::new();

        for name in directories {
            let dir_path = temp.path().join(name);
            fs::create_dir_all(&dir_path)?;
            sub_folders.push(dir_path);
        }

        for file_name in files {
            let file_path = temp.path().join(file_name);
            create_test_file(&file_path, b"test content")?;
            created_files.push(file_path);
        }

        // Save current working directory
        let prev_dir = env::current_dir()?;

        // Automatically cd into the last directory (e.g. "dest")
        if let Some(last) = sub_folders.last() {
            env::set_current_dir(last)?;
        }

        Ok(TestEnv {
            _root_temp_dir: temp,
            sub_folders,
            files: created_files,
            prev_dir: prev_dir,
        })
    }
}

#[cfg(test)]
impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.prev_dir);
    }
}

/// Create **one** file (auto-makes parent dirs).
#[cfg(test)]
fn create_test_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}
