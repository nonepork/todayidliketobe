use crate::args::{NewUserArgs, UserArgs};
use crate::config::{User, add_user, config_exists, create_config, delete_user, read_config_file};
use crate::git::{
    check_cwd_is_repo, get_repo_name, get_repo_name_from_user, parse_origin_url, set_git_remote,
};
use crate::ssh::{add_to_ssh_config, generate_ssh_key, get_ssh_dir_path, remove_from_ssh_config};
use inquire::validator::Validation;
use inquire::{Confirm, Password, PasswordDisplayMode};
use log::info;
use owo_colors::OwoColorize;
use regex::Regex;
use std::collections::HashSet;
use std::process::Command;

// TODO:
// make return properly handled
// print message better
// handle ctrlc

// WARN:
// expect untested

fn is_reasonable_email(email: &str) -> bool {
    // intentional basic check only
    let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
    re.is_match(email)
}

fn parse_domain_name(url: &str) -> Option<String> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .split('/')
        .next()
        .map(|s| s.to_string())
}

fn is_git_hosting_site(url: &str) -> bool {
    let git_domains: HashSet<&str> = [
        "github.com",
        "gitlab.com",
        "bitbucket.org",
        "gitea.io",
        "codeberg.org",
        "sr.ht",
    ]
    .into();

    parse_domain_name(url)
        .map(|domain| git_domains.contains(domain.as_str()))
        .unwrap_or(false)
}

pub fn handle_user_add(user_args: NewUserArgs) {
    let user = user_args.user;
    let email = user_args.email;
    let website = user_args.website;
    let use_https = user_args.use_https;

    // TODO: check if user already exists
    println!("Adding user: {}", user.green());

    if !is_reasonable_email(&email) {
        eprintln!(
            "Erm, '{}' doesn't look like a valid email address",
            email.bright_red()
        );
        let ans = Confirm::new("Continue anyway?")
            .with_default(false)
            .prompt();

        match ans {
            Ok(true) => {}
            Ok(false) => {
                println!("see ya (¯꒳¯)ᐝ");
                return;
            }
            Err(_) => {
                println!("see ya (¯꒳¯)ᐝ");
                return;
            }
        }
    }

    if !is_git_hosting_site(&website) {
        eprintln!(
            "{} doesn't look like a git hosting site",
            email.bright_red()
        );
        let ans = Confirm::new("Continue anyway?")
            .with_default(false)
            .prompt();

        match ans {
            Ok(true) => {}
            Ok(false) => {
                println!("see ya (¯꒳¯)ᐝ");
                return;
            }
            Err(_) => {
                println!("see ya (¯꒳¯)ᐝ");
                return;
            }
        }
    }

    let domain_name = parse_domain_name(&website).unwrap_or_else(|| {
        eprintln!("Failed to parse domain name from URL: {}", website);
        std::process::exit(1);
    });

    info!("{:?}", &domain_name);

    if !config_exists() {
        match create_config() {
            Ok(()) => {
                info!("Created new config file.");
            }
            Err(err) => {
                panic!("Error creating config: {}", err);
            }
        }
    }

    if !use_https {
        let validator = |input: &str| {
            if input.contains(' ') {
                Ok(Validation::Invalid(
                    "Passphrase cannot contain spaces".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        };

        let passphrase = Password::new("Enter passphrase (leave empty for no passphrase):")
            .with_display_mode(PasswordDisplayMode::Masked)
            .with_validator(validator)
            .prompt()
            .expect("failed to read passphrase");

        let ssh_file = format!("id_{}_ed25519", &user);
        let ssh_path = get_ssh_dir_path()
            .expect("no home dir")
            .join(ssh_file)
            .display()
            .to_string();

        let pub_content = generate_ssh_key(&user, &passphrase).expect("failed to generate ssh key");

        println!(
            "Public key (make sure to add to {}):\n{}",
            &domain_name, &pub_content
        );

        // username is being used as host alias in ssh config
        // check ssh for format
        let host_alias = format!("tilb-{}", &user);

        add_to_ssh_config(&host_alias, &domain_name, &user, &ssh_path)
            .expect("failed to update ssh config");
    }

    let new_user = User {
        name: user.clone(),
        email: email.clone(),
        git_host: domain_name.clone(),
        use_https: use_https,
    };

    if let Err(err) = add_user(new_user) {
        eprintln!("Error updating config: {}", err);
        return;
    }

    println!("User: {} <{}> added", user.green(), email.green());
}

pub fn handle_user_remove(user: UserArgs) {
    let user = user.user;

    if !config_exists() {
        println!(
            "Config not found, add a new user via `{}`!",
            "tilb add".blue()
        );
        return;
    }

    let config = match read_config_file() {
        Ok(config) => config,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Config not found, add a new user via `{}`!",
                "tilb add".blue()
            );
            return;
        }
        Err(err) => {
            eprintln!("Error reading config: {}", err);
            return;
        }
    };

    if config.users.get(&user).is_none() {
        eprintln!("User '{}' not found in config.", user);
        return;
    }

    println!("NOTE: The ssh key for {} will not be delete", user.green());

    let ans = Confirm::new(&format!("Are you sure you want to remove {}", &user)) // i'm sorry?
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => {}
        Ok(false) => {
            println!("see ya (¯꒳¯)ᐝ");
            return;
        }
        Err(_) => {
            println!("see ya (¯꒳¯)ᐝ");
            return;
        }
    }

    if let Err(err) = delete_user(&user) {
        eprintln!("Error deleting user: {}", err);
        return;
    }

    let host_alias = format!("github-{}", &user);
    remove_from_ssh_config(&host_alias).expect("failed to update ssh config");

    println!("User: {} removed", user.green());
}

