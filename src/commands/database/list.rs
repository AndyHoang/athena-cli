use crate::context::Context;
use crate::utils::display::DatabaseDisplay;
use anyhow::Result;

pub async fn list(ctx: &Context) -> Result<()> {
    let client = ctx.create_athena_client();

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
    println!("Databases in catalog: {}", ctx.catalog());

    let table = DatabaseDisplay::create_databases_table(databases);
    table.printstd();

    Ok(())
}
