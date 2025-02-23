# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please do NOT open an issue. 
Email security@your-domain.com instead.

## Security Best Practices

1. AWS Credentials
   - Never commit AWS credentials
   - Use environment variables or AWS credentials file
   - Use IAM roles with minimum required permissions

2. Configuration
   - Don't store sensitive data in config files
   - Use environment variables for sensitive values
   - Keep S3 bucket policies restrictive

3. Development
   - Run pre-commit hooks
   - Review code for credential leaks
   - Keep dependencies updated 