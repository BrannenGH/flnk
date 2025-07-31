use clap::{Arg, ArgAction, Command};
use flnk::link::link_files::link_files;
use flnk::link::link_options::LinkOptions;
use flnk::ui;
use std::path::PathBuf;
use std::process;

fn main() {
    let matches = Command::new("flnk")
        .arg(
            Arg::new("symbolic")
                .short('s')
                .long("symbolic")
                .help("make symbolic links instead of hard links")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .help("remove existing destination files")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("backup")
                .short('b')
                .help("make a backup of each existing destination file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("relative")
                .short('r')
                .long("relative")
                .help("with -s, create links relative to link location")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("print name of each linked file")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("target-directory")
                .short('t')
                .help("specify the DIRECTORY in which to create the links")
                .value_name("DIRECTORY"),
        )
        .arg(
            Arg::new("suffix")
                .short('S')
                .help("override the usual backup suffix")
                .default_value("~"),
        )
        .arg(
            Arg::new("ui-mode")
                .short('u')
                .help("run in ui mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("targets")
                .required_unless_present("ui-mode")
                .num_args(1..)
                .value_name("TARGET"),
        )
        .get_matches();

    let opts = LinkOptions {
        symbolic: matches.get_flag("symbolic"),
        force: matches.get_flag("force"),
        backup: matches.get_flag("backup"),
        relative: matches.get_flag("relative"),
        backup_suffix: matches.get_one::<String>("suffix").unwrap().clone(),
        symlink_files_only: false,
    };

    let targets: Vec<&String> = matches
        .get_many::<String>("targets")
        .map(|v| v.collect())
        .unwrap_or_default();

    if matches.get_flag("ui-mode") {
        if let Err(err) = ui::run_ui(&Vec::new()) {
            eprintln!("Error in UI mode: {}", err);
            process::exit(1);
        }
        return;
    }

    let result = if let Some(target_dir) = matches.get_one::<String>("target-directory") {
        link_multiple_to_directory(&targets, target_dir, &opts)
    } else if targets.len() == 1 {
        handle_link_files(targets[0], ".", &opts)
    } else if targets.len() == 2 {
        let (target, link_name) = (targets[0], targets[1]);
        if PathBuf::from(link_name).is_dir() {
            let new_link =
                PathBuf::from(link_name).join(PathBuf::from(target).file_name().unwrap());
            handle_link_files(target, new_link.to_str().unwrap(), &opts)
        } else {
            handle_link_files(target, link_name, &opts)
        }
    } else {
        let dir = targets.last().unwrap();
        link_multiple_to_directory(&targets[..targets.len() - 1], dir, &opts)
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn handle_link_files(target: &str, link_name: &str, opts: &LinkOptions) -> Result<(), String> {
    match link_files(target, link_name, Some(opts)) {
        Ok(linked_files) => {
            for file in linked_files {
                println!("Created link: {}", file.display());
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

fn link_multiple_to_directory(
    targets: &[&String],
    dir: &str,
    opts: &LinkOptions,
) -> Result<(), String> {
    for target in targets {
        handle_link_files(target, dir, opts)?;
    }
    Ok(())
}
