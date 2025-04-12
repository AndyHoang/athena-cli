//! SQL validation module for Athena CLI.
//!
//! This module provides functionality to validate SQL queries before sending them to AWS Athena.
//! It uses the sqlparser crate to parse and validate SQL syntax according to ANSI SQL standards,
//! which is compatible with Athena's SQL dialect.
//!
//! ## Features
//!
//! - SQL syntax validation using ANSI SQL standards
//! - Detailed error messages for syntax issues
//! - Validation before query execution to save time and costs

use anyhow::{Context, Result};
use sqlparser::ast::{Query, SetExpr, Statement};
use sqlparser::dialect::AnsiDialect;
use sqlparser::parser::Parser;

/// Validates the syntax of an Athena SQL query.
///
/// This function uses the sqlparser crate to parse the query using ANSI SQL standards,
/// which is compatible with Athena's SQL syntax. It returns a Result that indicates
/// whether the query syntax is valid.
///
/// # Arguments
///
/// * `query` - The SQL query string to validate
///
/// # Returns
///
/// * `Ok(())` if the query syntax is valid
/// * `Err(anyhow::Error)` with a descriptive error message if the syntax is invalid
///
/// # Examples
///
/// ```
/// use athena_cli::validation::validate_query_syntax;
///
/// // Valid query
/// let valid_query = "SELECT * FROM my_table WHERE id = 1";
/// assert!(validate_query_syntax(valid_query).is_ok());
///
/// // Invalid query with trailing comma
/// let invalid_query = "SELECT id, FROM my_table";
/// assert!(validate_query_syntax(invalid_query).is_err());
/// ```
pub fn validate_query_syntax(query: &str) -> Result<()> {
    // Use ANSI SQL dialect for standards-compliant parsing
    let dialect = AnsiDialect {};

    // Attempt to parse the SQL query
    match Parser::parse_sql(&dialect, query) {
        Ok(statements) => {
            // Additional validation for SELECT statements
            for stmt in statements {
                if let Statement::Query(query_box) = stmt {
                    validate_select_query(*query_box)?;
                }
            }
            Ok(())
        }
        Err(e) => {
            // Return a user-friendly error message
            Err(anyhow::anyhow!("SQL syntax error: {}", e))
                .with_context(|| format!("Failed to parse query: {}", query))
        }
    }
}

/// Validates a SELECT query for common issues that might not be caught by the parser.
fn validate_select_query(query: Query) -> Result<()> {
    // Check if it's a simple SELECT query
    if let SetExpr::Select(select) = *query.body {
        // Check if FROM clause is missing when it should be present
        if select.from.is_empty() {
            // Check if it's a special case that doesn't require FROM
            let is_special_case = select.projection.iter().any(|proj| {
                // Check for special cases like SELECT 1, SELECT CURRENT_DATE, etc.
                match proj {
                    sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                        matches!(expr, sqlparser::ast::Expr::Value(_))
                            || matches!(expr, sqlparser::ast::Expr::Function(_))
                    }
                    _ => false,
                }
            });

            if !is_special_case {
                return Err(anyhow::anyhow!(
                    "SQL syntax error: SELECT query missing FROM clause"
                ));
            }
        }
    }

    Ok(())
}

/// Checks if the query is a DDL (Data Definition Language) statement.
///
/// DDL statements include CREATE, ALTER, DROP, etc. This function is useful
/// for determining if a query will modify the database schema.
///
/// # Arguments
///
/// * `query` - The SQL query string to check
///
/// # Returns
///
/// * `true` if the query is a DDL statement
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use athena_cli::validation::is_ddl_statement;
///
/// assert!(is_ddl_statement("CREATE TABLE my_table (id INT)"));
/// assert!(is_ddl_statement("DROP TABLE my_table"));
/// assert!(!is_ddl_statement("SELECT * FROM my_table"));
/// ```
/// #[allow(dead_code)]
//pub fn is_ddl_statement(query: &str) -> bool {
//let query_upper = query.trim().to_uppercase();

