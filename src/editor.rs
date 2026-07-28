use anyhow::{Context, Result, bail};
use std::process::Command;
use std::{env, fs, path::PathBuf};
use tempfile::NamedTempFile;
use which::which;

pub fn open_in_editor(content: &str) -> Result<String> {
    let mut editor_path = PathBuf::new();

    if let Ok(edit) = env::var("VISUAL").or_else(|_| env::var("EDITOR")) {
        if let Ok(path) = which(edit) {
            editor_path = path;
        }
    }

    if !editor_path.exists() {
        let fallback = ["nvim", "helix", "code", "micro", "nano", "vim", "vi"];
        for cmd in fallback {
            if let Ok(path) = which(cmd) {
                editor_path = path;
                break;
            }
        }
    }

    if !editor_path.exists() {
        bail!(
            "No text editor found.\nHint: Set the EDITOR environment variable (e.g. export EDITOR=nano)."
        );
    }

    let tmpfile = NamedTempFile::with_suffix(".toml")
        .context("Failed to create temporary file for editing")?;
    let filepath = tmpfile.path();

    fs::write(filepath, content).with_context(|| {
        format!(
            "Failed to write config content to temporary file {:?}",
            filepath
        )
    })?;

    let status = Command::new(&editor_path)
        .arg(filepath)
        .status()
        .with_context(|| format!("Failed to launch text editor at {:?}", editor_path))?;

    if !status.success() {
        bail!("Editor process exited with an error.");
    }

    let edited = fs::read_to_string(filepath)
        .context("Failed to read back edited content from temporary file")?;

    Ok(edited)
}
