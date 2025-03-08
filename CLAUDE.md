# Athena CLI Development Guide

## Build Commands
```bash
cargo build                  # Build the project
cargo run                    # Run the application
cargo test                   # Run all tests
cargo test -- test_name      # Run a specific test
cargo clippy                 # Run linter
cargo fmt                    # Format code
cargo publish                # Publish to crates.io
```

## GitHub Actions Configuration
- A GitHub secret named `CARGO_REGISTRY_TOKEN` is required for automated publishing to crates.io
- This can be generated at https://crates.io/me/settings/tokens and added to the repository secrets

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
- Always ask for confirmation before pushing to origin
- Use tags without the "v" prefix (e.g., "0.2.1" instead of "v0.2.1")
- Before pushing a version tag, ensure CHANGELOG.md is updated with the latest changes
- Examples:
  - "Update version to 0.2.0"
  - "Fix query parsing issue"
  - "Add download command support"
