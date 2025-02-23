use aws_sdk_athena::Client;
use crate::cli::QueryArgs;
use anyhow::Result;
use aws_sdk_athena::types::{QueryExecutionContext, ResultConfiguration, QueryExecutionState};
use polars::prelude::*;
use std::{thread, time::Duration};

pub async fn execute(client: Client, args: QueryArgs) -> Result<()> {
    println!("Executing query: {}", args.query);
    
    let query_id = start_query(
        &client,
        &args.database,
        &args.query,
        &args.workgroup
    ).await?;
    
    println!("Query execution ID: {}", query_id);

    let df = get_query_results(&client, &query_id).await?;
    println!("Results DataFrame:");
    println!("{}", df);

    Ok(())
}

async fn start_query(
    client: &Client,
    database: &str,
    query: &str,
    workgroup: &str,
) -> Result<String> {
    let context = QueryExecutionContext::builder()
        .database(database)
        .build();

    let config = ResultConfiguration::builder()
        .output_location("s3://pp-data-lake-athena-dump/")  // Configure this based on your S3 bucket
        .build();

    let result = client
        .start_query_execution()
        .query_string(query)
        .query_execution_context(context)
        .result_configuration(config)
        .work_group(workgroup)
        .send()
        .await?;

    Ok(result.query_execution_id().unwrap_or_default().to_string())
}

async fn get_query_results(
    client: &Client,
    query_execution_id: &str,
) -> Result<DataFrame> {
    // Wait for query to complete
    loop {
        let status = client
            .get_query_execution()
            .query_execution_id(query_execution_id)
            .send()
            .await?;

        if let Some(execution) = status.query_execution() {
            match execution.status().unwrap().state().as_ref() {
                Some(QueryExecutionState::Succeeded) => break,
                Some(QueryExecutionState::Failed) | Some(QueryExecutionState::Cancelled) => {
                    return Err(anyhow::anyhow!("Query failed or was cancelled"));
                }
                _ => {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        }
    }

    let mut all_columns: Vec<Vec<String>> = Vec::new();
    let mut column_names: Vec<String> = Vec::new();
    let mut next_token: Option<String> = None;

    // Get first page and column names
    let mut results = client
        .get_query_results()
        .query_execution_id(query_execution_id)
        .max_results(100)
        .send()
        .await?;

    // Initialize column names from first result
    if let Some(rs) = results.result_set() {
        if let Some(first_row) = rs.rows().get(0) {
            column_names = first_row.data()
                .iter()
                .map(|d| d.var_char_value().unwrap_or_default().to_string())
                .collect();
            all_columns = vec![Vec::new(); column_names.len()];
        }
    }

    // Process results page by page
    let mut page_count = 1;
    loop {
        if let Some(rs) = results.result_set() {
            let start_idx = if next_token.is_none() { 1 } else { 0 };
            let rows_count = rs.rows().len() - start_idx;
            
            println!("Processing page {}: {} rows", page_count, rows_count);
            
            for row in rs.rows().iter().skip(start_idx) {
                for (i, data) in row.data().iter().enumerate() {
                    all_columns[i].push(data.var_char_value().unwrap_or_default().to_string());
                }
            }
        }

        next_token = results.next_token().map(|s| s.to_string());
        
        if next_token.is_none() {
            println!("Finished processing {} pages, total rows: {}", page_count, all_columns[0].len());
            break;
        }

        page_count += 1;
        results = client
            .get_query_results()
            .query_execution_id(query_execution_id)
            .max_results(100)
            .next_token(next_token.as_ref().unwrap())
            .send()
            .await?;
    }

    // Create DataFrame
    let series: Vec<Series> = all_columns.iter()
        .zip(column_names.iter())
        .map(|(col, name)| Series::new(name, col))
        .collect();

    Ok(DataFrame::new(series)?)
} 