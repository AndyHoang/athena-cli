use clap::{Args, Parser, Subcommand};
use humantime::parse_duration;
use std::time::Duration;

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
    #[arg(long, global = true)]
    pub catalog: Option<String>,

    /// AWS Region
    #[arg(long, global = true)]
    pub region: Option<String>,

    /// S3 output location (for query results)
    #[arg(long, global = true)]
    pub output_location: Option<String>,
}

// Global display settings
#[derive(Args, Clone, Default)]
pub struct DisplayArgs {
    /// Suppress detailed output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

// Shared arguments for commands that support file output
#[derive(Args, Clone)]
pub struct OutputArgs {
    /// Output directory for results
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub aws: AwsArgs,

    #[command(flatten)]
    pub display: DisplayArgs,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute a query
    Query(QueryArgs),

    /// List available databases
    ListDatabases(DatabaseArgs),

    /// List tables in a database
    ListTables(TableArgs),

    /// Describe table structure with columns and partitions
    DescribeTable(DescribeTableArgs),

    /// List workgroups
    ListWorkgroups(WorkgroupArgs),

    /// Show query history
    History(HistoryArgs),

    /// Inspect details of a specific query
    Inspect(InspectArgs),

    /// Download query results (shortcut for 'inspect -o')
    #[command(alias = "dl")] // Optional: add even shorter alias
    Download(DownloadArgs),
}

#[derive(Args, Clone)]
pub struct QueryArgs {
    #[command(flatten)]
    pub aws: AwsArgs,

    /// SQL query to execute
    pub query: String,

    /// Query reuse time (e.g., "10m", "2h", "1h30m")
    #[arg(short = 'r', long, value_parser = parse_duration, default_value = "60m")]
    pub reuse_time: Duration,
}

#[derive(Args, Clone)]
pub struct DatabaseArgs {
    // Empty - will use global catalog from AwsArgs
    #[command(flatten)]
    pub aws: AwsArgs,
}

#[derive(Args, Clone)]
pub struct TableArgs {
    /// Database name (overrides global settings)
    #[arg(short = 'n', long)]
    pub db: Option<String>,

    /// Filter table names by pattern (e.g. "pp_*" for tables starting with pp_)
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Maximum number of tables to list
    #[arg(short, long, default_value = "50")]
    pub limit: i32,
}

#[derive(Args, Clone)]
pub struct DescribeTableArgs {
    /// Table identifier (can be 'database.table' or just 'table')
    pub table: String,

    /// Database name (alternative to using 'database.table' format)
    #[arg(short = 'n', long)]
    pub db: Option<String>,
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

// For commands that support output
#[derive(Args, Clone)]
pub struct InspectArgs {
    /// Query execution ID to inspect
    pub query_id: String,

    /// Output directory for query results (e.g., "." for current directory)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Quiet mode - only output the downloaded file path
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Args, Clone)]
pub struct DownloadArgs {
    /// Query execution ID
    pub query_id: String,

    /// Output directory for results
    #[arg(short, long, default_value = ".")]
    pub output: Option<String>,
}
