mod cli;
mod commands;
mod config;
mod aws;

use clap::Parser;
use anyhow::Result;
use aws_sdk_athena::Client as AthenaClient;
// Uncomment if you need to use S3 client
// use aws_sdk_s3::Client as S3Client;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::Config::load()?;
    
    // With global=true, these args are available directly from cli.aws
    let profile = cli.aws.profile.or(config.aws.profile.clone());
    let region = config.aws.region.unwrap_or_else(|| "eu-west-1".to_string());
    
    // Build AWS configuration using our extracted module
    let aws_config = aws::build_aws_config(profile.clone(), region.clone()).await?;

    // Create Athena client
    let client = AthenaClient::new(&aws_config);
    
    // Example of creating an S3 client using the same configuration
    // This is commented out since it's not used in this application yet
    // let s3_client = aws::create_s3_client(profile.clone(), region.clone()).await?;
    
    // Alternative way to create clients using the generic function
    // let s3_client = aws::create_aws_client::<aws_sdk_s3::Client, _>(
    //     profile.clone(), 
    //     region.clone(),
    //     |config| aws_sdk_s3::Client::new(config)
    // ).await?;
    
    // Pass the appropriate arguments to each command
    let result = match &cli.command {
        cli::Commands::Query(args) => {
            let database = cli.aws.database.or(config.aws.database.clone());
            let workgroup = cli.aws.workgroup.or(config.aws.workgroup.clone());
            commands::query::execute(
                client, 
                args.clone(), 
                database, 
                workgroup
            ).await
        },
        cli::Commands::ListDatabases(_) => {
            let catalog = cli.aws.catalog.unwrap_or_else(|| "AwsDataCatalog".to_string());
            commands::database::list(client, catalog).await
        },
        cli::Commands::ListWorkgroups(args) => {
            commands::workgroup::list(client, args.clone()).await
        },
        cli::Commands::History(args) => {
            let workgroup = cli.aws.workgroup.or(config.aws.workgroup.clone());
            commands::history::list(client, args.clone(), workgroup).await
        },
        cli::Commands::Inspect(args) => {
            commands::inspect::inspect(client, args.clone()).await
        },
    };
    
    // Handle credential errors with helpful suggestions
    if let Err(err) = result {
        return Err(aws::handle_aws_auth_error(err, profile));
    }
    
    Ok(())
}
