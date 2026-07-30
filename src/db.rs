use crate::models::{Profile, Workspace};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self> {
        let config_dir = dirs::config_dir().context(
            "Could not locate system config directory.\nHint: Check your OS environment variables.",
        )?;
        let krun_dir = config_dir.join("krun");
        let db_path = krun_dir.join("config.db");

        if !krun_dir.exists() {
            fs::create_dir_all(&krun_dir)
                .with_context(|| format!("Failed to create directory at {:?}", krun_dir))?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database file at {:?}", db_path))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS repos (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE,
                shell TEXT,
                default_profile TEXT
            );
            CREATE TABLE IF NOT EXISTS profiles (
                id TEXT,
                name TEXT,
                content TEXT
            );
            ",
        )
        .context("Failed to initialize database schema")?;

        Ok(Self { conn })
    }

    pub fn save_workspace(&self, ws: &Workspace) -> Result<()> {
        let name = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT name FROM repos WHERE name = ?1 AND id != ?2)",
                [&ws.name, &ws.id],
                |row| row.get::<_, bool>(0),
            )
            .map(|exists| match exists {
                true => {
                    println!("The workspace {} already exists, falling back", &ws.name);

                    self.conn
                        .query_one("SELECT name FROM repos WHERE id = ?1", [&ws.id], |row| {
                            Ok(row.get::<_, String>(0)?)
                        })
                }
                false => Ok(ws.name.clone()),
            })??;

        self.delete_workspace(&ws.id)?;

        self.conn
            .execute(
                "INSERT INTO repos (id, name, shell, default_profile) VALUES (?1, ?2, ?3, ?4)",
                [&ws.id, &name, &ws.shell, &ws.default_profile],
            )
            .with_context(|| format!("Failed to save workspace '{}'", name))?;

        let mut stmt = self
            .conn
            .prepare("INSERT INTO profiles (id, name, content) VALUES (?1, ?2, ?3)")?;

        for profile in &ws.profiles {
            stmt.execute([&ws.id, &profile.name, &profile.content])
                .with_context(|| format!("Failed to save profile '{}'", profile.name))?;
        }

        Ok(())
    }

    pub fn get_workspace(&self, id: &str) -> Result<Option<Workspace>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, shell, default_profile FROM repos WHERE id = ?1")?;

        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            let name: String = row.get(0)?;
            let shell: String = row.get(1)?;
            let default_profile: String = row.get(2)?;
            let profiles = self.get_profiles_for(id)?;

            Ok(Some(Workspace {
                id: id.to_string(),
                name,
                shell,
                default_profile,
                profiles,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_profiles_for(&self, id: &str) -> Result<Vec<Profile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, content FROM profiles WHERE id = ?1")?;

        let rows = stmt.query_map([id], |row| {
            Ok(Profile {
                name: row.get(0)?,
                content: row.get(1)?,
            })
        })?;

        let mut profiles = Vec::new();
        for profile in rows {
            profiles.push(profile?);
        }

        Ok(profiles)
    }

    pub fn delete_workspace(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM repos WHERE id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_id_by_name(&self, name: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT id FROM repos WHERE name = ?1")?;
        let mut rows = stmt.query([name])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}
