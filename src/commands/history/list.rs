use anyhow::Result;
use prettytable::{Table, Row, Cell};
use crate::cli::HistoryArgs;
use super::fields::{get_field_value, HistoryField};
use std::collections::HashMap;
use crate::context::Context;

pub async fn list(ctx: &Context, args: &HistoryArgs) -> Result<()> {
    let client = ctx.create_athena_client();
    let workgroup = ctx.workgroup();

    // Use limit from CLI args if provided, otherwise from config
    let limit = args.limit.unwrap_or_else(|| ctx.history_size());

    let result = client
        .list_query_executions()
        .work_group(&workgroup)
        .max_results(limit)
        .send()
        .await?;

    // Get query IDs
    let query_ids = result.query_execution_ids();
    if query_ids.is_empty() {
        println!("No queries found in workgroup: {}", workgroup);
        return Ok(());
    }
    
    println!("Found {} queries in workgroup: {}", query_ids.len(), workgroup);
    
    // Get details for all queries in a single batch request
    let details = client
        .batch_get_query_execution()
        .set_query_execution_ids(Some(query_ids.to_vec()))
        .send()
        .await?;

    // Create a map of query ID to execution for quick lookup
    let executions_map: HashMap<String, &aws_sdk_athena::types::QueryExecution> = 
        details.query_executions()
            .iter()
            .filter_map(|exec| {
                exec.query_execution_id().map(|id| (id.to_string(), exec))
            })
            .collect();
    
    // Only fetch row counts if the RowCount field is being displayed
    let fields = super::fields::get_history_fields();
    let mut row_counts: HashMap<String, String> = HashMap::new();
    
    if fields.contains(&HistoryField::RowCount) {
        // Get only SUCCEEDED query IDs to minimize API calls
        let succeeded_query_ids: Vec<String> = query_ids.iter()
            .filter(|&id| {
                if let Some(execution) = executions_map.get(id) {
                    if let Some(status) = execution.status().and_then(|s| s.state()) {
                        return status.as_str() == "SUCCEEDED";
                    }
                }
                false
            })
            .map(|id| id.to_string())
            .collect();
        
        // Fetch row counts for successful queries in batches to reduce API calls
        for chunk in succeeded_query_ids.chunks(10) {
            for query_id in chunk {
                match client
                    .get_query_runtime_statistics()
                    .query_execution_id(query_id)
                    .send()
                    .await {
                    Ok(stats) => {
                        if let Some(rows) = stats.query_runtime_statistics().and_then(|s| s.rows()) {
                            if let Some(output_rows) = rows.output_rows() {
                                row_counts.insert(query_id.clone(), output_rows.to_string());
                            }
                        }
                    },
                    Err(e) => {
                        // Log the error but continue processing
                        eprintln!("Failed to get row count for query {}: {}", query_id, e);
                    }
                }
            }
        }
    }

    // Process query IDs in the original order
    let mut table = Table::new();

    // Add header row
    let header_row = Row::new(
        fields.iter()
            .map(|field| Cell::new(&field.to_string()))
            .collect()
    );
    table.add_row(header_row);

    // Add data rows in the original order from query_ids
    for query_id in query_ids {
        if let Some(execution) = executions_map.get(query_id) {
            // Filter by status if specified
            if let Some(status_filter) = &args.status {
                if let Some(status) = execution.status().and_then(|s| s.state()) {
                    if status.as_str() != status_filter.to_uppercase() {
                        continue;
                    }
                }
            }
            
            // Create a row with values for each field
            let row = Row::new(
                fields.iter()
                    .map(|&field| {
                        if field == HistoryField::RowCount {
                            // Use the row count from our map if available
                            if let Some(count) = row_counts.get(execution.query_execution_id().unwrap_or_default()) {
                                Cell::new(count)
                            } else {
                                Cell::new("-")
                            }
                        } else {
                            Cell::new(&get_field_value(execution, field))
                        }
                    })
                    .collect()
            );
            table.add_row(row);
        }
    }
    
    table.printstd();
    Ok(())
}

 