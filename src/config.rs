use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub device_id: String,
}

impl Config {
    fn path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("no config dir")?
            .join("jellyfin-cli");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            let cfg = Config {
                server_url: std::env::var("JELLYFIN_SERVER")
                    .unwrap_or_else(|_| "http://d.local:8096".into()),
                access_token: std::env::var("JELLYFIN_TOKEN").ok(),
                user_id: None,
                user_name: None,
                device_id: uuid::Uuid::new_v4().to_string(),
            };
            cfg.save()?;
            Ok(cfg)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}
