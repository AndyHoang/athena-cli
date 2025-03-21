# Athena CLI User Guide

## Installation

```bash
cargo install athena-cli
```

## Configuration

Create a configuration file at `~/.config/athena-cli/config.toml`:

```toml
# See config.example.toml for a complete example
[default]
region = "us-east-1"
workgroup = "primary"
database = "default"
output_location = "s3://aws-athena-query-results-123456789012-us-east-1/"
```

## Usage Examples

### Execute a Query

```bash
athena-cli query "SELECT * FROM my_table LIMIT 10"
```

### List Databases

```bash
athena-cli database list
```

### Describe a Table

```bash
athena-cli database describe my_table
```

### View Query History

```bash
athena-cli history list
```
