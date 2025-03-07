use anyhow::Result;
use crate::context::Context;

pub async fn list(ctx: &Context) -> Result<()> {
    let client = ctx.create_athena_client();
    
    let result = client
        .list_databases()
        .catalog_name(ctx.catalog())
        .send()
        .await?;
    
    Ok(())
} 