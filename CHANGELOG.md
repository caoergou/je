# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-03-20

### Added

- **Dual Interface**: TUI for interactive editing, CLI for scripting and AI agents
- **AI-Agent Friendly**: Structured JSON output (`--json` flag), token-efficient commands
- **Auto-Repair**: Fix common JSON errors (trailing commas, single quotes, unquoted keys, comments)
- **Atomic Writes**: Crash-safe file operations with fsync
- **Path Navigation**: jq-inspired syntax (`.key`, `.arr[0]`, `.arr[-1]`)
- **Format Conversion**: JSON ↔ YAML ↔ TOML
- **Schema Inference**: Generate type summaries without values
- **Shell Completions**: Bash, Zsh, Fish, PowerShell, Elvish
- **Cross-Platform**: Linux, macOS, Windows
- **JSON Patch**: Batch operations via RFC 6902
- **TUI Features**:
  - Visual tree navigation
  - Inline editing
  - Undo/Redo support
  - Search functionality
  - Large file optimization
  - Watch mode

### CLI Commands

| Command | Description |
|---------|-------------|
| `get` | Read value at path |
| `set` | Set value (creates if missing) |
| `del` | Delete key/array element |
| `add` | Append to array |
| `patch` | Batch operations (RFC 6902) |
| `keys` | List keys at path |
| `len` | Array length / object key count |
| `type` | Get value type |
| `exists` | Check path exists |
| `schema` | Generate type summary |
| `tree` | Visual tree display |
| `check` | Validate JSON syntax |
| `fix` | Auto-repair JSON errors |
| `fmt` | Pretty-print |
| `minify` | Remove whitespace |
| `diff` | Compare two JSON files |
| `convert` | Convert to YAML/TOML |
| `validate` | Validate against JSON Schema |

[0.1.0]: https://github.com/caoergou/jzen/releases/tag/v0.1.0
