use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_config::profile::ProfileFileCredentialsProvider;
use dialoguer::{Confirm, Input};
use std::process::Command;

/// Builds and returns an AWS SDK configuration based on the following priority:
/// 1. Specified AWS profile (if provided)
/// 2. AWS environment variables (if available)
/// 3. Interactive SSO login (if user confirms)
///
/// This function can be reused to create any AWS service client.
pub async fn build_aws_config(profile: Option<String>, region: String) -> Result<aws_config::SdkConfig> {
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

    Ok(aws_config)
}

/// Helper function to handle common AWS authentication errors with helpful messages
pub fn handle_aws_auth_error(err: anyhow::Error, profile: Option<String>) -> anyhow::Error {
    let err_string = format!("{:?}", err);
    
    if err_string.contains("ForbiddenException") || 
       err_string.contains("AccessDenied") ||
       err_string.contains("ExpiredToken") ||
       err_string.contains("credentials") || 
       err_string.contains("auth") {
        
        println!("AWS Authentication Error: Your credentials may be expired or insufficient.");
        
        if let Some(profile_name) = profile {
            println!("\nPlease run: aws sso login --profile {}", profile_name);
        } else {
            println!("\nPlease set valid AWS credentials or configure a profile.");
        }
        
        anyhow::anyhow!("Authentication failure")
    } else {
        err
    }
}

/// Example function to create an S3 client using the same AWS configuration
/// 
/// Usage example:
/// ```
/// let s3_client = aws::create_s3_client(profile.clone(), region.clone()).await?;
/// ```
pub async fn create_s3_client(profile: Option<String>, region: String) -> Result<aws_sdk_s3::Client> {
    let aws_config = build_aws_config(profile, region).await?;
    Ok(aws_sdk_s3::Client::new(&aws_config))
}

/// Generic function to create any AWS service client using the same configuration
/// 
/// This is a more flexible approach that can be used for any AWS service
pub async fn create_aws_client<T, F>(
    profile: Option<String>, 
    region: String,
    client_factory: F
) -> Result<T>
where
    F: FnOnce(&aws_config::SdkConfig) -> T,
{
    let aws_config = build_aws_config(profile, region).await?;
    Ok(client_factory(&aws_config))
}

// Example usage of the generic client creator:
// 
// ```
// // Create an S3 client
// let s3_client = aws::create_aws_client(
//     profile.clone(), 
//     region.clone(),
//     |config| aws_sdk_s3::Client::new(config)
// ).await?;
// 
// // Create a DynamoDB client
// let dynamodb_client = aws::create_aws_client(
//     profile.clone(), 
//     region.clone(),
//     |config| aws_sdk_dynamodb::Client::new(config)
// ).await?;
// ``` 