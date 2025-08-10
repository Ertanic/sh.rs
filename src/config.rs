use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    pub port: Option<u16>,
    pub logs: LogsConfig,
    pub database: Option<String>,
    pub redis: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct LogsConfig {
    pub debug: bool,
    pub time_format: Option<String>,
    pub folder: Option<String>,
}

pub async fn load_config() -> Config {
    let file_content = tokio::fs::read_to_string("config.toml")
        .await
        .unwrap_or_default();

    toml::from_str(&file_content).unwrap_or_default()
}