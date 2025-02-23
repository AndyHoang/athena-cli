use aws_sdk_athena::Client;
use crate::cli::DatabaseArgs;
use anyhow::Result;

pub async fn list(client: Client, args: DatabaseArgs) -> Result<()> {
    println!("Listing databases in catalog: {}", args.catalog);
    
    let result = client.list_databases().catalog_name(args.catalog).send().await?;
    

    Ok(())
} 