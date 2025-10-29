use log::{debug, info};
use std::process::Command;

use inquire::{Text, validator::Validation};

pub fn check_cwd_is_repo() -> bool {
    match Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn get_repo_name() -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_repo_name_from_user() -> String {
    let validator = |input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("Repo name cannot be empty".into()))
        } else if input.contains(' ') {
            Ok(Validation::Invalid(
                "Repo name cannot contain spaces".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };

    match Text::new("Enter repo name (e.g., 'my-repo')")
        .with_validator(validator)
        .prompt()
    {
        Ok(name) => name,
        Err(err) => {
            panic!("Error with your repo name input: {}", err);
        }
    }
}

pub fn parse_origin_url(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches(".git");

    // ssh style: git@github.com:owner/repo
    if url.contains(':') && url.contains('@') {
        let after_colon = url.split_once(':')?.1;
        let mut parts = after_colon.split('/');
        debug!("{:?}", parts);
        let owner = parts.next()?.to_string();
        let repo = parts.next()?.to_string();
        return Some((owner, repo));
    }

    // https style: https://github.com/owner/repo
    if url.contains("://") {
        let parts: Vec<&str> = url.split('/').collect();
        debug!("{:?}", parts);
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2].to_string();
            let repo = parts[parts.len() - 1].to_string();
            return Some((owner, repo));
        }
    }

    None
}

pub fn set_git_remote(full_origin: &str) -> Result<(), Box<dyn std::error::Error>> {
    let add_result = Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg(full_origin)
        .output()?;

    if add_result.status.success() {
        info!("Successfully added remote origin");
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&add_result.stderr);
    if stderr.contains("already exists") {
        let set_result = Command::new("git")
            .arg("remote")
            .arg("set-url")
            .arg("origin")
            .arg(full_origin)
            .status()?;

        if set_result.success() {
            info!("Successfully updated remote origin");
            Ok(())
        } else {
            Err("Failed to set remote URL".into())
        }
    } else {
        Err(format!("Git remote add failed: {}", stderr).into())
    }
}
