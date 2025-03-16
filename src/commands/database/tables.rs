use crate::cli::TableArgs;
use crate::context::Context;
use crate::utils::display::TableMetadataDisplay;
use crate::utils::filter;
use anyhow::{Context as _, Result};

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

    // No server-side filtering - we'll filter client-side instead

    let result = request.send().await.context("Failed to list tables")?;

    let tables = result.table_metadata_list();

    // Debug: Print all tables from server
    println!("DEBUG: Received {} tables from server", tables.len());
    if !tables.is_empty() {
        println!("DEBUG: First few table names:");
        for (i, table) in tables.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, table.name());
        }
    }

    if tables.is_empty() {
        println!("No tables found in database: {}", database);
        return Ok(());
    }

    // Apply filter if specified
    let filtered_tables = if let Some(filter_pattern) = &args.filter {
        println!("DEBUG: Applying filter pattern: '{}'", filter_pattern);

        // Use filter_items from the utils module
        let filtered = filter::filter_items(tables, Some(filter_pattern), |table| table.name());

        println!(
            "DEBUG: Filter reduced tables from {} to {}",
            tables.len(),
            filtered.len()
        );
        filtered
    } else {
        tables.iter().collect()
    };

    if filtered_tables.is_empty() {
        println!(
            "No tables found matching filter: {}",
            args.filter.as_ref().unwrap()
        );
        return Ok(());
    }

    // Display tables
    println!(
        "Tables in database: {} (filtered: {})",
        database,
        args.filter.as_deref().unwrap_or("none")
    );

    // Create a pretty table using our display struct
    let table = TableMetadataDisplay::create_table_metadata_table(&filtered_tables);
    table.printstd();

    Ok(())
}
