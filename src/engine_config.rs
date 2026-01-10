use serde::Deserialize;
use anyhow::{Result, bail};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub weights_and_corr_version: String,
    pub calculation_currency: String,
    pub exchange_rate: f64,
}

impl EngineConfig {
    /// Load from TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        let cfg: EngineConfig = toml::from_str(&text)?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Optional but recommended: semantic validation
    fn validate(&self) -> Result<()> {
        if self.weights_and_corr_version.is_empty() {
            bail!("weights_and_corr_version must not be empty");
        }

        if self.calculation_currency.len() != 3 {
            bail!("calculation_currency must be ISO-4217 (e.g. USD, EUR)");
        }

        if self.exchange_rate <= 0.0 {
            bail!("exchange_rate must be > 0");
        }

        Ok(())
    }
}
