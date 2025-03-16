use crate::cli::DescribeTableArgs;
use crate::context::Context;
use anyhow::{Context as _, Result};
use prettytable::{Cell, Row, Table};

pub async fn describe_table(ctx: &Context, args: &DescribeTableArgs) -> Result<()> {
    let client = ctx.create_athena_client();

    // Parse database and table names
    let (database_name, table_name) = if args.table.contains('.') {
        let parts: Vec<&str> = args.table.splitn(2, '.').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else if let Some(db) = args.db.as_ref() {
        (db.clone(), args.table.clone())
    } else if let Some(db) = ctx.database().as_ref() {
        (db.clone(), args.table.clone())
    } else {
        anyhow::bail!("No database specified. Use --db or 'database.table' format")
    };

    // Get table metadata
    let result = client
        .get_table_metadata()
        .catalog_name(ctx.catalog())
        .database_name(&database_name)
        .table_name(&table_name)
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to get metadata for table {}.{}",
                database_name, table_name
            )
        })?;

    let table_metadata = result.table_metadata().ok_or_else(|| {
        anyhow::anyhow!(
            "No metadata found for table {}.{}",
            database_name,
            table_name
        )
    })?;

    // Display table info
    println!("Table: {}.{}", database_name, table_name);
    println!();

    // Display table properties
    if let Some(table_type) = table_metadata.table_type() {
        println!("Type: {}", table_type);
    }

    if let Some(create_time) = table_metadata.create_time() {
        println!("Created: {}", create_time);
    }

    if let Some(description) = table_metadata.parameters().and_then(|p| p.get("comment")) {
        println!("Description: {}", description);
    }

    // Display columns
    let columns = table_metadata.columns();
    println!("\nColumns: (found {})", columns.len());
    if !columns.is_empty() {
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Name").style_spec("Fb"),
            Cell::new("Type").style_spec("Fb"),
            Cell::new("Description").style_spec("Fb"),
        ]));

        // Display each column
        for column in columns {
            table.add_row(Row::new(vec![
                Cell::new(
                    &format!("{:?}", column.name())
                        .replace("Some(\"", "")
                        .replace("\")", ""),
                ),
                Cell::new(
                    &format!("{:?}", column.r#type())
                        .replace("Some(\"", "")
                        .replace("\")", ""),
                ),
                Cell::new(
                    &format!("{:?}", column.comment())
                        .replace("Some(\"", "")
                        .replace("\")", ""),
                ),
            ]));
        }

        table.printstd();
    } else {
        println!("No columns found in table metadata");
    }

    // Display partitions
    let partitions = table_metadata.partition_keys();

    // Always show partition information
    println!("\nPartition Details:");
    if partitions.is_empty() {
        println!("Table is not partitioned");
    } else {
        println!("Table has {} partition keys", partitions.len());

        // Display partition keys in a table
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Name").style_spec("Fb"),
            Cell::new("Type").style_spec("Fb"),
        ]));

        for partition in partitions {
            table.add_row(Row::new(vec![
                Cell::new(
                    &format!("{:?}", partition.name())
                        .replace("Some(\"", "")
                        .replace("\")", ""),
                ),
                Cell::new(
                    &format!("{:?}", partition.r#type())
                        .replace("Some(\"", "")
                        .replace("\")", ""),
                ),
            ]));
        }

        table.printstd();

        println!("\nDetailed partition information is available through SQL with:");
        println!("SHOW PARTITIONS {}.{}", database_name, table_name);
    }

    // Display storage parameters
    if let Some(parameters) = table_metadata.parameters() {
        println!("\nStorage Parameters:");
        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Parameter").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));

        for (key, value) in parameters {
            // Skip comment as we already displayed it
            if key == "comment" {
                continue;
            }
            table.add_row(Row::new(vec![Cell::new(key), Cell::new(value)]));
        }

        table.printstd();
    }

    Ok(())
}
