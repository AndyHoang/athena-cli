mod cli;
mod commands;
mod config;

use clap::Parser;
use anyhow::Result;
use aws_config::{Region, BehaviorVersion};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    
    let config = config::Config::load()?;
    
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(config.aws.region.unwrap_or_else(|| "eu-west-1".to_string())))
        .load()
        .await;
    let client = aws_sdk_athena::Client::new(&config);
    
    match cli.command {
        cli::Commands::Query(args) => {
            commands::query::execute(client, args).await?;
        }
        cli::Commands::ListDatabases(args) => {
            commands::database::list(client, args).await?;
        }
        cli::Commands::ListWorkgroups(args) => {
            commands::workgroup::list(client, args).await?;
        }
    }
    
    Ok(())
}
