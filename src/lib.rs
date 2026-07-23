use crate::config::DatabaseConfig;

pub mod api;
pub mod app;
pub mod auth;
pub mod config;
pub mod key_pairs;
pub mod logs;
pub mod mailer;
pub mod models;
pub mod openapi;
pub mod services;
pub mod views;

pub async fn build_database(database_config: &DatabaseConfig) -> toasty::Result<toasty::Db> {
    let db_url = database_config.build_address();

    toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .pool_pre_ping(true)
        .connect(&db_url)
        .await
}
