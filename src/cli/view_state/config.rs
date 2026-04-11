#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;

use super::{Config, ViewState, CONFIG_FILE_NAME};
use anyhow::bail;
use std::{
    fs::File,
    io::{Read, Write},
};

impl ViewState {
    pub(crate) fn accept_license_terms(&mut self) {
        self.accepted_license_terms = true;
        // TODO: Push any error into some sort of error stream and expose in UI.
        let _ = self.write_config_file();
    }

    pub(crate) fn read_config_file(&mut self) -> anyhow::Result<()> {
        self.ensure_config_file_path()?;

        let file_path = self
            .config_file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The default config file path was not set!"))?;

        let mut file = File::open(file_path)?;
        let mut yaml = String::new();
        file.read_to_string(&mut yaml)?;

        let config: Config = serde_yaml::from_str(&yaml)?;

        self.accepted_license_terms = config.accepted_license_terms;

        Ok(())
    }

    pub(super) fn write_config_file(&mut self) -> anyhow::Result<()> {
        self.ensure_config_file_path()?;

        let file_path = self
            .config_file_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The default config file path was not set!"))?;

        let config = Config {
            accepted_license_terms: self.accepted_license_terms,
        };

        let yaml = serde_yaml::to_string(&config)?;

        let mut file = File::create(file_path)?;
        file.write_all(yaml.as_bytes())?;

        Ok(())
    }

    fn ensure_config_file_path(&mut self) -> anyhow::Result<()> {
        if self.config_file_path.is_some() {
            return Ok(());
        }

        if let Some(user_home_dir) = dirs::home_dir() {
            self.config_file_path = Some(user_home_dir.join(".space").join(CONFIG_FILE_NAME));
            Ok(())
        } else {
            bail!("Could not determine the home directory in order to write the config file.");
        }
    }
}
