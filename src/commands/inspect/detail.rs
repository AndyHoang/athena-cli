use super::download::download_from_s3;
use super::fields::{get_field_value, get_inspect_fields};
use crate::cli::InspectArgs;
use crate::context::Context;
use anyhow::Result;
use aws_sdk_s3;
use owo_colors::OwoColorize;
use prettytable::{format, Cell, Row, Table};

pub async fn detail(ctx: &Context, args: &InspectArgs) -> Result<()> {
    let client = ctx.create_athena_client();
    let query_id = args.query_id.clone();

    // Command-specific quiet overrides global setting
    let quiet_mode = args.quiet || ctx.quiet();

    if !quiet_mode {
        println!("\n{}", "Query Execution Details".bold());
        println!("ID: {}\n", query_id.bright_green());
    }

    // Get query execution details
    let result = client
        .get_query_execution()
        .query_execution_id(&query_id)
        .send()
        .await?;

    let execution = result
        .query_execution()
        .ok_or_else(|| anyhow::anyhow!("No query execution found with ID: {}", query_id))?;

    if !quiet_mode {
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
    Ok(())
}
