use anyhow::{Context, Result, bail};
use git2::{Oid, Repository};
use hex::ToHex;
use sha2::{Digest, Sha256};

pub fn resolve_id(path: &str) -> Result<String> {
    if let Ok(repo) = Repository::discover(path) {
        let oid = first_commit(repo)?;
        Ok(oid.to_string())
    } else {
        let hash = Sha256::digest(path.as_bytes()).encode_hex();
        Ok(hash)
    }
}

fn first_commit(repo: Repository) -> Result<Oid> {
    if repo
        .is_empty()
        .context("Failed to check if repository is empty")?
    {
        bail!("Git repository is empty.\nHint: Make an initial commit first.");
    }

    if repo.is_bare() {
        bail!("Bare git repositories are not supported.");
    }

    let mut rwalk = repo.revwalk().context("Failed to initialize git revwalk")?;
    rwalk
        .push_head()
        .context("Failed to find git HEAD reference")?;

    let first = rwalk
        .last()
        .context("No commits found in repository history")?
        .context("Error walking commit history")?;

    Ok(first)
}
