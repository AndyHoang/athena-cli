mod cli;
mod commands;
mod config;

use clap::Parser;
use anyhow::Result;
use aws_config::{Region, BehaviorVersion};
use dialoguer::{Confirm, Input};
use std::process::Command;
use aws_config::profile::ProfileFileCredentialsProvider;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let config = config::Config::load()?;
    let region = config.aws.region.unwrap_or_else(|| "eu-west-1".to_string());
    
    // Try to get profile from CLI args first, then config file
    let profile = cli.command.get_profile().or(config.aws.profile.clone());
    let profile_name = if let Some(name) = profile {
        println!("Using AWS profile from {}: {}", 
            if cli.command.get_profile().is_some() { "CLI args" } else { "config file" },
            name);
        name
    } else {
        println!("No profile specified in CLI args or config file");
        if Confirm::new()
            .with_prompt("Would you like to login with AWS SSO?")
            .default(true)
            .interact()? 
        {
            let profile: String = Input::new()
                .with_prompt("Enter your AWS profile name")
                .interact()?;

            println!("Initiating AWS SSO login...");
            let status = Command::new("aws")
                .args(["sso", "login", "--profile", &profile])
                .status()?;

            if !status.success() {
                println!("SSO login failed. Please try again with:");
                println!("aws sso login --profile {}", profile);
                return Err(anyhow::anyhow!("Please rerun the program after logging in"));
            }
            profile
        } else {
            return Err(anyhow::anyhow!("AWS profile is required to continue"));
        }
    };

    let provider = ProfileFileCredentialsProvider::builder()
        .profile_name(&profile_name)
        .build();

    // Build AWS config with credentials provider
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(provider)
        .region(Region::new(region))
        .load()
        .await;

    // Create a test client to verify credentials
    let client = aws_sdk_athena::Client::new(&aws_config);
    let workgroups_result = client.list_work_groups()
        .send()
        .await;
    
    let creds_valid = workgroups_result.is_ok();
    println!("Credentials validation result: {}", creds_valid);
    if let Err(err) = &workgroups_result {
        println!("Credentials error: {:?}", err);
        return Err(anyhow::anyhow!("Invalid or expired credentials. Please run 'aws sso login --profile {}'", profile_name));
    }
    
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
