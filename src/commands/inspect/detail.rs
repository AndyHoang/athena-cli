use aws_sdk_athena::Client;
use anyhow::Result;
use prettytable::{Table, Row, Cell};
use crate::cli::InspectArgs;
use super::fields::{get_inspect_fields, get_field_value};
use polars::prelude::*;
use super::download::download_from_s3;
use aws_sdk_s3;
use aws_config::BehaviorVersion;

pub async fn detail(client: Client, args: InspectArgs) -> Result<()> {
    let query_id = args.query_id.clone();
    
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
                    
                    println!("Query results S3 location: {}", s3_output_location);
                    
                    // Use the aws module's create_s3_client function with required parameters
                    let s3_client = crate::aws::create_s3_client(None, "eu-west-1".to_string()).await?;
                    
                    println!("Attempting to download from S3...");
                    match download_from_s3(&s3_client, s3_output_location, output_dir, &query_id).await {
                        Ok(file_path) => println!("\nQuery results downloaded to: {}", file_path.display()),
                        Err(e) => println!("\nError downloading results: {}", e),
                    }
                } else {
                    // Otherwise display results in console
                }
            } else {
                println!("\nCannot display results: Query status is {}", state.as_str());
            }
        }
    }
    
    Ok(())
}