//query_upper.starts_with("CREATE ") ||
//query_upper.starts_with("ALTER ") ||
//query_upper.starts_with("DROP ") ||
//query_upper.starts_with("TRUNCATE ") ||
//query_upper.starts_with("RENAME ")
//}

/// Checks if the query is a DML (Data Manipulation Language) statement.
///
/// DML statements include SELECT, INSERT, UPDATE, DELETE, etc. This function is useful
/// for determining if a query will modify data in the database.
///
/// # Arguments
///
/// * `query` - The SQL query string to check
///
/// # Returns
///
/// * `true` if the query is a DML statement
/// * `false` otherwise
///
/// # Examples
///
/// ```
/// use athena_cli::validation::is_dml_statement;
///
/// assert!(is_dml_statement("SELECT * FROM my_table"));
/// assert!(is_dml_statement("INSERT INTO my_table VALUES (1, 'test')"));
/// assert!(!is_dml_statement("CREATE TABLE my_table (id INT)"));
/// ```
/// #[allow(dead_code)]
//pub fn is_dml_statement(query: &str) -> bool {
//let query_upper = query.trim().to_uppercase();

//query_upper.starts_with("SELECT ") ||
//query_upper.starts_with("INSERT ") ||
//query_upper.starts_with("UPDATE ") ||
//query_upper.starts_with("DELETE ") ||
//query_upper.starts_with("MERGE ")
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_syntax() {
        let valid_queries = vec![
            "SELECT * FROM my_table",
            "SELECT id, name FROM my_table WHERE id > 10",
            "SELECT COUNT(*) FROM my_table GROUP BY category",
            "CREATE TABLE my_table (id INT, name STRING)",
            "DROP TABLE my_table",
            "INSERT INTO my_table VALUES (1, 'test')",
            "WITH t AS (SELECT * FROM my_table) SELECT * FROM t",
            "SELECT 1",            // Simple constant select
            "SELECT CURRENT_DATE", // Date function
            "SELECT NOW()",        // Current timestamp function
        ];

        for query in valid_queries {
            assert!(
                validate_query_syntax(query).is_ok(),
                "Query should be valid: {}",
                query
            );
        }
    }

    #[test]
    fn test_invalid_syntax() {
        let invalid_queries = vec![
            "SELECT * FORM my_table",                     // Misspelled FROM
            "SELECT id, FROM my_table",                   // Extra comma
            "SELECT * FROM my_table WHERE",               // Incomplete WHERE clause
            "CREATE TABLE my_table (id INT, name STRING", // Missing closing parenthesis
            "DROP TABLE",                                 // Missing table name
            "SELECT * WHERE id = 1",                      // Missing FROM clause
        ];

        for query in invalid_queries {
            assert!(
                validate_query_syntax(query).is_err(),
                "Query should be invalid: {}",
                query
            );
        }
    }

    //#[test]
    //fn test_ddl_detection() {
    //assert!(is_ddl_statement("CREATE TABLE my_table (id INT)"));
    //assert!(is_ddl_statement("DROP TABLE my_table"));
    //assert!(is_ddl_statement("ALTER TABLE my_table ADD COLUMN new_col INT"));
    //assert!(is_ddl_statement("TRUNCATE TABLE my_table"));
    //assert!(is_ddl_statement("RENAME TABLE old_table TO new_table"));

    //assert!(!is_ddl_statement("SELECT * FROM my_table"));
    //assert!(!is_ddl_statement("INSERT INTO my_table VALUES (1)"));
    //}

    //#[test]
    //fn test_dml_detection() {
    //assert!(is_dml_statement("SELECT * FROM my_table"));
    //assert!(is_dml_statement("INSERT INTO my_table VALUES (1)"));
    //assert!(is_dml_statement("UPDATE my_table SET col = 1"));
    //assert!(is_dml_statement("DELETE FROM my_table WHERE id = 1"));

    //assert!(!is_dml_statement("CREATE TABLE my_table (id INT)"));
    //assert!(!is_dml_statement("DROP TABLE my_table"));
    //}
}
