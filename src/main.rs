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
    
    // First check for profile in CLI args or config file
    let profile = cli.command.get_profile().or(config.aws.profile.clone());
    let profile_for_errors = profile.clone();
    
    let aws_config = if let Some(name) = profile {
        // Use the specified profile
        println!("Using AWS profile: {}", name);
        
        let provider = ProfileFileCredentialsProvider::builder()
            .profile_name(&name)
            .build();
            
        aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(provider)
            .region(Region::new(region))
            .load()
            .await
    } else if std::env::var("AWS_ACCESS_KEY_ID").is_ok() && 
              std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
        // Fallback to environment variables if available
        println!("Using AWS credentials from environment variables");
        
        aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region))
            .load()
            .await
    } else {
        // No credentials found, prompt for login
        println!("No AWS credentials found in profile or environment variables");
        
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
                println!("SSO login failed. Please try again manually with:");
                println!("aws sso login --profile {}", profile);
                return Err(anyhow::anyhow!("Please rerun the program after logging in"));
            }
            
            let provider = ProfileFileCredentialsProvider::builder()
                .profile_name(&profile)
                .build();
                
            aws_config::defaults(BehaviorVersion::latest())
                .credentials_provider(provider)
                .region(Region::new(region))
                .load()
                .await
        } else {
            return Err(anyhow::anyhow!("AWS credentials are required to continue"));
        }
    };

    // Create Athena client and execute command
    let client = aws_sdk_athena::Client::new(&aws_config);
    let result = match cli.command {
        cli::Commands::Query(args) => commands::query::execute(client, args).await,
        cli::Commands::ListDatabases(args) => commands::database::list(client, args).await,
        cli::Commands::ListWorkgroups(args) => commands::workgroup::list(client, args).await,
        cli::Commands::History(args) => commands::history::list(client, args).await,
    };
    
    // Handle credential errors with helpful suggestions
    if let Err(err) = result {
        let err_string = format!("{:?}", err);
        
        if err_string.contains("ForbiddenException") || 
           err_string.contains("AccessDenied") ||
           err_string.contains("ExpiredToken") ||
           err_string.contains("credentials") || 
           err_string.contains("auth") {
            
            println!("AWS Authentication Error: Your credentials may be expired or insufficient.");
            
            if let Some(profile_name) = profile_for_errors {
                println!("\nPlease run: aws sso login --profile {}", profile_name);
            } else {
                println!("\nPlease set valid AWS credentials or configure a profile.");
            }
            
            return Err(anyhow::anyhow!("Authentication failure"));
        }
        
        return Err(err);
    }
    
    Ok(())
}
