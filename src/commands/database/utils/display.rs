use aws_sdk_athena::types::Column;
use prettytable::Cell;

/// Display struct for AWS Athena Column
pub struct ColumnDisplay {
    /// Column name
    name: String,
    /// Column type
    column_type: String,
    /// Column comment/description (may be empty for ColumnInfo)
    comment: String,
}

impl From<&Column> for ColumnDisplay {
    fn from(column: &Column) -> Self {
        Self {
            name: column.name().to_string(),
            column_type: column.r#type().unwrap_or("").to_string(),
            comment: column.comment().unwrap_or("").to_string(),
        }
    }
}

impl From<Column> for ColumnDisplay {
    fn from(column: Column) -> Self {
        Self::from(&column)
    }
}

impl ColumnDisplay {
    /// Convert the ColumnDisplay into a prettytable Row
    pub fn to_row(&self) -> prettytable::Row {
        prettytable::Row::new(vec![
            Cell::new(&self.name),
            Cell::new(&self.column_type),
            Cell::new(&self.comment),
        ])
    }

    /// Create a formatted table from a slice of Columns
    pub fn create_columns_table(columns: &[Column]) -> prettytable::Table {
        let mut table = prettytable::Table::new();

        // Add header row
        table.add_row(prettytable::Row::new(vec![
            Cell::new("Name"),
            Cell::new("Type"),
            Cell::new("Description"),
        ]));

        // Add data rows
        for column in columns {
            let display = ColumnDisplay::from(column);
            table.add_row(display.to_row());
        }

        table
    }
}

/// Helper struct for parameter display
pub struct ParameterDisplay {
    /// Parameter name
    name: String,
    /// Parameter value
    value: String,
}

impl ParameterDisplay {
    /// Convert the ParameterDisplay into a prettytable Row
    pub fn to_row(&self) -> prettytable::Row {
        prettytable::Row::new(vec![Cell::new(&self.name), Cell::new(&self.value)])
    }

    /// Create a formatted table from a map of parameters
    pub fn create_parameters_table(
        parameters: &std::collections::HashMap<String, String>,
        exclude_keys: &[&str],
    ) -> prettytable::Table {
        let mut table = prettytable::Table::new();

        // Add header row
        table.add_row(prettytable::Row::new(vec![
            Cell::new("Parameter"),
            Cell::new("Value"),
        ]));

        // Add data rows
        for (key, value) in parameters {
            // Skip excluded keys
            if exclude_keys.contains(&key.as_str()) {
                continue;
            }

            let display = Self {
                name: key.clone(),
                value: value.clone(),
            };
            table.add_row(display.to_row());
        }

        table
    }
}

/// Display struct for database information
pub struct DatabaseDisplay {
    /// Database name
    name: String,
    /// Database description
    description: String,
}

impl DatabaseDisplay {
    /// Create a new DatabaseDisplay from AWS SDK types
    pub fn from_database(db: &aws_sdk_athena::types::Database) -> Self {
        Self {
            name: db.name().to_string(),
            description: db.description().unwrap_or("").to_string(),
        }
    }

    /// Convert the DatabaseDisplay into a prettytable Row
    pub fn to_row(&self) -> prettytable::Row {
        prettytable::Row::new(vec![Cell::new(&self.name), Cell::new(&self.description)])
    }

    /// Create a formatted table from a slice of Databases
    pub fn create_databases_table(
        databases: &[aws_sdk_athena::types::Database],
    ) -> prettytable::Table {
        let mut table = prettytable::Table::new();

        // Add header row
        table.add_row(prettytable::Row::new(vec![
            Cell::new("Name"),
            Cell::new("Description"),
        ]));

        // Add data rows
        for db in databases {
            let display = Self::from_database(db);
            table.add_row(display.to_row());
        }

        table
    }
}

/// Display struct for table metadata
pub struct TableMetadataDisplay {
    /// Table name
    name: String,
    /// Table type
    table_type: String,
    /// Column count
    column_count: usize,
}

impl TableMetadataDisplay {
    /// Create a new TableMetadataDisplay from AWS SDK types
    pub fn from_table_metadata(table: &aws_sdk_athena::types::TableMetadata) -> Self {
        Self {
            name: table.name().to_string(),
            table_type: table.table_type().unwrap_or("").to_string(),
            column_count: table.columns().len(),
        }
    }

    /// Convert the TableMetadataDisplay into a prettytable Row
    pub fn to_row(&self) -> prettytable::Row {
        prettytable::Row::new(vec![
            Cell::new(&self.name),
            Cell::new(&self.table_type),
            Cell::new(&self.column_count.to_string()),
        ])
    }

    /// Create a formatted table from a slice of TableMetadata
    pub fn create_table_metadata_table(
        tables: &[&aws_sdk_athena::types::TableMetadata],
    ) -> prettytable::Table {
        let mut table = prettytable::Table::new();

        // Add header row
        table.add_row(prettytable::Row::new(vec![
            Cell::new("Name"),
            Cell::new("Type"),
            Cell::new("Columns"),
        ]));

        // Add data rows
        for table_meta in tables {
            let display = Self::from_table_metadata(table_meta);
            table.add_row(display.to_row());
        }

        table
    }
}
