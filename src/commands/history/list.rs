use aws_sdk_athena::Client;
use anyhow::Result;
use prettytable::{Table, Row, Cell};
use crate::cli::HistoryArgs;
use crate::config;
use super::fields::get_field_value;

pub async fn list(client: Client, args: HistoryArgs, workgroup: Option<String>) -> Result<()> {
    let config = config::Config::load()?;
    
    let workgroup = workgroup
        .or_else(|| config.aws.workgroup.clone())
        .ok_or_else(|| anyhow::anyhow!("Workgroup is required but was not provided"))?;

    // Use limit from CLI args if provided, otherwise from config
    let limit = args.limit.unwrap_or(config.app.history_size);

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
    let executions_map: std::collections::HashMap<String, &aws_sdk_athena::types::QueryExecution> = 
        details.query_executions()
            .iter()
            .filter_map(|exec| {
                exec.query_execution_id().map(|id| (id.to_string(), exec))
            })
            .collect();

    // Process query IDs in the original order
    let mut table = Table::new();
    let fields = super::fields::get_history_fields();

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
                    .map(|&field| Cell::new(&get_field_value(execution, field)))
                    .collect()
            );
            table.add_row(row);
        }
    }
    
    table.printstd();
    Ok(())
} 