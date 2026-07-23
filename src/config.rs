use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::env;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Configuration {
    pub application: ApplicationConfig,
    pub database: DatabaseConfig,
    pub mailer: MailerConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplicationConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub secret_key_seed: SecretBox<str>,
    pub api_key: SecretBox<str>,
}

impl ApplicationConfig {
    pub fn build_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretBox<str>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub db_port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseConfig {
    pub fn build_address(&self) -> String {
        let ssl_mode = if self.require_ssl {
            "sslmode=verify-full"
        } else {
            ""
        };

        format!(
            "postgres://{}:{}@{}:{}/{}{ssl_mode}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.db_port,
            self.database_name
        )
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MailerConfig {
    pub from: String,
    pub host: String,
    pub username: String,
    pub api_key: SecretBox<str>,
    pub domain: String,
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    let configuration_dir = env::current_dir()
        .expect("Failed to get current directory")
        .join("./configuration");

    let app_env = Environment::app_env();

    let configurations = config::Config::builder()
        .add_source(config::File::from(
            configuration_dir.join(format!("{}.yaml", app_env.as_str())),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    configurations.try_deserialize::<Configuration>()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Dev,
    Production,
}

impl Environment {
    pub fn app_env() -> Self {
        env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "dev".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.")
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Supported environments are: `dev` or `production`.",
                other
            )),
        }
    }
}
