use aws_sdk_athena::Client;
use anyhow::Result;
use prettytable::{Table, row};
use crate::cli::HistoryArgs;
use crate::config;
use byte_unit::Byte;
use chrono::{DateTime, Utc};
use std::time::{Duration, UNIX_EPOCH};

pub async fn list(client: Client, args: HistoryArgs) -> Result<()> {
    let config = config::Config::load()?;
    
    let workgroup = args.workgroup
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

    let mut table = Table::new();
    table.add_row(row![
        "Execution ID",
        "Query",
        "Start Time",
        "Status",
        "Run Time",
        "Cache",
        "Data Scanned"
    ]);

    // Get details for all queries in a single batch request
    let query_ids = result.query_execution_ids();
    if !query_ids.is_empty() {
        let details = client
            .batch_get_query_execution()
            .set_query_execution_ids(Some(query_ids.to_vec()))
            .send()
            .await?;

        for execution in details.query_executions() {
            // Filter by status if specified
            if let Some(status_filter) = &args.status {
                if let Some(status) = execution.status().and_then(|s| s.state()) {
                    if status.as_str() != status_filter.to_uppercase() {
                        continue;
                    }
                }
            }

            let status = execution.status()
                .and_then(|s| s.state())
                .map(|s| s.as_str().to_string())
                .unwrap_or_else(|| "UNKNOWN".to_string());

            let start_time = execution.status()
                .and_then(|s| s.submission_date_time())
                .map(|dt| {
                    let secs = dt.as_secs_f64();
                    let system_time = UNIX_EPOCH + Duration::from_secs_f64(secs);
                    DateTime::<Utc>::from(system_time)
                })
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "-".to_string());

            let runtime = if let (Some(start), Some(end)) = (
                execution.status().and_then(|s| s.submission_date_time()),
                execution.status().and_then(|s| s.completion_date_time())
            ) {
                let duration = end.as_secs_f64() - start.as_secs_f64();
                humantime::format_duration(Duration::from_secs_f64(duration)).to_string()
            } else {
                "-".to_string()
            };

            let (cache_hit, data_scanned) = if let Some(stats) = execution.statistics() {
                let bytes = stats.data_scanned_in_bytes().unwrap_or(0);
                let is_cached = bytes == 0;
                (
                    if is_cached { "HIT" } else { "MISS" }.to_string(),
                    if is_cached {
                        "-".to_string()
                    } else {
                        Byte::from_bytes(bytes as u128)
                            .get_appropriate_unit(true)
                            .to_string()
                    }
                )
            } else {
                ("-".to_string(), "-".to_string())
            };

            let query_string = execution.query()
                .map(|q| if q.len() > 50 {
                    format!("{}...", &q[..47])
                } else {
                    q.to_string()
                })
                .unwrap_or_else(|| "-".to_string());

            let query_id = execution.query_execution_id().unwrap_or("-");
            table.add_row(row![
                query_id,
                query_string,
                start_time,
                status,
                runtime,
                cache_hit,
                data_scanned
            ]);
        }
    }

    table.printstd();
    Ok(())
} 