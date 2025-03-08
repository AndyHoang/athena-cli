# Athena CLI Development Guide

## Build Commands
```bash
cargo build                  # Build the project
cargo run                    # Run the application
cargo test                   # Run all tests
cargo test -- test_name      # Run a specific test
cargo clippy                 # Run linter
cargo fmt                    # Format code
```

## Code Style Guidelines
- **Imports**: Group by stdlib, external crates, internal modules
- **Error Handling**: Use `anyhow::Result` with context and `?` operator
- **Types**: Prefer structs with named fields, implement `Default` where appropriate
- **Naming**: Use `snake_case` for variables/functions, `CamelCase` for types
- **Organization**: 
  - Commands in `commands/` module
  - Each command function takes client and args, returns `Result<()>`
- **Documentation**: Document public APIs and complex logic
- **Tests**: Write tests for new functionality

## Project Structure
- `src/main.rs` - Entry point
- `src/cli.rs` - CLI definition with clap
- `src/config.rs` - Configuration handling
- `src/commands/` - Command implementations
- `src/athena/` - AWS Athena client wrapper

## Commit Guidelines
- Use simple, descriptive commit messages
- Do not include "Generated with Claude Code" or co-author tags
- Format: "<action> <component>: <brief description>"
- Always ask which branch to commit and push to before making changes
- Examples:
  - "Update version to 0.2.0"
  - "Fix query parsing issue"
  - "Add download command support"