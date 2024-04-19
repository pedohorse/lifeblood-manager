use lifeblood_manager::InstallationsData;
use std::{
    env::{self, Args},
    io::Error,
    path::PathBuf, str::FromStr,
};

const MAIN_HELP_MESSAGE: &str = "\
Usage:
    lifeblood-manager-cli <command> <arguments>

    Commangs:
        - installs
";

fn main() -> Result<(), Error> {
    let mut args = env::args();
    if args.len() <= 1 {
        eprintln!("not enough arguments.");
        eprintln!("{}", MAIN_HELP_MESSAGE);
        std::process::exit(2);
    }
    args.next().unwrap(); // skip self
    let command = args.next().unwrap(); // we just checked len, so this can't fail

    match command.as_str() {
        "installs" => process_installs(args),
        _ => {
            eprintln!("invalid command");
            eprintln!("{}", MAIN_HELP_MESSAGE);
            std::process::exit(1);
        }
    }
}

const INSTALL_HELP_MESSAGE: &str = "\
Usage:
    lifeblood-manager-cli installs <subcommand> <args> base_path

    Sub-Commangs:
        - list
        - new
";

fn process_installs(mut args: Args) -> Result<(), Error> {
    if args.len() < 1 {
        eprintln!("not enough arguments.");
        eprintln!("{}", INSTALL_HELP_MESSAGE);
        std::process::exit(2);
    }

    let subcommand = args.next().unwrap(); // should not error, as we checked len
    match subcommand.as_str() {
        "list" => process_installs_list(args),
        "new" => process_installs_new(args),
        x => {
            eprintln!("unknown subcommand '{}'", x);
            eprintln!("{}", INSTALL_HELP_MESSAGE);
            std::process::exit(2);
        }
    }
}

enum InstallArgsListParsingState {
    ExpectPathOrFlag,
    NotExpectingAnything,
}

enum InstallArgsNewParsingState {
    ExpectPathOrFlag,
    ExpectingBranch,
    NotExpectingAnything,
}

fn help_get_installs_from_dir(base_path: PathBuf) -> InstallationsData {
    if !base_path.exists() {
        eprintln!("given base_path {:?} does not exist", base_path);
        std::process::exit(1);
    }
    if !base_path.is_dir() {
        eprintln!("given base_path {:?} is not a directory", base_path);
        std::process::exit(1);
    }

    match InstallationsData::from_dir(base_path.clone()) {
        Ok(installs) => installs,
        Err(e) => {
            eprintln!("failed to scan given base_path {:?}: {}", base_path, e);
            std::process::exit(1);
        }
    }
}

fn process_installs_list(args: Args) -> Result<(), Error> {
    let mut state = InstallArgsListParsingState::ExpectPathOrFlag;
    let mut base_path = PathBuf::new();

    for arg in args {
        match (state, arg) {
            (InstallArgsListParsingState::ExpectPathOrFlag, arg) if arg.starts_with("--") => {
                eprintln!("not expecting flags");
                eprintln!("{}", INSTALL_HELP_MESSAGE);
                std::process::exit(2);
            }
            (InstallArgsListParsingState::ExpectPathOrFlag, arg) => {
                base_path = PathBuf::from(arg);
                state = InstallArgsListParsingState::NotExpectingAnything;
            }
            (InstallArgsListParsingState::NotExpectingAnything, _) => {
                eprintln!("not expecting any more arguments after base_path");
                eprintln!("{}", INSTALL_HELP_MESSAGE);
                std::process::exit(2);
            }
        }
    }

    let installs = help_get_installs_from_dir(base_path.clone());

    list_installs(&installs);

    Ok(())
}

fn process_installs_new(args: Args) -> Result<(), Error> {
    let mut state = InstallArgsNewParsingState::ExpectPathOrFlag;
    let mut branch = "dev".to_owned();
    let mut base_path = PathBuf::from(".");
    let mut do_viewer = true;

    for arg in args {
        match (state, arg) {
            (InstallArgsNewParsingState::ExpectPathOrFlag, arg) if arg == "--branch" => {
                state = InstallArgsNewParsingState::ExpectingBranch
            }
            (InstallArgsNewParsingState::ExpectPathOrFlag, arg) if arg == "--no-viewer" => {
                do_viewer = false;
                state = InstallArgsNewParsingState::ExpectPathOrFlag;
            }
            (InstallArgsNewParsingState::ExpectPathOrFlag, arg) => {
                base_path = PathBuf::from(arg);
                state = InstallArgsNewParsingState::NotExpectingAnything;
            }
            (InstallArgsNewParsingState::ExpectingBranch, arg) => {
                branch = arg;
                state = InstallArgsNewParsingState::ExpectPathOrFlag;
            }
            (InstallArgsNewParsingState::NotExpectingAnything, _) => {
                eprintln!("not expecting any more arguments after base_path");
                eprintln!("{}", INSTALL_HELP_MESSAGE);
                std::process::exit(2);
            }
        }
    }

    let mut installs = help_get_installs_from_dir(base_path);

    match installs.download_new_version(&branch, do_viewer) {
        Ok(_) => {
            println!("New version downloaded and set current");
        }
        Err(e) => {
            eprintln!("Failed to get latest version: {}", e);
        }
    }

    list_installs(&installs);

    Ok(())
}

fn list_installs(installs: &InstallationsData) {
    println!("valid base path: {:?}", installs.base_path());
    if installs.is_base_path_tainted() {
        println!("    Warning: given path contains elements unrelated to lifeblood.");
        println!("    It's recommended to choose an empty directory for lifeblood installations");
    }
    println!("");
    for (i, ver) in installs.iter_versions().enumerate().rev() {
        println!(
            "{} | {} | {} | {}",
            if installs.current_version_index() == i {
                "current"
            } else {
                "       "
            },
            ver.nice_name(),
            ver.date().format("%d-%m-%Y %H:%M:%S").to_string(),
            ver.source_commit(),
        );
    }
}
