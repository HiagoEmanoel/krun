use anyhow::{Context, Result};
use clap::Parser;
use lib::cli::{Commands, KrunCli};
use lib::models::{Environment, Workspace};
use lib::*;

fn main() -> Result<()> {
    let cli = KrunCli::parse();
    let env = Environment::new(cli.common)?;

    match &cli.command {
        Commands::Init => {
            if env.get_workspace()?.is_some() {
                println!(
                    "Workspace is already initialized.\nHint: Use 'krun --edit' to change configuration."
                );
                return Ok(());
            }

            let dir_name = env
                .dir
                .split('/')
                .filter(|s| !s.is_empty())
                .last()
                .unwrap_or("unknown");

            let workspace_id = identity::resolve_id(&env.dir)?;
            let initial_ws = Workspace::new(&workspace_id, dir_name);
            let raw_text = editor::open_in_editor(&initial_ws.to_text())?;

            if let Some(parsed_ws) = Workspace::from_text(&workspace_id, &raw_text) {
                env.db.save_workspace(&parsed_ws)?;
                println!("Workspace '{}' initialized successfully.", parsed_ws.name);
            } else {
                println!("Initialization aborted: Invalid configuration text format.");
            }
            return Ok(());
        }

        Commands::Edit => {
            let ws = env.get_workspace()?.with_context(
                || "Workspace not found.\nHint: Run 'qrun --init' to set up this directory.",
            )?;

            let edited_text = editor::open_in_editor(&ws.to_text())?;

            if let Some(parsed_ws) = Workspace::from_text(&ws.id, &edited_text) {
                env.db.save_workspace(&parsed_ws)?;
                println!("Workspace '{}' updated successfully.", parsed_ws.name);
            } else {
                println!("Update aborted: Invalid configuration text format.");
            }
            return Ok(());
        }

        Commands::Remove => {
            let ws = env
                .get_workspace()?
                .with_context(|| "Workspace not found.\nHint: Run 'qrun --init' first.")?;

            if cli::confirm(&format!(
                "Are you sure you want to delete workspace '{}'?",
                ws.name
            ))? {
                env.db.delete_workspace(&ws.id)?;
                println!("Workspace deleted successfully.");
            }
            return Ok(());
        }

        Commands::Run { profile } => {
            let ws = env.get_workspace()?.with_context(|| {
                    "Workspace not initialized for this directory.\nHint: Run 'qrun --init' to get started."
                })?;

            let target_profile = profile.as_deref().unwrap_or(&ws.default_profile);

            let profile_data = ws.get_profile(target_profile).with_context(|| {
                    format!(
                        "Profile '{}' not found in workspace '{}'.\nHint: Run 'qrun --edit' to add this profile.",
                        target_profile, ws.name
                    )
                })?;

            exec::run_script(&ws.shell, &profile_data.content)?;
        }
    }

    Ok(())
}
