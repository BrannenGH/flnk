/// CLI options for the recursive linking utility, intentionally limited to
/// GNU `ln`-compatible flags as agreed.
#[derive(Debug, Clone)]
pub struct LinkOptions {
    /// -s, --symbolic: create symbolic links instead of hard links (default false)
    pub symbolic: bool,
    /// -f, --force: remove existing destination paths before linking
    pub force: bool,
    /// -n, --no-dereference: treat DEST that is a symlink to a directory as a normal file
    pub no_dereference: bool,
    /// -v, --verbose: print actions performed
    pub verbose: bool,
    /// -T, --no-target-directory: treat DEST as a normal file even if it exists as a directory
    pub no_target_directory: bool,
    /// -i, --interactive: prompt before overwrite (mutually exclusive with --force in non-TTY)
    pub interactive: bool,
}

impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            symbolic: false,
            force: false,
            no_dereference: false,
            verbose: false,
            no_target_directory: false,
            interactive: false
        }
    }
}
