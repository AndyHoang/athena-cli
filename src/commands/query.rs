//! Query execution module for Athena CLI.
//!
//! This module provides functionality to:
//! - Execute SQL queries against AWS Athena
//! - Retrieve and display query results
//! - Monitor query execution status and statistics
//! - Handle result pagination and data formatting
//!
//! ## Usage Examples
//!
//! Simple query:
//!
//! ```bash
//! athena-cli query "SELECT * FROM my_table"
//! ```
//!
//! Query with database and workgroup specified:
//!
//! ```bash
//! athena-cli -d my_database -w my_workgroup query "SELECT * FROM my_table"
//! ```
//!
//! Query with custom result reuse time:
//!
//! ```bash
//! athena-cli query --reuse-time 2h "SELECT * FROM my_table"
//! ```
//!
//! Query with output location:
//!
//! ```bash
//! athena-cli --output-location s3://my-bucket/results/ query "SELECT * FROM my_table"
//! ```

use crate::cli;
use crate::context::Context;
use crate::validation;
use anyhow::Result;
use aws_sdk_athena::types::{
    QueryExecutionContext, QueryExecutionState, ResultConfiguration, ResultReuseByAgeConfiguration,
    ResultReuseConfiguration,
};
use aws_sdk_athena::Client;
use byte_unit::Byte;
use colored::Colorize;
use polars::prelude::*;
use std::{thread, time::Duration};

/// Executes an Athena SQL query and displays the results.
///
/// # Arguments
///
/// * `ctx` - The application context containing configuration and connection details
/// * `args` - Command line arguments including the SQL query text and reuse time
///
/// # Returns
///
/// Returns a Result indicating success or failure of the query execution
///
/// # Features
///
/// * Configurable query result reuse (caching) duration
/// * Displays query statistics including data scanned and cache status
/// * Supports pagination for large result sets
/// * Returns results as a Polars DataFrame for further processing
///
/// # Examples
///
/// Basic query example:
///
/// ```bash
/// athena-cli query "SELECT * FROM my_database.my_table LIMIT 10"
/// ```
///
/// Using result reuse/caching (30 minutes):
///
/// ```bash
/// athena-cli query --reuse-time 30m "SELECT count(*) FROM my_table"
/// ```
///
/// Query with specific database:
///
/// ```bash
/// athena-cli -d my_database query "SELECT * FROM my_table WHERE id=123"
/// ```
///
/// Query with custom workgroup and output location:
///
/// ```bash
/// athena-cli -w my_workgroup --output-location s3://my-bucket/results/ query "SELECT * FROM my_table"
/// ```
pub async fn execute(ctx: &Context, args: &cli::QueryArgs) -> Result<()> {
    println!("Executing query: {}", args.query);

    // Validate SQL syntax before sending to Athena
    if let Err(e) = validation::validate_query_syntax(&args.query) {
        println!("{}", "SQL syntax validation failed".red().bold());
        return Err(e);
    }

    let database = ctx
        .database()
        .ok_or_else(|| anyhow::anyhow!("Database name is required but was not provided"))?;

    let client = ctx.create_athena_client();

    let query_id = start_query(
        &client,
        &database,
        &args.query,
        &ctx.workgroup(),
        args.reuse_time,
        ctx.output_location()
            .as_deref()
            .unwrap_or("s3://aws-athena-query-results"),
    )
    .await?;

    println!("Query execution ID: {}", query_id);

    let df = get_query_results(&client, &query_id).await?;
    println!("Results DataFrame:");
    println!("{}", df);

    Ok(())
}

