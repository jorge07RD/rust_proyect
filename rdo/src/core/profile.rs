use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub xhost: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    #[serde(default)]
    pub container_id: Option<String>,
    pub dir_backup: Option<String>,
}

impl Profile {
    pub fn new(
        xhost: String,
        port: u16,
        username: String,
        password: Option<String>,
        container_id: Option<String>,
        dir_backup: Option<String>,
    ) -> Self {
        Self {
            xhost,
            port,
            username,
            password,
            container_id,
            dir_backup,
        }
    }

    /// Check if running in Docker mode (container_id is present)
    pub fn is_docker_mode(&self) -> bool {
        self.container_id.as_ref().map(|id| !id.is_empty()).unwrap_or(false)
    }
}

const PROFILE_FILE: &str = "profile.json";

pub fn load_profile() -> anyhow::Result<Profile> {
    if !std::path::Path::new(PROFILE_FILE).exists() {
        anyhow::bail!("No profile found. Create one first.");
    }

    let mut file = fs::File::open(PROFILE_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let profile: Profile = serde_json::from_str(&contents)?;
    Ok(profile)
}

pub fn save_profile(profile: &Profile) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(profile)?;
    let mut file = fs::File::create(PROFILE_FILE)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
