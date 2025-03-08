use std::fmt;
use std::str::FromStr;
use aws_sdk_athena::types::QueryExecution;
use crate::config;
use crate::commands::common::{OptionDisplayValue, OptionDurationFormat, OptionByteDisplay};

// Define all possible fields that can be displayed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HistoryField {
    Id,
    Status,
    Query,
    StartTime,
    EndTime,
    DataScanned,
    Runtime,
    OutputLocation,
    Cache,
    RowCount,
}

// Add FromStr implementation for parsing from config
impl FromStr for HistoryField {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Id" => Ok(HistoryField::Id),
            "Status" => Ok(HistoryField::Status),
            "Query" => Ok(HistoryField::Query),
            "StartTime" => Ok(HistoryField::StartTime),
            "EndTime" => Ok(HistoryField::EndTime),
            "DataScanned" => Ok(HistoryField::DataScanned),
            "Runtime" => Ok(HistoryField::Runtime),
            "OutputLocation" => Ok(HistoryField::OutputLocation),
            "Cache" => Ok(HistoryField::Cache),
            "RowCount" => Ok(HistoryField::RowCount),
            _ => Err(format!("Unknown history field: {}", s)),
        }
    }
}

impl fmt::Display for HistoryField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryField::Id => write!(f, "ID"),
            HistoryField::Status => write!(f, "Status"),
            HistoryField::Query => write!(f, "Query"),
            HistoryField::StartTime => write!(f, "Start Time"),
            HistoryField::EndTime => write!(f, "End Time"),
            HistoryField::DataScanned => write!(f, "Data Scanned"),
            HistoryField::Runtime => write!(f, "Runtime"),
            HistoryField::OutputLocation => write!(f, "Output Location"),
            HistoryField::Cache => write!(f, "Cache"),
            HistoryField::RowCount => write!(f, "Row Count"),
        }
    }
}

// Default set of fields to display
pub fn default_history_fields() -> Vec<HistoryField> {
    vec![
        HistoryField::Id,
        HistoryField::Status,
        HistoryField::Query,
        HistoryField::StartTime,
        HistoryField::EndTime,
        HistoryField::DataScanned,
        HistoryField::Runtime,
        HistoryField::RowCount,
        HistoryField::OutputLocation,
        HistoryField::Cache,
    ]
}

// Get fields from config or use defaults
pub fn get_history_fields() -> Vec<HistoryField> {
    if let Ok(config) = config::Config::load() {
        if let Some(field_names) = config.app.history_fields {
            let fields: Vec<HistoryField> = field_names.iter()
                .filter_map(|name| HistoryField::from_str(name).ok())
                .collect();
            
            if !fields.is_empty() {
                return fields;
            }
        }
    }
    
    // Fall back to defaults if config loading fails or fields are empty
    default_history_fields()
}

// Format status for display
pub fn format_status(status: &Option<&aws_sdk_athena::types::QueryExecutionStatus>) -> String {
    status.and_then(|s| s.state())
        .map(|s| s.as_str())
        .to_display_value_or_default()
}

// Extract a field value from a query execution
pub fn get_field_value(execution: &QueryExecution, field: HistoryField) -> String {
    match field {
        HistoryField::Id => execution.query_execution_id().to_display_value_or_default(),
        
        HistoryField::Status => format_status(&execution.status()),
        
        HistoryField::Query => execution.query()
            .map(|q| if q.len() > 30 {
                format!("{}...", &q[..27])
            } else {
                q.to_string()
            })
            .to_display_value_or_default(),
            
        HistoryField::StartTime => execution.status()
            .and_then(|s| s.submission_date_time())
            .to_display_value_or_default(),
            
        HistoryField::EndTime => execution.status()
            .and_then(|s| s.completion_date_time())
            .to_display_value_or_default(),
            
        HistoryField::DataScanned => execution.statistics()
            .and_then(|s| s.data_scanned_in_bytes())
            .format_bytes_or_default(),
            
        HistoryField::Runtime => execution.statistics()
            .and_then(|s| s.engine_execution_time_in_millis())
            .format_duration_ms_or_default(),
            
        HistoryField::OutputLocation => execution.result_configuration()
            .and_then(|c| c.output_location())
            .to_display_value_or_default(),
            
        HistoryField::Cache => {
            let data_scanned = execution.statistics()
                .and_then(|s| s.data_scanned_in_bytes())
                .unwrap_or(1);
            
            if data_scanned == 0 {
                "Used cache".to_string()
            } else {
                "-".to_string()
            }
        },
        
        HistoryField::RowCount => {
            // The row count is not directly available in QueryExecution
            // We'll need to fetch it separately using get_query_runtime_statistics
            // For now, return a placeholder
            "-".to_string()
        },
    }
} 
