mod filesystem;
mod log;

use std::env;
use std::path::PathBuf;
use std::process;

use clap::Arg;
use clap::ArgAction;
use clap::Command;
use clap::value_parser;
use filesystem::mount::mount;
use filesystem::unmount::unmount;
use tracing::error;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

fn main() {
    let mut command = build_command();

    if env::args().len() == 1 {
        command.print_help().expect("failed to print help");
        process::exit(0);
    }

    let matches = command.get_matches();

    let verbose = matches.get_flag("verbose");
    let log_level = if verbose { "info" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(log_level))
        .event_format(fmt::format().without_time().compact())
        .init();

    if let Some(("log", _)) = matches.subcommand() {
        if let Err(err) = log::view() {
            error!("Failed to view log: {}", err);
            process::exit(1);
        }
        return;
    }

    match (
        matches.get_one::<PathBuf>("unmount"),
        matches.get_one::<PathBuf>("root_dir"),
        matches.get_one::<PathBuf>("mount_point"),
    ) {
        (Some(unmount_dir), _, _) => {
            info!("Unmounting directory '{}'...", unmount_dir.display());
            if let Err(err) = unmount(unmount_dir) {
                error!("Failed to unmount: {}", err);
                process::exit(1);
            }
        }
        (None, Some(root_dir), Some(mount_point)) => {
            info!(
                "Mounting encrypted storage from '{}' to '{}'...",
                root_dir.display(),
                mount_point.display()
            );
            if let Err(err) = mount(root_dir, mount_point) {
                error!("Failed to mount: {}", err);
                process::exit(1);
            }
        }
        _ => {
            error!("Invalid combination of arguments. Use --help to see usage.");
            process::exit(1);
        }
    }
}

fn build_command() -> Command {
    Command::new("vylfs")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(Command::new("log").about("Print the log from the last run if it exists"))
        .arg(
            Arg::new("root_dir")
                .help("Set the root directory for the encrypted storage")
                .value_parser(value_parser!(PathBuf))
                .required(false)
                .requires("mount_point"),
        )
        .arg(
            Arg::new("mount_point")
                .help("Set the mount point for the decrypted filesystem")
                .value_parser(value_parser!(PathBuf))
                .required(false)
                .requires("root_dir"),
        )
        .arg(
            Arg::new("unmount")
                .short('u')
                .long("unmount")
                .help("Unmount a specific mount point (requires root)")
                .value_parser(value_parser!(PathBuf))
                .required(false)
                .conflicts_with_all(["root_dir", "mount_point"]),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
}
