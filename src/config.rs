use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub pairs: Vec<Pair>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pair {
    /// Optional label shown in logs
    pub name: Option<String>,
    /// Bind XSUB here (publishers connect)
    pub frontend: String,
    /// Bind XPUB here (subscribers connect)
    pub backend: String,
    /// XSUB receive high-water mark
    #[serde(default = "default_hwm")]
    pub xsub_rcvhwm: i32,
    /// XPUB send high-water mark
    #[serde(default = "default_hwm")]
    pub xpub_sndhwm: i32,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration load error: {0}")]
    Load(#[from] config::ConfigError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

pub fn default_hwm() -> i32 {
    100_000
}

pub fn load_settings() -> Result<Settings, ConfigError> {
    let cwd = std::env::current_dir()?;
    load_settings_from(&cwd)
}

pub fn load_settings_from<P: AsRef<Path>>(dir: P) -> Result<Settings, ConfigError> {
    let dir = dir.as_ref();
    let cfg = config::Config::builder()
        .add_source(config::File::from(dir.join("proxy.yaml")).required(false))
        .add_source(config::File::from(dir.join("proxy.toml")).required(false))
        .add_source(config::File::from(dir.join("proxy.json")).required(false))
        .build()?;

    let settings: Settings = cfg.try_deserialize()?;
    if settings.pairs.is_empty() {
        return Err(ConfigError::Invalid("no pairs defined".into()));
    }
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hwm_is_100k() {
        assert_eq!(default_hwm(), 100_000);
    }

    #[test]
    fn load_settings_from_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let yaml = r#"
            pairs:
              - name: default
                frontend: "tcp://*:5557"
                backend: "tcp://*:5558"
        "#;
        std::fs::write(dir.path().join("proxy.yaml"), yaml).unwrap();

        let settings = load_settings_from(dir.path()).unwrap();
        assert_eq!(settings.pairs.len(), 1);
        let pair = &settings.pairs[0];
        assert_eq!(pair.xsub_rcvhwm, default_hwm());
        assert_eq!(pair.xpub_sndhwm, default_hwm());
    }

    #[test]
    fn error_on_empty_pairs() {
        let dir = tempfile::tempdir().unwrap();
        let yaml = "pairs: []";
        std::fs::write(dir.path().join("proxy.yaml"), yaml).unwrap();

        let err = load_settings_from(dir.path()).unwrap_err();
        match err {
            ConfigError::Invalid(msg) => assert_eq!(msg, "no pairs defined"),
            _ => panic!("unexpected error: {err}"),
        }
    }
}
