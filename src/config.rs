use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub aws: AwsConfig,
    pub app: AppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: Option<String>,
    pub workgroup: Option<String>,
    pub output_location: String,
    pub catalog: Option<String>,
    pub database: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum HistorySortBy {
    StartTime,
    EndTime,
    DataScanned,
    Status,
}

impl Default for HistorySortBy {
    fn default() -> Self {
        Self::StartTime
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(with = "humantime_serde")]
    pub query_reuse_time: Duration,
    pub max_rows: usize,
    /// Default number of history items to show
    #[serde(default = "default_history_size")]
    pub history_size: i32,
    /// Fields to display in history view
    #[serde(default)]
    pub history_fields: Option<Vec<String>>,
    /// Fields to display in inspect view
    #[serde(default)]
    pub inspect_fields: Option<Vec<String>>,
}

fn default_history_size() -> i32 {
    20
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aws: AwsConfig {
                region: Some("eu-west-1".to_string()),
                workgroup: Some("primary".to_string()),
                output_location: "s3://athena-query-results/".to_string(),
                catalog: Some("AwsDataCatalog".to_string()),
                database: None,
                profile: None,
            },
            app: AppConfig {
                query_reuse_time: Duration::from_secs(3600), // 1 hour
                max_rows: 1000,
                history_size: 20,
                history_fields: None,
                inspect_fields: None,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        println!("Looking for config at: {}", config_path.display());

        if !config_path.exists() {
            println!("Config file not found, creating default");
            let config = Config::default();
            std::fs::create_dir_all(config_path.parent().unwrap())?;
            std::fs::write(&config_path, toml::to_string_pretty(&config)?)?;
            return Ok(config);
        }

        println!("Loading config from: {}", config_path.display());
        let config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()?;

        let config: Config = config.try_deserialize()?;
        println!("Loaded workgroup: {:?}", config.aws.workgroup);

        Ok(config)
    }
}

fn get_config_path() -> Result<PathBuf> {
    // Always use XDG config dir (~/.config/athena-cli/config.toml)
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config/athena-cli/config.toml"));
    }

    // Fallback only if HOME is not available
    let proj_dirs = ProjectDirs::from("com", "your-org", "athena-cli")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    Ok(proj_dirs.config_dir().join("config.toml"))
}
