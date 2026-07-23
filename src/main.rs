use dotenv::dotenv;
use portal::auth::oauth_providers::OAuthProviders;
use portal::build_database;
use portal::key_pairs::ApplicationKeyPairs;
use portal::mailer::MailerContext;
use portal::{app::Application, config::get_configuration, logs::init_subscriber};
use std::env;
use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    init_subscriber(
        env!("CARGO_BIN_NAME").into(),
        env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        std::io::stdout,
    );

    info!("Reading configuration");
    let configuration = get_configuration().expect("Failed to read configuration.");

    info!("Generating application key pairs");
    let application_key_pairs =
        ApplicationKeyPairs::build(&configuration.application.secret_key_seed);

    info!("Getting Database connection");
    let database = build_database(&configuration.database).await?;

    info!("Creating mailer connection");
    let mailer_context = MailerContext::build(&configuration.mailer);

    info!("Building providers");
    let oauth_providers = OAuthProviders::build_all(&configuration.oauth);

    info!("Creating application");
    let application = Application::build(
        configuration,
        application_key_pairs,
        database,
        mailer_context,
        oauth_providers,
    )
    .await?;

    info!("Starting application");
    let application = tokio::spawn(application.run());

    info!("Started application");
    tokio::select! {
        o = application => report_exit("APPLICATION", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
