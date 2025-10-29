use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

use directories::UserDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UsersConfig {
    pub users: HashMap<String, User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub git_host: String,
    pub use_https: bool,
}

fn get_config_path() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| user_dirs.home_dir().join(".tilb").join("config.toml"))
}

pub fn config_exists() -> bool {
    get_config_path().map_or(false, |p| p.exists())
}

pub fn create_config() -> Result<()> {
    let path = get_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?; // ensure ~/.tilb exists
        }
        OpenOptions::new().create(true).write(true).open(&path)?;
        println!("Created config file at: {:?}", path);
    }

    Ok(())
}

pub fn read_config_file() -> Result<UsersConfig> {
    let path = get_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Err(err);
        }
        Err(err) => {
            return Err(err);
        }
    };

    let config: UsersConfig =
        toml::from_str(&content).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    Ok(config)
}

pub fn add_user(new_user: User) -> Result<()> {
    let mut config = read_config_file().unwrap_or(UsersConfig {
        users: HashMap::new(),
    });

    config.users.insert(new_user.name.clone(), new_user);

    let toml_str = toml::to_string_pretty(&config).map_err(|e| Error::new(ErrorKind::Other, e))?;

    let path = get_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;
    fs::write(path, toml_str)?;
    Ok(())
}

pub fn delete_user(alias: &str) -> Result<()> {
    let mut config = read_config_file().unwrap_or(UsersConfig {
        users: HashMap::new(),
    });

    config.users.remove(alias);

    let toml_str = toml::to_string_pretty(&config).map_err(|e| Error::new(ErrorKind::Other, e))?;

    let path = get_config_path().ok_or_else(|| Error::new(ErrorKind::Other, "no home dir"))?;
    fs::write(path, toml_str)?;
    Ok(())
}
