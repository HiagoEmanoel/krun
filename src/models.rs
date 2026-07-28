use crate::cli::CommonArgs;
use crate::db::Database;
use crate::identity;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub default_profile: String,
    pub profiles: Vec<Profile>,
}

pub struct Environment {
    pub dir: String,
    pub common: CommonArgs,
    pub db: Database,
}

impl Environment {
    pub fn new(common: CommonArgs) -> Result<Self> {
        let db = Database::open()?;

        let target_dir = common.dir.clone().unwrap_or_else(|| ".".to_string());
        let canonical_dir = fs::canonicalize(&target_dir)
            .with_context(|| format!("Directory path '{}' not found.", target_dir))?
            .to_string_lossy()
            .to_string();

        Ok(Environment {
            dir: canonical_dir,
            common,
            db,
        })
    }

    pub fn get_workspace(&self) -> Result<Option<Workspace>> {
        let workspace_id = match &self.common.workspace {
            Some(name) => {
                self.db.get_id_by_name(name)?.with_context(|| {
                    format!(
                        "Workspace with name '{}' not found.\nHint: Run 'krun --init' in the target directory first.",
                        name
                    )
                })?
            }
            None => identity::resolve_id(&self.dir)?,
        };

        Ok(self.db.get_workspace(workspace_id.as_str())?)
    }
}

impl Workspace {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            shell: "bash".to_string(),
            default_profile: "test".to_string(),
            profiles: vec![Profile {
                name: "test".to_string(),
                content: "make".to_string(),
            }],
        }
    }

    pub fn to_text(&self) -> String {
        let mut out = format!(
            "[config]\nname = \"{}\"\nshell = \"{}\"\ndefault_profile = \"{}\"\n\n",
            self.name, self.shell, self.default_profile
        );

        for profile in &self.profiles {
            out.push_str(&format!("[{}]\n{}\n\n", profile.name, profile.content));
        }

        out = out.trim_end().to_string();
        out.push('\n');
        out
    }

    pub fn from_text(id: &str, raw: &str) -> Option<Self> {
        let mut sections: Vec<(String, String)> = Vec::new();
        let mut current_name = String::new();
        let mut current_lines = Vec::new();

        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if !current_name.is_empty() {
                    sections.push((current_name.clone(), current_lines.join("\n")));
                    current_lines.clear();
                }
                current_name = trimmed[1..trimmed.len() - 1].to_string();
            } else if !current_name.is_empty() && !trimmed.is_empty() {
                current_lines.push(line.to_string());
            }
        }

        if !current_name.is_empty() {
            sections.push((current_name, current_lines.join("\n")));
        }

        let config_idx = sections.iter().position(|(name, _)| name == "config")?;
        let (_, config_content) = sections.remove(config_idx);

        let mut config_map = HashMap::new();
        for line in config_content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                config_map.insert(k.trim().to_lowercase(), val);
            }
        }

        let name = config_map
            .remove("name")
            .unwrap_or_else(|| "unknown".to_string());
        let shell = config_map
            .remove("shell")
            .unwrap_or_else(|| "bash".to_string());
        let default_profile = config_map
            .remove("default_profile")
            .unwrap_or_else(|| "test".to_string());

        let profiles = sections
            .into_iter()
            .map(|(p_name, content)| Profile {
                name: p_name,
                content,
            })
            .collect();

        Some(Workspace {
            id: id.to_string(),
            name,
            shell,
            default_profile,
            profiles,
        })
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }
}
