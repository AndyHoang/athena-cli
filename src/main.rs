mod aws;
mod cli;
mod commands;
mod config;
mod context;
mod utils;
mod validation;

use anyhow::Result;
use clap::Parser;
use context::Context;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::Config::load()?;

    // Create global context
    let ctx = Context::new(config, cli.aws, cli.display).await?;

    // Execute command with context
    let result = match &cli.command {
        cli::Commands::Query(args) => commands::query::execute(&ctx, args).await,
        cli::Commands::Database { command } => match command {
            cli::DatabaseCommands::List(args) => commands::database::list(&ctx, args).await,
        },
        cli::Commands::Table { command } => match command {
            cli::TableCommands::List(args) => commands::database::list_tables(&ctx, args).await,
            cli::TableCommands::Describe(args) => {
                commands::database::describe_table(&ctx, args).await
            }
        },
        cli::Commands::Workgroup { command } => match command {
            cli::WorkgroupCommands::List(args) => commands::workgroup::list(&ctx, args).await,
        },
        cli::Commands::History(args) => commands::history::list(&ctx, args).await,
        cli::Commands::Inspect(args) => commands::inspect::inspect(&ctx, args).await,
        cli::Commands::Download(args) => commands::inspect::download(&ctx, args).await,
    };

    // Handle credential errors
    if let Err(err) = result {
        return Err(aws::handle_aws_auth_error(err, ctx.profile()));
    }

    Ok(())
}
