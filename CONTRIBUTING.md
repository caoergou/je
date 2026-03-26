# Contributing to Jzen

Thank you for your interest in contributing to Jzen! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Development Guidelines](#development-guidelines)
- [Commit Message Convention](#commit-message-convention)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone.

## Development Setup

### Prerequisites

- **Rust 1.85+** (required for Rust 2024 edition)
- **cargo** (comes with Rust)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/caoergou/jzen.git
cd jzen

# Build the project
cargo build

# Run tests
cargo test

# Run the binary
cargo run -- --help

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check
```

### Project Structure

```
jzen/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli.rs           # CLI argument parsing
│   ├── command/         # Command handlers
│   ├── engine/          # Core JSON engine (pure, no I/O)
│   │   ├── parser.rs    # Strict & lenient parsing
│   │   ├── path.rs      # Path expressions
│   │   ├── edit.rs      # In-place modifications
│   │   ├── fix.rs       # Auto-repair
│   │   ├── format.rs    # Pretty-print/minify
│   │   ├── schema.rs    # Schema inference
│   │   └── value.rs     # JsonValue type
│   ├── tui/             # Terminal UI (ratatui)
│   ├── i18n.rs          # Internationalization
│   └── output.rs        # Output formatting
├── tests/               # Integration tests
└── skills/              # AI agent skill definitions
```

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/caoergou/jzen/issues)
2. If not, create a new issue using the Bug Report template
3. Include:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment (OS, Rust version)

### Suggesting Features

1. Check existing issues for similar suggestions
2. Create a new issue using the Feature Request template
3. Describe the use case and expected behavior

### Submitting Code

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add/update tests
5. Run `cargo test` and `cargo clippy`
6. Commit with a conventional commit message
7. Push and open a Pull Request

## Development Guidelines

### Code Style

- Run `cargo fmt` before committing
- Address all clippy warnings
- Use meaningful variable names
- Add documentation comments for public APIs

### Testing

- Add unit tests for new functionality
- Add integration tests for new commands
- Ensure all tests pass: `cargo test`

### Documentation

- Update README.md if adding new features
- Update CLI_SPEC.md for new commands
- Add inline comments for complex logic

## Commit Message Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(cli): add 'merge' command for combining JSON files
fix(parser): handle empty strings in lenient mode
docs(readme): add comparison table with jq
test(engine): add property-based tests for path expressions
```

## Pull Request Process

1. **Ensure CI passes**: All tests and clippy checks must pass
2. **Update documentation**: README, CLI_SPEC, etc.
3. **Add tests**: New features need tests
4. **One PR per feature**: Keep changes focused
5. **Descriptive title**: Use conventional commit format

### PR Checklist

- [ ] Code compiles without errors
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation updated
- [ ] Commit messages follow convention
- [ ] PR title is descriptive

## Questions?

Feel free to open an issue for any questions or discussions.

Thank you for contributing! 🎉
