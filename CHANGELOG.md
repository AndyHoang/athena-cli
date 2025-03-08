# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2025-03-08
### Changed
- Updated build and CI workflows
- Improved GitHub Actions release process
- Fixed dependency audit issues

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