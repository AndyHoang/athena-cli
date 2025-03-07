use aws_sdk_athena::Client;
use anyhow::Result;
use prettytable::{Table, Row, Cell, format};
use owo_colors::OwoColorize;
use crate::cli::InspectArgs;
use super::fields::{get_inspect_fields, get_field_value};
use polars::prelude::*;
use super::download::download_from_s3;

pub async fn detail(client: Client, args: InspectArgs) -> Result<()> {
    let query_id = args.query_id.clone();
    
    println!("\n{}", "Query Execution Details".bold());
    println!("ID: {}\n", query_id.bright_green());
    
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
    
    // Configure table style
    table.set_format(*format::consts::FORMAT_CLEAN); // Clean borders
    
    // Get fields to display
    let fields = get_inspect_fields();
    
    // Add header
    table.add_row(Row::new(vec![
        Cell::new("Field").style_spec("Fb"),  // Bold
        Cell::new("Value").style_spec("Fb"),  // Bold
    ]));
    
    // Add rows for each field
    for field in fields {
        let value = get_field_value(execution, field);
        let formatted_value = match field.to_string().as_str() {
            "Status" => match value.as_str() {
                "SUCCEEDED" => value.bright_green().to_string(),
                "FAILED" => value.bright_red().to_string(),
                _ => value.yellow().to_string(),
            },
            "Data Scanned" => value.bright_cyan().to_string(),
            _ => value,
        };
        
        table.add_row(Row::new(vec![
            Cell::new(&field.to_string()).style_spec("Fb"),  // Bold field names
            Cell::new(&formatted_value),
        ]));
    }
    
    // Print the table
    table.printstd();
    
    // Check if query was successful before trying to get results
    if let Some(status) = execution.status() {
        if let Some(state) = status.state() {
            if state.as_str() == "SUCCEEDED" {
                // If output option is provided, download results from S3
                if let Some(output_dir) = &args.output {
                    // Get S3 output location
                    let s3_output_location = execution.result_configuration()
                        .and_then(|c| c.output_location())
                        .ok_or_else(|| anyhow::anyhow!("No output location found for query: {}", query_id))?;
                    
                    println!("\n{}", "S3 Output Location:".bold());
                    println!("📂 {}", s3_output_location.bright_blue());
                    
                    let s3_client = crate::aws::create_s3_client(None, "eu-west-1".to_string()).await?;
                    
                    println!("\n{}", "Downloading Results...".bold());
                    match download_from_s3(&s3_client, s3_output_location, output_dir, &query_id).await {
                        Ok(file_path) => println!("✅ Downloaded to: {}", file_path.display().to_string().bright_green()),
                        Err(e) => println!("❌ Error: {}", e.to_string().bright_red()),
                    }
                }
            } else {
                println!("\n{}", "Cannot display results:".bold());
                println!("❌ Query status is {}", state.as_str().bright_red());
            }
        }
    }
    
    println!(); // Add final newline
    Ok(())
}