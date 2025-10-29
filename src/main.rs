mod args;
mod config;
mod git;
mod ops;
mod ssh;

use args::TilbArgs;
use clap::Parser;
use log::info;
use ops::{handle_user_add, handle_user_list, handle_user_remove, handle_user_switch};

fn check_git_installed() -> bool {
    match std::process::Command::new("git").arg("--version").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn main() {
    env_logger::init();
    let args = TilbArgs::parse();

    if check_git_installed() {
        info!("Git is installed.");
    } else {
        eprintln!("Git is not installed. Please install Git to proceed.");
        return;
    }

    match args.action {
        args::Actions::List => handle_user_list(),
        args::Actions::Add(new_user_args) => handle_user_add(new_user_args),
        args::Actions::Remove(user_args) => handle_user_remove(user_args),
        args::Actions::Switch(user_args) => handle_user_switch(user_args),
    };
}
