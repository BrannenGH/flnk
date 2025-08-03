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
            symlink_files_only: false,
        }
    }
}
