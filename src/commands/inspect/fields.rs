use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use aws_sdk_athena::types::QueryExecution;
use crate::config;
use byte_unit::Byte;

// Define all possible fields that can be displayed in the inspect command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InspectField {
    Id,
    Status,
    StatusReason,
    Query,
    SubmissionTime,
    CompletionTime,
    Database,
    Catalog,
    Workgroup,
    DataScanned,
    CacheStatus,
    EngineExecutionTime,
    TotalExecutionTime,
    QueryPlanningTime,
    QueryQueueTime,
    ServiceProcessingTime,
    OutputLocation,
}

// Add FromStr implementation for parsing from config
impl FromStr for InspectField {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Id" => Ok(InspectField::Id),
            "Status" => Ok(InspectField::Status),
            "StatusReason" => Ok(InspectField::StatusReason),
            "Query" => Ok(InspectField::Query),
            "SubmissionTime" => Ok(InspectField::SubmissionTime),
            "CompletionTime" => Ok(InspectField::CompletionTime),
            "Database" => Ok(InspectField::Database),
            "Catalog" => Ok(InspectField::Catalog),
            "Workgroup" => Ok(InspectField::Workgroup),
            "DataScanned" => Ok(InspectField::DataScanned),
            "CacheStatus" => Ok(InspectField::CacheStatus),
            "EngineExecutionTime" => Ok(InspectField::EngineExecutionTime),
            "TotalExecutionTime" => Ok(InspectField::TotalExecutionTime),
            "QueryPlanningTime" => Ok(InspectField::QueryPlanningTime),
            "QueryQueueTime" => Ok(InspectField::QueryQueueTime),
            "ServiceProcessingTime" => Ok(InspectField::ServiceProcessingTime),
            "OutputLocation" => Ok(InspectField::OutputLocation),
            _ => Err(format!("Unknown inspect field: {}", s)),
        }
    }
}

impl fmt::Display for InspectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InspectField::Id => write!(f, "Query ID"),
            InspectField::Status => write!(f, "Status"),
            InspectField::StatusReason => write!(f, "Status Reason"),
            InspectField::Query => write!(f, "Query"),
            InspectField::SubmissionTime => write!(f, "Submission Time"),
            InspectField::CompletionTime => write!(f, "Completion Time"),
            InspectField::Database => write!(f, "Database"),
            InspectField::Catalog => write!(f, "Catalog"),
            InspectField::Workgroup => write!(f, "Workgroup"),
            InspectField::DataScanned => write!(f, "Data Scanned"),
            InspectField::CacheStatus => write!(f, "Cache Status"),
            InspectField::EngineExecutionTime => write!(f, "Engine Execution Time"),
            InspectField::TotalExecutionTime => write!(f, "Total Execution Time"),
            InspectField::QueryPlanningTime => write!(f, "Query Planning Time"),
            InspectField::QueryQueueTime => write!(f, "Query Queue Time"),
            InspectField::ServiceProcessingTime => write!(f, "Service Processing Time"),
            InspectField::OutputLocation => write!(f, "Output Location"),
        }
    }
}

// Default set of fields to display
pub fn default_inspect_fields() -> Vec<InspectField> {
    vec![
        InspectField::Id,
        InspectField::Status,
        InspectField::StatusReason,
        InspectField::Query,
        InspectField::SubmissionTime,
        InspectField::CompletionTime,
        InspectField::Database,
        InspectField::Catalog,
        InspectField::Workgroup,
        InspectField::DataScanned,
        InspectField::CacheStatus,
        InspectField::EngineExecutionTime,
        InspectField::TotalExecutionTime,
        InspectField::QueryPlanningTime,
        InspectField::QueryQueueTime,
        InspectField::ServiceProcessingTime,
        InspectField::OutputLocation,
    ]
}

// Get fields from config or use defaults
pub fn get_inspect_fields() -> Vec<InspectField> {
    if let Ok(config) = config::Config::load() {
        if let Some(field_names) = config.app.inspect_fields {
            let fields: Vec<InspectField> = field_names.iter()
                .filter_map(|name| InspectField::from_str(name).ok())
                .collect();
            
            if !fields.is_empty() {
                return fields;
            }
        }
    }
    
    // Fall back to defaults if config loading fails or fields are empty
    default_inspect_fields()
}

// Extract a field value from a query execution
pub fn get_field_value(execution: &QueryExecution, field: InspectField) -> String {
    match field {
        InspectField::Id => execution.query_execution_id().unwrap_or("-").to_string(),
        
        InspectField::Status => execution.status()
            .and_then(|s| s.state())
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "-".to_string()),
        
        InspectField::StatusReason => execution.status()
            .and_then(|s| s.state_change_reason())
            .unwrap_or("-").to_string(),
        
        InspectField::Query => execution.query().unwrap_or("-").to_string(),
            
        InspectField::SubmissionTime => execution.status()
            .and_then(|s| s.submission_date_time())
            .map(|t| format!("{}", t))
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::CompletionTime => execution.status()
            .and_then(|s| s.completion_date_time())
            .map(|t| format!("{}", t))
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::Database => execution.query_execution_context()
            .and_then(|c| c.database())
            .unwrap_or("-").to_string(),
            
        InspectField::Catalog => execution.query_execution_context()
            .and_then(|c| c.catalog())
            .unwrap_or("-").to_string(),
            
        InspectField::Workgroup => execution.work_group().unwrap_or("-").to_string(),
            
        InspectField::DataScanned => execution.statistics()
            .and_then(|s| s.data_scanned_in_bytes())
            .map(|b| Byte::from_i64(b as i64)
                .map(|b| b.get_appropriate_unit(byte_unit::UnitType::Decimal).to_string())
                .unwrap_or_else(|| "-".to_string()))
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::CacheStatus => {
            // Check if data scanned is 0 (indicating cache was used)
            let data_scanned = execution.statistics()
                .and_then(|s| s.data_scanned_in_bytes())
                .unwrap_or(1); // Default to non-zero if not available
            
            if data_scanned == 0 {
                "Used cache".to_string()
            } else {
                "Fresh execution".to_string()
            }
        },
            
        InspectField::EngineExecutionTime => execution.statistics()
            .and_then(|s| s.engine_execution_time_in_millis())
            .map(|ms| {
                let duration = Duration::from_millis(ms as u64);
                humantime::format_duration(duration).to_string()
            })
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::TotalExecutionTime => execution.statistics()
            .and_then(|s| s.total_execution_time_in_millis())
            .map(|ms| {
                let duration = Duration::from_millis(ms as u64);
                humantime::format_duration(duration).to_string()
            })
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::QueryPlanningTime => execution.statistics()
            .and_then(|s| s.query_planning_time_in_millis())
            .map(|ms| {
                let duration = Duration::from_millis(ms as u64);
                humantime::format_duration(duration).to_string()
            })
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::QueryQueueTime => execution.statistics()
            .and_then(|s| s.query_queue_time_in_millis())
            .map(|ms| {
                let duration = Duration::from_millis(ms as u64);
                humantime::format_duration(duration).to_string()
            })
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::ServiceProcessingTime => execution.statistics()
            .and_then(|s| s.service_processing_time_in_millis())
            .map(|ms| {
                let duration = Duration::from_millis(ms as u64);
                humantime::format_duration(duration).to_string()
            })
            .unwrap_or_else(|| "-".to_string()),
            
        InspectField::OutputLocation => execution.result_configuration()
            .and_then(|c| c.output_location())
            .unwrap_or("-")
            .to_string(),
    }
} 