use clap::{Parser, Subcommand, Args};
use std::time::Duration;
use humantime::parse_duration;

// Shared AWS arguments used by multiple commands
#[derive(Args, Clone, Default)]
pub struct AwsArgs {
    /// AWS Profile name
    #[arg(short, long, global = true)]
    pub profile: Option<String>,
    
    /// Workgroup name
    #[arg(short, long, global = true)]
    pub workgroup: Option<String>,
    
    /// Database name
    #[arg(short, long, global = true)]
    pub database: Option<String>,
    
    /// Catalog name
    #[arg(long, global = true, default_value = "AwsDataCatalog")]
    pub catalog: Option<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[command(flatten)]
    pub aws: AwsArgs,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a query
    Query(QueryArgs),
    
    /// List available databases
    ListDatabases(DatabaseArgs),
    
    /// List workgroups
    ListWorkgroups(WorkgroupArgs),
    
    /// Show query history
    History(HistoryArgs),

    /// Inspect details of a specific query
    Inspect(InspectArgs),
}

#[derive(Args, Clone)]
pub struct QueryArgs {
    /// SQL query to execute
    pub query: String,
    
    /// Query reuse time (e.g., "10m", "2h", "1h30m")
    #[arg(short = 'r', long, value_parser = parse_duration, default_value = "60m")]
    pub reuse_time: Duration,
    
    /// S3 output location (overrides config)
    #[arg(long)]
    pub output_location: Option<String>,
}

#[derive(Args, Clone)]
pub struct DatabaseArgs {
    // Empty - will use global catalog from AwsArgs
}

#[derive(Args, Clone)]
pub struct WorkgroupArgs {
    /// Maximum number of workgroups to list
    #[arg(short, long, default_value = "50")]
    pub limit: i32,
}

#[derive(Args, Clone)]
pub struct HistoryArgs {
    /// Maximum number of history items to show (overrides config)
    #[arg(short, long)]
    pub limit: Option<i32>,
    
    /// Show only queries with specific status (SUCCEEDED, FAILED, CANCELLED)
    #[arg(short, long)]
    pub status: Option<String>,
}

#[derive(Args, Clone)]
pub struct InspectArgs {
    /// Query execution ID to inspect
    pub query_id: String,
    
    /// Output directory for query results (e.g., "." for current directory)
    #[arg(short, long)]
    pub output: Option<String>,
} 