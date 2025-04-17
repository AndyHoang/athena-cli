# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2025-04-17
### Added
- SQL syntax validation before sending queries to Athena (ansi sql dialect)
- Improved error handling for SQL queries with syntax errors
- User-friendly error messages with suggestions for fixing common issues
- Colorized output for better readability of error messages
- Support for different error types (syntax, permissions, table not found)

## [0.3.0] - 2025-03-21
### Added
- Database schema exploration functionality
- Added `table describe` command to show detailed table structure
- Added table column type display and partitioning information

### Changed
- Restructured CLI commands to follow `object action` convention
- Changed `list-databases` to `database list`
- Changed `list-tables` to `table list`
- Changed `describe-table` to `table describe`
- Changed `list-workgroups` to `workgroup list`
- Updated documentation and README to reflect new command structure

## [0.2.2] - 2025-03-08
### Fixed
- Fixed issue with query command where output location was incorrectly handled

## [0.2.1] - 2025-03-08
### Changed
- Updated build and CI workflows
- Improved GitHub Actions release process
- Fixed dependency audit issues
- Added automated crates.io publishing to release workflow

## [0.2.0] - 2025-03-08
### Added
- Query inspection command to show detailed information about query executions
- Download command to fetch query results from S3
- Support for row count display in query inspection
- Configurable field display for query inspection through config file
- Quiet mode for minimal output in both inspect and download commands

## [0.1.0]
### Added
- Initial CLI implementation
- Query execution support
- Database listing
- Workgroup listing
- Configuration file support
- Query result caching
- Human-readable data size formatting
