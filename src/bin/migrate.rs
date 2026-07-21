use dotenv::dotenv;
use platform::config::get_configuration;
use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let config = Config::load()?;

    let configuration = get_configuration().expect("Failed to read configuration.");

    let database = platform::build_database(configuration.database).await?;

    let cli = ToastyCli::with_config(database, config);
    cli.parse_and_run().await?;

    Ok(())
}
