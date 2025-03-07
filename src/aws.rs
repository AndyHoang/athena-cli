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