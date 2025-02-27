use aws_sdk_athena::Client;
use crate::cli::DatabaseArgs;
use anyhow::Result;

pub async fn list(client: Client, catalog: String) -> Result<()> {
    println!("Listing databases in catalog: {}", catalog);
    
    let result = client.list_databases().catalog_name(catalog).send().await?;
    

    Ok(())
} 