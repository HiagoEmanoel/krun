use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::io::{self, Write};

#[derive(Args, Debug, Clone)]
#[group(required = false, multiple = false)]
pub struct CommonArgs {
    /// Set target directory
    #[arg(short, long)]
    pub dir: Option<String>,

    /// Set target workspace by name
    #[arg(short, long)]
    pub workspace: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Quick profile runner for directories and git repositories"
)]
pub struct KrunCli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Quick profile runner (alias for krun run)")]
pub struct KrCli {
    /// Profile to execute
    pub profile: Option<String>,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Open configuration editor
    Edit,

    /// Register a new directory or repository
    Init,

    /// Delete target workspace configuration
    Remove,

    /// Execute the given <profile> or the default profile
    Run { profile: Option<String> },
}

pub fn confirm(msg: &str) -> Result<bool> {
    print!("{} [Y/n]: ", msg);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(true);
    }

    Ok(trimmed.eq_ignore_ascii_case("y"))
}