/// Starts an Athena query execution with the specified parameters and returns the execution ID.
///
/// # Arguments
///
/// * `client` - The AWS Athena SDK client
/// * `database` - The database to query against
/// * `query` - The SQL query string to execute
/// * `workgroup` - The Athena workgroup to use
/// * `reuse_duration` - Duration for which query results should be reused/cached
/// * `output_location` - S3 location where query results will be stored
///
/// # Returns
///
/// Returns a Result containing the query execution ID as a String
///
/// # Implementation Details
///
/// * Configures the query context with database and output location
/// * Sets up result reuse configuration based on the provided duration
/// * Returns the execution ID that can be used to track and retrieve results
async fn start_query(
    client: &Client,
    database: &str,
    query: &str,
    workgroup: &str,
    reuse_duration: Duration,
    output_location: &str,
) -> Result<String> {
    let context = QueryExecutionContext::builder().database(database).build();

    let config = ResultConfiguration::builder()
        .output_location(output_location)
        .build();

    let result = client
        .start_query_execution()
        .result_reuse_configuration(
            ResultReuseConfiguration::builder()
                .result_reuse_by_age_configuration(
                    ResultReuseByAgeConfiguration::builder()
                        .enabled(true)
                        .max_age_in_minutes(reuse_duration.as_secs() as i32 / 60)
                        .build(),
                )
                .build(),
        )
        .query_string(query)
        .query_execution_context(context)
        .result_configuration(config)
        .work_group(workgroup)
        .send()
        .await?;

    Ok(result.query_execution_id().unwrap_or_default().to_string())
}

/// Retrieves query results and converts them to a Polars DataFrame.
///
/// # Arguments
///
/// * `client` - The AWS Athena SDK client
/// * `query_execution_id` - The execution ID of the query whose results to retrieve
///
/// # Returns
///
/// Returns a Result containing a Polars DataFrame with the query results
///
/// # Behavior
///
/// * Polls the query execution until it succeeds, fails, or is cancelled
/// * Displays query statistics including data scanned and cache status
/// * Paginates through results if they span multiple pages (100 rows per page)
/// * Converts query results to a Polars DataFrame for analysis and display
///
/// # Error Handling
///
/// * Returns an error if the query fails or is cancelled
/// * Handles partial results and pagination automatically
async fn get_query_results(client: &Client, query_execution_id: &str) -> Result<DataFrame> {
    // Wait for query to complete
    loop {
        let status = client
            .get_query_execution()
            .query_execution_id(query_execution_id)
            .send()
            .await?;

        if let Some(execution) = status.query_execution() {
            match execution.status().unwrap().state().as_ref() {
                Some(QueryExecutionState::Succeeded) => {
                    // Print query info once before breaking
                    if let Some(result_config) = execution.result_configuration() {
                        if let Some(output_location) = result_config.output_location() {
                            println!("Results S3 path: {}", output_location);
                        }
                    }

                    if let Some(statistics) = execution.statistics() {
                        let data_scanned = statistics.data_scanned_in_bytes().unwrap_or(0);
                        let is_cached = data_scanned == 0;
                        println!(
                            "Query cache status: {}",
                            if is_cached {
                                String::from("Results retrieved from cache")
                            } else {
                                let formatted_size = Byte::from_i64(data_scanned)
                                    .map(|b| {
                                        b.get_appropriate_unit(byte_unit::UnitType::Decimal)
                                            .to_string()
                                    })
                                    .unwrap_or_else(|| "-".to_string());
                                format!("Fresh query execution (scanned {})", formatted_size)
                            }
                        );
                    }
                    break;
                }
                Some(QueryExecutionState::Failed) | Some(QueryExecutionState::Cancelled) => {
                    let error_message = if let Some(status) = execution.status() {
                        if let Some(reason) = status.state_change_reason() {
                            format!("Query failed: {}", reason)
                        } else {
                            "Query failed or was cancelled without specific reason".to_string()
                        }
                    } else {
                        "Query failed or was cancelled".to_string()
                    };
                    return Err(anyhow::anyhow!("{}", error_message.red().bold()));
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
        if let Some(first_row) = rs.rows().first() {
            column_names = first_row
                .data()
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
            println!(
                "Finished processing {} pages, total rows: {}",
                page_count,
                all_columns[0].len()
            );
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
    let series = all_columns
        .iter()
        .zip(column_names.iter())
        .map(|(col, name)| Series::new(name.into(), col))
        .map(|s| s.into_column())
        .collect();

    // Convert Series to Columns and create DataFrame
    Ok(DataFrame::new(series)?)
}
