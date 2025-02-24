use clap::{Parser, Subcommand};
use std::time::Duration;
use humantime::parse_duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a query
    Query(QueryArgs),
    /// List available databases
    ListDatabases(DatabaseArgs),
    /// List workgroups
    ListWorkgroups(WorkgroupArgs),
}

impl Commands {
    pub fn get_profile(&self) -> Option<String> {
        match self {
            Commands::Query(args) => args.profile.clone(),
            Commands::ListDatabases(_) => None,
            Commands::ListWorkgroups(_) => None,
        }
    }
}

#[derive(Parser)]
pub struct QueryArgs {
    /// SQL query to execute
    pub query: String,
    /// Database name
    #[arg(short, long)]
    pub database: Option<String>,
    /// Workgroup name
    #[arg(short, long)]
    pub workgroup: Option<String>,
    /// AWS Profile name
    #[arg(short, long)]
    pub profile: Option<String>,
    /// Query reuse time (e.g., "10m", "2h", "1h30m")
    #[arg(short = 'r', long, value_parser = parse_duration, default_value = "60m")]
    pub reuse_time: Duration,
    /// S3 output location (overrides config)
    #[arg(long)]
    pub output_location: Option<String>,
}

#[derive(Parser)]
pub struct DatabaseArgs {
    /// Catalog name
    #[arg(short, long, default_value = "AwsDataCatalog")]
    pub catalog: String,
}

#[derive(Parser)]
pub struct WorkgroupArgs {
    /// Maximum number of workgroups to list
    #[arg(short, long, default_value = "50")]
    pub limit: i32,
} 