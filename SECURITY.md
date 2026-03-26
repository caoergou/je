# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Jzen, please report it responsibly:

### How to Report

1. **Do not** open a public issue
2. Use GitHub's [Private Security Advisory](https://github.com/caoergou/jzen/security/advisories/new) feature
3. Alternatively, email the maintainer directly (if contact info is available)

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Timeline**: Depends on severity

## Security Considerations

### File Operations

Jzen performs file I/O operations. Be aware of:

- **Atomic writes**: Jzen writes to a temp file, then renames to prevent data corruption
- **No network operations**: Jzen does not make network requests
- **No code execution**: Jzen does not execute arbitrary code from JSON files

### Input Validation

- Jzen parses JSON input with both strict and lenient modes
- Malformed JSON is rejected with clear error messages
- Path expressions are validated before traversal

### AI Agent Usage

When using Jzen with AI agents:

- The `--json` flag produces structured output
- Paths are not evaluated as code
- Values are JSON-parsed, not shell-evaluated

## Best Practices

1. **Validate input files**: Use `jzen check` before processing untrusted JSON
2. **Use `--dry-run`**: Preview changes with `jzen fix --dry-run`
3. **Atomic writes**: Jzen's default behavior ensures crash safety
4. **Schema validation**: Use `jzen validate` to ensure data integrity

## Disclosure Policy

- Security fixes will be released as patch versions
- CVEs will be requested for significant vulnerabilities
- Credit will be given to reporters (unless they prefer to remain anonymous)

Thank you for helping keep Jzen secure!
