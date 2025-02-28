use aws_sdk_athena::Client;
use anyhow::Result;
use prettytable::{Table, Row, Cell};
use crate::cli::InspectArgs;
use super::fields::{get_inspect_fields, get_field_value};

pub async fn detail(client: Client, args: InspectArgs) -> Result<()> {
    let query_id = args.query_id;
    
    println!("Inspecting query execution: {}", query_id);
    
    // Get query execution details
    let result = client
        .get_query_execution()
        .query_execution_id(&query_id)
        .send()
        .await?;
    
    let execution = result.query_execution().ok_or_else(|| {
        anyhow::anyhow!("No query execution found with ID: {}", query_id)
    })?;
    
    // Create a table for the query information
    let mut table = Table::new();
    
    // Get fields to display
    let fields = get_inspect_fields();
    
    // Add rows for each field
    for field in fields {
        table.add_row(Row::new(vec![
            Cell::new(&field.to_string()),
            Cell::new(&get_field_value(execution, field)),
        ]));
    }
    
    // Print the table
    table.printstd();
    
    Ok(())
} 