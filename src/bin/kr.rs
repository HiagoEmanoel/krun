use anyhow::{Context, Result};
use clap::Parser;
use lib::cli::KrCli;
use lib::models::Environment;
use lib::*;

fn main() -> Result<()> {
    let cli = KrCli::parse();
    let env = Environment::new(cli.common)?;

    let ws = env.get_workspace()?.with_context(
        || "Workspace not initialized for this directory.\nHint: Run 'qrun --init' to get started.",
    )?;

    let target_profile = cli.profile.as_deref().unwrap_or(&ws.default_profile);

    let profile_data = ws.get_profile(target_profile).with_context(|| {
        format!(
            "Profile '{}' not found in workspace '{}'.\nHint: Run 'qrun --edit' to add this profile.",
            target_profile, ws.name
        )
    })?;

    exec::run_script(&ws.shell, &profile_data.content)?;

    Ok(())
}
