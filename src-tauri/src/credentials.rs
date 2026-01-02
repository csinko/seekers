use crate::constants;
use crate::Credentials;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home)
        .join(".config")
        .join(constants::CONFIG_DIR_NAME);
    fs::create_dir_all(&config_dir).ok();
    config_dir.join(constants::CREDENTIALS_FILE)
}

pub struct CredentialsManager;

impl CredentialsManager {
    pub fn new() -> Self {
        Self
    }

    pub fn save(&self, org_id: &str, session_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = get_config_path();
        let creds = Credentials {
            org_id: org_id.to_string(),
            session_key: session_key.to_string(),
        };

        let json = serde_json::to_string_pretty(&creds)?;
        fs::write(&path, &json)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(constants::SECURE_FILE_MODE);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    pub fn load(&self) -> Result<Credentials, Box<dyn std::error::Error>> {
        let path = get_config_path();
        if !path.exists() {
            return Ok(Credentials {
                org_id: String::new(),
                session_key: String::new(),
            });
        }

        let json = fs::read_to_string(&path)?;
        let creds: Credentials = serde_json::from_str(&json)?;
        Ok(creds)
    }
}
