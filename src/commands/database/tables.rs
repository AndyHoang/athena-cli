use crate::cli::TableArgs;
use crate::context::Context;
use anyhow::{Context as _, Result};
use prettytable::{Cell, Row, Table};

pub async fn list_tables(ctx: &Context, args: &TableArgs) -> Result<()> {
    let client = ctx.create_athena_client();

    // Determine which database to use (command arg takes precedence)
    let database = if let Some(db) = &args.db {
        db.clone()
    } else if let Some(db) = ctx.database() {
        db
    } else {
        anyhow::bail!("No database specified. Use --db or set a default database in config")
    };

    let mut request = client
        .list_table_metadata()
        .catalog_name(ctx.catalog())
        .database_name(&database);

    // Apply limit
    request = request.max_results(args.limit);

    // If filter is specified, apply it
    if let Some(filter) = &args.filter {
        request = request.expression(format!(r#"TableName LIKE '{}%'"#, filter));
    }

    let result = request.send().await.context("Failed to list tables")?;

    let tables = result.table_metadata_list();

    if tables.is_empty() {
        println!("No tables found in database: {}", database);
        return Ok(());
    }

    // Create a pretty table instead of plain text output
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Name").style_spec("Fb"),
        Cell::new("Type").style_spec("Fb"),
        Cell::new("Columns").style_spec("Fb"),
    ]));

    for table_meta in tables {
        let name = table_meta.name();
        let table_type = table_meta.table_type().unwrap_or("Unknown");
        let column_count = table_meta.columns().len();

        table.add_row(Row::new(vec![
            Cell::new(name),
            Cell::new(table_type),
            Cell::new(&column_count.to_string()),
        ]));
    }

    table.printstd();

    Ok(())
}
