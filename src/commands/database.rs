use crate::context::Context;
use anyhow::Result;

pub async fn list(ctx: &Context) -> Result<()> {
    let client = ctx.create_athena_client();

    let _result = client
        .list_databases()
        .catalog_name(ctx.catalog())
        .send()
        .await?;

    Ok(())
}
