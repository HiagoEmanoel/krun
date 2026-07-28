use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn run_script(shell: &str, script: &str) -> Result<()> {
    let status = Command::new(shell)
        .arg("-c")
        .arg(script)
        .status()
        .with_context(|| format!("Failed to execute script using shell '{}'", shell))?;

    if !status.success() {
        bail!("Script execution failed with a non-zero exit code.");
    }

    Ok(())
}
