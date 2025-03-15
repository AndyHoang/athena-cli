use crate::context::Context;
use anyhow::Result;
use prettytable::{Cell, Row, Table};

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

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Name").style_spec("Fb"),
        Cell::new("Description").style_spec("Fb"),
    ]));

    for db in databases {
        let name = db.name();
        let description = db.description().unwrap_or("");

        table.add_row(Row::new(vec![Cell::new(name), Cell::new(description)]));
    }

    table.printstd();
    Ok(())
}
