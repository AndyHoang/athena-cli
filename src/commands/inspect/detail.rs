use super::download::download_from_s3;
use super::fields::{get_field_value, get_inspect_fields, InspectField};
use crate::cli::InspectArgs;
use crate::context::Context;
use anyhow::Result;
use aws_sdk_s3;
use owo_colors::OwoColorize;
use prettytable::{format, Cell, Row, Table};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tempfile::tempdir;

pub async fn detail(ctx: &Context, args: &InspectArgs) -> Result<()> {
    let client = ctx.create_athena_client();
    let query_id = args.query_id.clone();

    // Command-specific quiet overrides global setting
    let quiet_mode = args.quiet || ctx.quiet();

    // Get query execution details
    let result = client
        .get_query_execution()
        .query_execution_id(&query_id)
        .send()
        .await?;

    let execution = result
        .query_execution()
        .ok_or_else(|| anyhow::anyhow!("No query execution found with ID: {}", query_id))?;

    let mut total_rows: Option<usize> = None;
    let mut temp_dir = None; // To hold the TempDir if created

    // If query succeeded, calculate total rows
    if let Some(status) = execution.status() {
        if let Some(state) = status.state() {
            if state.as_str() == "SUCCEEDED" {
                if let Some(s3_output_location) = execution
                    .result_configuration()
                    .and_then(|c| c.output_location())
                {
                    let output_dir = match &args.output {
                        Some(dir) => dir.clone(),
                        None => {
                            let dir = tempdir()?;
                            let path = dir.path().to_str().unwrap().to_string();
                            temp_dir = Some(dir); // Keep TempDir in scope
                            path
                        }
                    };

                    let s3_client = aws_sdk_s3::Client::new(ctx.aws_config());
                    if let Ok(file_path) =
                        download_from_s3(&s3_client, s3_output_location, &output_dir, &query_id)
                            .await
                    {
                        if let Ok(file) = File::open(&file_path) {
                            let reader = BufReader::new(file);
                            // Subtract 1 for the header row
                            total_rows = Some(reader.lines().count().saturating_sub(1));
                        }
                        if args.output.is_none() {
                            let _ = fs::remove_file(file_path);
                        }
                    }
                }
            }
        }
    }

    if !quiet_mode {
        println!("\n{}", "Query Execution Details".bold());
        println!("ID: {}\n", query_id.bright_green());
        // Create a table for the query information
        let mut table = Table::new();

        // Configure table style
        table.set_format(*format::consts::FORMAT_CLEAN); // Clean borders

        // Get fields to display
        let fields = get_inspect_fields();

        // Add header
        table.add_row(Row::new(vec![
            Cell::new("Field").style_spec("Fb"), // Bold
            Cell::new("Value").style_spec("Fb"), // Bold
        ]));

        // Add rows for each field
        for field in fields {
            let value = if field == InspectField::TotalRows {
                total_rows
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".to_string())
            } else {
                get_field_value(execution, field)
            };

            let formatted_value = match field {
                InspectField::Status => match value.as_str() {
                    "SUCCEEDED" => value.bright_green().to_string(),
                    "FAILED" => value.bright_red().to_string(),
                    _ => value.yellow().to_string(),
                },
                InspectField::DataScanned => value.bright_cyan().to_string(),
                InspectField::TotalRows => value.bright_magenta().to_string(),
                _ => value,
            };

            table.add_row(Row::new(vec![
                Cell::new(&field.to_string()).style_spec("Fb"), // Bold field names
                Cell::new(&formatted_value),
            ]));
        }

        // Print the table
        table.printstd();
    }

    // Check if query was successful before trying to get results
    if let Some(status) = execution.status() {
        if let Some(state) = status.state() {
            if state.as_str() == "SUCCEEDED" {
                if !quiet_mode {
                    // Get sample of query results
                    let results = client
                        .get_query_results()
                        .query_execution_id(&query_id)
                        .max_results(11) // 10 rows + header
                        .send()
                        .await?;

                    if let Some(result_set) = results.result_set {
                        if let Some(ref rows) = result_set.rows {
                            if !rows.is_empty() {
                                println!("\n{}", "Query Result Sample (first 10 rows):".bold());
                                let mut sample_table = Table::new();
                                sample_table.set_format(*format::consts::FORMAT_BOX_CHARS);

                                // Header row
                                if let Some(metadata) = result_set.result_set_metadata() {
                                    if let Some(columns) = &metadata.column_info {
                                        let headers = columns
                                            .iter()
                                            .map(|c| Cell::new(&c.name).style_spec("Fb"))
                                            .collect::<Vec<Cell>>();
                                        sample_table.add_row(Row::new(headers));
                                    }
                                }

                                // Data rows
                                for row in rows.iter().skip(1) {
                                    if let Some(data) = &row.data {
                                        let data_cells = data
                                            .iter()
                                            .map(|d| {
                                                Cell::new(d.var_char_value.as_deref().unwrap_or(""))
                                            })
                                            .collect();
                                        sample_table.add_row(Row::new(data_cells));
                                    }
                                }
                                sample_table.printstd();
                            }
                        }
                    }
                }

                // If output option is provided, download results from S3
                if let Some(output_dir) = &args.output {
                    let s3_output_location = execution
                        .result_configuration()
                        .and_then(|c| c.output_location())
                        .ok_or_else(|| {
                            anyhow::anyhow!("No output location found for query: {}", query_id)
                        })?;

                    if !quiet_mode {
                        println!("\n{}", "S3 Output Location:".bold());
                        println!("📂 {}", s3_output_location.bright_blue());
                        println!("\n{}", "Downloading Results...".bold());
                    }

                    let s3_client = aws_sdk_s3::Client::new(ctx.aws_config());

                    match download_from_s3(&s3_client, s3_output_location, output_dir, &query_id)
                        .await
                    {
                        Ok(file_path) => {
                            if quiet_mode {
                                println!("{}", file_path.display());
                            } else {
                                println!(
                                    "✅ Downloaded to: {}",
                                    file_path.display().to_string().bright_green()
                                )
                            }
                        }
                        Err(e) => {
                            if quiet_mode {
                                return Err(e);
                            } else {
                                println!("❌ Error: {}", e.to_string().bright_red())
                            }
                        }
                    }
                }
            } else if !quiet_mode {
                println!("\n{}", "Cannot display results:".bold());
                println!("❌ Query status is {}", state.as_str().bright_red());
            }
        }
    }

    if !quiet_mode {
        println!(); // Add final newline
    }
    // temp_dir will be dropped here, and the temporary directory will be cleaned up.
    let _ = temp_dir;
    Ok(())
}