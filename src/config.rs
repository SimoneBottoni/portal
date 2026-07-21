// use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::env;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Configuration {
    pub application: ApplicationConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplicationConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

impl ApplicationConfig {
    pub fn build_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
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
