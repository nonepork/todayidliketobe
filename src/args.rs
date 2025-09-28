use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "A cli that handles multiple github accounts",
    subcommand_help_heading = "Actions"
)]
pub struct TilbArgs {
    #[clap(subcommand, name = "action")]
    pub action: Actions,
}

#[derive(Debug, Subcommand)]
#[command(about, rename_all = "kebab-case")]
pub enum Actions {
    /// List all users
    List,
    /// Add a new user
    Add(NewUserArgs),
    /// Remove an existing user
    Remove(UserArgs),
    /// Switch to a different user
    Switch(UserArgs),
}

#[derive(Debug, Args)]
pub struct NewUserArgs {
    /// The username to operate on
    pub user: String,
    /// The email shown in commits
    pub email: String,
    /// The website of the git host, e.g. github, gitlab, bitbucket
    #[arg(long, short, default_value_t = String::from("github"))]
    pub website: String,
}

#[derive(Debug, Args)]
pub struct UserArgs {
    /// The username to operate on
    pub user: String,
}
