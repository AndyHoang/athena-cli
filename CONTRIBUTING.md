# Contributing to Athena CLI

## Development Process
1. Fork the repository
2. Create a feature branch
3. Install pre-commit hooks: `pre-commit install`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy`
7. Commit your changes
8. Push to your fork
9. Submit a Pull Request

## Code Style
- Follow Rust standard formatting (enforced by `rustfmt`)
- Use meaningful variable names
- Add comments for complex logic
- Write tests for new features

## Security
- Never commit credentials
- Use environment variables for sensitive data
- Run security checks before committing

## Pull Request Process
1. Update documentation
2. Add tests if needed
3. Update CHANGELOG.md
4. Wait for CI checks to pass
5. Get review approval 