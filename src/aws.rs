use anyhow::Result;

/// Builds and returns an AWS SDK configuration based on the following priority:
/// 1. Specified AWS profile (if provided)
/// 2. AWS environment variables (if available)
/// 3. Interactive SSO login (if user confirms)
///
/// This function can be reused to create any AWS service client.
pub async fn build_aws_config(
    profile: Option<String>,
    region: String,
) -> Result<aws_config::SdkConfig> {
    let mut builder = aws_config::from_env();
    
    if let Some(profile_name) = profile {
        builder = builder.profile_name(profile_name);
    }
    
    builder = builder.region(aws_config::Region::new(region));
    
    Ok(builder.load().await)
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