use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::ConnectOptions;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email: EmailSettings,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}
impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            sqlx::postgres::PgSslMode::Require
        } else {
            sqlx::postgres::PgSslMode::Disable
        };
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password)
            .ssl_mode(ssl_mode)
    }
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(log::LevelFilter::Trace)
    }
}

pub fn get() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let environment = std::env::var("ZERO_ENVIRONMENT").unwrap_or("local".to_string());
    let base = configuration_directory.join("base");
    let file = configuration_directory.join(environment);
    let config = config::Config::builder()
        .add_source(config::File::from(base).required(true))
        .add_source(config::File::from(file).required(false))
        .add_source(
            config::Environment::with_prefix("zero")
                .try_parsing(true)
                .separator("_"),
        )
        .build()?;
    config.try_deserialize()
}
#[derive(Deserialize, Debug)]
pub struct EmailSettings {
    pub base_url: String,
    pub sender: String,
    pub apikey: String,
    pub timeout: u64,
}
impl EmailSettings {
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout)
    }
}
