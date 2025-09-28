use directories::UserDirs;
use log::info;
use std::fs::{self, OpenOptions};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Command;

/*
config generation logic:
ignore everything then add a new block at the end, starts with a marker comment like so

#tilb generated
Host github.com-username
  HostAlias github.com
  User git
  IdentityFile ~/.ssh/id_username_ed25519

i prob should support websites other than github?
*/

// TODO:
// uses include in ssh config rather than modifying main config

fn get_ssh_config_path() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| user_dirs.home_dir().join(".ssh").join("config"))
}

pub fn get_ssh_dir_path() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| user_dirs.home_dir().join(".ssh").join("tilb"))
}

fn ssh_config_exists() -> bool {
    get_ssh_config_path().map_or(false, |p| p.exists())
}

fn ssh_dir_exists() -> bool {
    get_ssh_dir_path().map_or(false, |p| p.exists())
}

fn create_ssh_config() -> Result<(), Error> {
    let config_path =
        get_ssh_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?; // ensure ~/.ssh exists
        }
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&config_path)?;
    }

    Ok(())
}

fn create_ssh_dir() -> Result<(), Error> {
    let dir_path = get_ssh_dir_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)?;
    }

    Ok(())
}

pub fn generate_ssh_key(user: &str, passphrase: &str) -> Result<String, Error> {
    if !ssh_dir_exists() {
        create_ssh_dir()?;
    }

    let ssh_dir = get_ssh_dir_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    let private_key_path = ssh_dir.join(format!("id_{}_ed25519", user));
    let public_key_path = private_key_path.with_extension("pub");

    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-C")
        .arg(user)
        .arg("-f")
        .arg(&private_key_path)
        .arg("-N")
        .arg(passphrase)
        .status()?;

    if !status.success() {
        return Err(Error::new(ErrorKind::Other, "SSH key generation failed"));
    }

    let public_key_content = fs::read_to_string(&public_key_path)?;
    Ok(public_key_content.trim().to_string())
}

pub fn add_to_ssh_config(
    host_alias: &str,
    host_name: &str,
    user: &str,
    identity_file: &str,
) -> Result<(), std::io::Error> {
    if !ssh_config_exists() {
        create_ssh_config()?;
    }

    let path = get_ssh_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    let mut lines: Vec<String> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.to_string())
        .collect();

    let block_marker = "#tilb generated";
    let block_header = format!("Host {}", host_alias);

    // new block we want to ensure
    let new_block = vec![
        block_marker.to_string(),
        block_header.clone(),
        format!("  HostName {}", host_name),
        format!("  User {}", user),
        format!("  IdentityFile {}", identity_file),
    ];

    // search for an existing generated block with same host
    let mut start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == block_marker {
            if let Some(next) = lines.get(i + 1) {
                if next.trim() == block_header {
                    start = Some(i);
                    break;
                }
            }
        }
    }

    if let Some(idx) = start {
        // replace existing block
        // find where this block ends (next comment or end of file)
        let mut end = idx + 2; // skipping first host line
        while end < lines.len() {
            if lines[end].trim() == block_marker {
                break; // another generated block
            }
            // stop when encountering a non-indented host (user-defined)
            if lines[end].starts_with("Host ") {
                break;
            }
            end += 1;
        }
        lines.splice(idx..end, new_block);
    } else {
        // append new block at the end
        if !lines.is_empty() {
            lines.push("".into()); // blank line before appending
        }
        lines.extend(new_block);
    }

    fs::write(path, lines.join("\n") + "\n")?;
    return Ok(());
}

pub fn remove_from_ssh_config(host_alias: &str) -> Result<(), std::io::Error> {
    if !ssh_config_exists() {
        eprintln!("SSH config not found, nothing to remove.");
        return Ok(());
    }

    let path = get_ssh_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    let mut lines: Vec<String> = fs::read_to_string(&path)?
        .lines()
        .map(|l| l.to_string())
        .collect();

    let block_marker = "#tilb generated";
    let block_header = format!("Host {}", host_alias);

    let mut start = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == block_marker {
            if let Some(next) = lines.get(i + 1) {
                if next.trim() == block_header {
                    start = Some(i);
                    break;
                }
            }
        }
    }

    if let Some(idx) = start {
        // replace existing block
        // find where this block ends (next comment or end of file)
        let mut end = idx + 2; // skipping first host line
        while end < lines.len() {
            if lines[end].trim() == block_marker {
                break; // another generated block
            }
            // stop when encountering a non-indented host (user-defined)
            if lines[end].starts_with("Host ") {
                break;
            }
            end += 1;
        }
        lines.splice(idx..end, std::iter::empty());

        fs::write(path, lines.join("\n").trim_end().to_string() + "\n")?;
        info!("Removed generated ssh block for '{}'.", host_alias);
    } else {
        eprintln!(
            "No generated block found for host alias '{}', nothing to remove.",
            host_alias
        );
    }

    return Ok(());
}
