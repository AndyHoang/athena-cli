use super::utils::DatabaseDisplay;
use crate::cli::DatabaseArgs;
use crate::context::Context;
use anyhow::Result;

pub async fn list(ctx: &Context, args: &DatabaseArgs) -> Result<()> {
    let client = ctx.create_athena_client();

    // Use workgroup from args if provided
    let workgroup = args
        .aws
        .workgroup
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ctx.workgroup());

    let result = client
        .list_databases()
        .catalog_name(ctx.catalog())
        .send()
        .await?;

    let databases = result.database_list();

    if databases.is_empty() {
        println!("No databases found in catalog: {}", ctx.catalog());
        return Ok(());
    }

    // Display databases in a simple list
    println!(
        "Databases in catalog: {} (workgroup: {})",
        ctx.catalog(),
        workgroup
    );

    let table = DatabaseDisplay::create_databases_table(databases);
    table.printstd();

    Ok(())
}
