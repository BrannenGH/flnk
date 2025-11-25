use clap::{Arg, ArgAction, Command};
use flnk::link::link_files::link_files;
use flnk::link::link_options::LinkOptions;

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");

    let matches = Command::new(name)
        .version(version)
        .about("Recursive ln-compatible linking utility")
        .arg(
            Arg::new("symbolic")
                .short('s')
                .long("symbolic")
                .help("Make symbolic links instead of hard links")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("Remove existing destination files before linking")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_dereference")
                .short('n')
                .long("no-dereference")
                .help("Treat DEST that is a symlink to a directory as a normal file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print actions performed")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_target_directory")
                .short('T')
                .long("no-target-directory")
                .help("Treat DEST as a normal file even if it exists as a directory")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Prompt before overwrite (mutually exclusive with --force in non-TTY)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("source")
                .help("Source file or directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("dest")
                .help("Destination file or directory")
                .required(false)
                .index(2),
        )
        .get_matches();

    let opts = LinkOptions {
        symbolic: matches.get_flag("symbolic"),
        force: matches.get_flag("force"),
        no_dereference: matches.get_flag("no_dereference"),
        verbose: matches.get_flag("verbose"),
        no_target_directory: matches.get_flag("no_target_directory"),
        interactive: matches.get_flag("interactive")
    };

    let source = matches
        .get_one::<String>("source")
        .expect("missing source argument");
    let dest = matches.get_one::<String>("dest").map(|s| s.as_str());

    // Perform linking
    if let Err(err) = link_files(source, dest.unwrap_or("."), Some(&opts)) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}