pub fn handle_user_switch(user: UserArgs) {
    let user = user.user;

    if !check_cwd_is_repo() {
        eprintln!("Current directory is not a git repository.");
        return;
    }

    let config = match read_config_file() {
        Ok(config) => config,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Config not found, add a new user via `{}`!",
                "tilb add".blue()
            );
            return;
        }
        Err(err) => {
            eprintln!("Error reading config: {}", err);
            return;
        }
    };

    let selected_user = if let Some(user_fetehed) = config.users.get(&user) {
        user_fetehed
    } else {
        eprintln!("User '{}' not found in config.", user);
        return;
    };

    let (repo_owner, repo_name) = match get_repo_name() {
        Some(origin_url) => parse_origin_url(&origin_url).unwrap_or_else(|| {
            eprintln!("Couldn't parse origin URL, falling back to cached owner");
            let repo_name = get_repo_name_from_user();
            (selected_user.name.clone(), repo_name)
        }),
        None => {
            let repo_name = get_repo_name_from_user();
            (selected_user.name.clone(), repo_name)
        }
    };

    Command::new("git")
        .arg("config")
        .arg("--local")
        .arg("user.name")
        .arg(&selected_user.name)
        .status()
        .expect("Failed to set git user.name");

    Command::new("git")
        .arg("config")
        .arg("--local")
        .arg("user.email")
        .arg(&selected_user.email)
        .status()
        .expect("Failed to set git user.email");

    if selected_user.use_https {
        let full_origin = format!(
            "https://{}/{}/{}",
            selected_user.git_host, repo_owner, repo_name
        );

        match set_git_remote(&full_origin) {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Error setting git remote: {}", err);
                return;
            }
        }
    } else {
        let full_origin = format!(
            "git@{}-{}:{}/{}",
            selected_user.git_host, selected_user.name, repo_owner, repo_name
        );

        match set_git_remote(&full_origin) {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Error setting git remote: {}", err);
                return;
            }
        }
    }

    println!("Switched to user: {}", selected_user.name.green());
}

pub fn handle_user_list() {
    match read_config_file() {
        Ok(config) => {
            println!("Users:");
            if config.users.is_empty() {
                println!(
                    "(no users found, add a new user via `{}`)",
                    "tilb add".blue()
                );
                return;
            }
            for (_, user) in config.users {
                println!("- {} <{}>", user.name.green(), user.email);
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "Config not found, add a new user via `{}`!",
                "tilb add".blue()
            );
        }
        Err(err) => {
            eprintln!("Error reading config: {}", err);
        }
    }
}
