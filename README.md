# AWS Athena CLI

A command-line interface tool written in Rust for interacting with AWS Athena, providing a streamlined experience similar to the Athena web console.

## Purpose

- Execute SQL queries against AWS Athena databases
- List and manage workgroups
- View query history and results
- Manage databases and tables
- Save frequently used queries

## Installation

1. Ensure you have Rust installed (1.70 or later)
2. Clone this repository
3. Run `cargo install --path .`
4. Copy `config.example.toml` to `~/.config/athena-cli/config.toml` and update with your settings

## Usage

### Basic commands:
- `athena-cli query "SELECT * FROM table"` - Execute a query
- `athena-cli workgroup list` - List available workgroups
- `athena-cli db list` - List databases
- `athena-cli history` - Show recent queries
- `athena-cli inspect <query-id>` - Show detailed information about a query
- `athena-cli inspect <query-id> -o <path>` - Inspect query and download results
- `athena-cli download <query-id>` - Download query results (shortcut for inspect with download)

### Configuration
For configuration, edit `~/.config/aws-athena-cli/config.toml` to set:
- Default workgroup
- Output location
- AWS credentials (if not using AWS CLI configuration)

## Requirements

- AWS account with Athena access
- Configured AWS credentials
- Rust 1.70+

For detailed documentation and examples, see the [Wiki](link-to-wiki).

## Milestones

### Completed ✅
- [x] Authorize athena with SSO profile and ENV variables
- [x] Basic query execution with AWS Athena
- [x] Configuration management with TOML config file
- [x] Query result caching and reuse
- [x] Polars integration for DataFrame handling
- [x] Query history tracking
- [x] Get detail a history query
- [x] Export results to various formats

### Coming Soon 🚀
- [ ] Database schema exploration
- [ ] Custom output formatting

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Clone the repository
2. Install pre-commit hooks to ensure code quality:
   ```bash
   pip install pre-commit
   pre-commit install
   ```

The pre-commit hooks will automatically:
- Format your code with `cargo fmt`
- Run `cargo check` to ensure compilation
- Fix common issues with `cargo clippy --fix`
- Check for other issues like trailing whitespace, YAML/TOML format, etc.
