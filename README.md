# Jzen — JSON Config Editor

[中文版本](./README-zh.md)

JSON editor with **TUI for humans** and **CLI for AI agents**.

[![CI](https://github.com/caoergou/jzen/actions/workflows/ci.yml/badge.svg)](https://github.com/caoergou/jzen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Features

- **Dual Interface**: TUI for interactive editing, CLI for scripting and AI agents
- **AI-Agent Friendly**: Structured JSON output (`--json` flag), token-efficient commands
- **Auto-Repair**: Fix common JSON errors (trailing commas, single quotes, unquoted keys, comments)
- **Atomic Writes**: Crash-safe file operations with fsync
- **Path Navigation**: jq-inspired syntax (`.key`, `.arr[0]`, `.arr[-1]`)
- **Format Conversion**: JSON ↔ YAML ↔ TOML
- **Schema Inference**: Generate type summaries without values
- **Shell Completions**: Bash, Zsh, Fish, PowerShell, Elvish
- **Cross-Platform**: Linux, macOS, Windows

---

## Why Jzen?

### Problem 1: Claude Code MCP Config

Traditional: load entire file → find field → rewrite full file → high token cost

With Jzen:
```bash
jzen schema ~/.claude/settings.json        # structure only
jzen get .mcpServers.github.command ~      # single value
jzen set .mcpServers.github.env.TOKEN '"xxx"' ~
jzen patch '[{"op":"replace",path:".defaultMode",value:"Command"}]' ~
```
**Token savings: 90%+**

### Problem 2: OpenClaw Agent Config

Traditional: open full file → manual edit → risk format errors

With Jzen:
```bash
jzen tree ~/.config/openclaw/agent.json
jzen set .model.provider '"openai"' ~/.config/openclaw/agent.json
jzen fix --strip-comments ~/.config/openclaw/agent.json
```
**Atomic writes, auto-repair**

---

## Quick Start

```bash
# TUI (human)
jzen config.json

# CLI (agent)
jzen get .name config.json
jzen set .name '"Bob"' config.json
jzen fix --strip-comments config.json
```

---

## Install

```bash
# One-liner (auto-installs completions)
curl -fsSL https://github.com/caoergou/jzen/releases/latest/download/install.sh | sh

# Or Homebrew
brew install caoergou/jzen/jzen

# Or download binary from Releases
# https://github.com/caoergou/jzen/releases
```

**Requirements**: Rust 1.85+ (for building from source)

---

## Commands

| Command | Description |
|---------|-------------|
| `get .key f.json` | Read value at path |
| `set .key val f.json` | Set value (creates if missing) |
| `del .key f.json` | Delete key/array element |
| `add .arr val f.json` | Append to array |
| `patch '[...]' f.json` | Batch operations (RFC 6902) |
| `keys . f.json` | List keys at path |
| `len .arr f.json` | Array length / object key count |
| `type .key f.json` | Get value type |
| `exists .key f.json` | Check path exists (exit 0/2) |
| `schema f.json` | Generate type summary |
| `tree f.json` | Visual tree display |
| `check f.json` | Validate JSON syntax |
| `fix f.json` | Auto-repair JSON errors |
| `fmt f.json` | Pretty-print |
| `minify f.json` | Remove whitespace |
| `diff other.json f.json` | Compare two JSON files |
| `convert yaml f.json` | Convert to YAML/TOML |
| `validate schema.json f.json` | Validate against JSON Schema |

**Path syntax**: `.key`, `.arr[0]`, `.arr[-1]`, `.a.b.c`

**Exit codes**: 0=success, 1=error, 2=not found, 3=type mismatch

---

## AI Agent Integration

### JSON Output Mode

All commands support `--json` flag for structured output:

```bash
jzen --json get .name config.json
# {"value":"test","path":".name"}

jzen --json schema config.json
# {"type":"object","properties":{...}}
```

### Use as Agent Skill

```bash
npx skills add caoergou/jzen
```

---

## TUI Keys

| Key | Action |
|-----|--------|
| `↑/↓/←/→` | Navigate tree |
| `Enter` | Edit value |
| `e` | Edit key name |
| `N` | Add node |
| `Delete` | Delete node |
| `Ctrl+S` | Save |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `/` | Search |
| `?` | Help |
| `q` | Quit |

---

## Auto-Repair Capabilities

`jzen fix` repairs common JSON format errors:

| Issue | Fix |
|-------|-----|
| Trailing commas | `[1, 2,]` → `[1, 2]` |
| Single quotes | `{'key': 'value'}` → `{"key": "value"}` |
| Unquoted keys | `{name: "test"}` → `{"name": "test"}` |
| Missing commas | Auto-insert |
| Comments | `//` and `/* */` (with `--strip-comments`) |
| Python literals | `True/False/None` → `true/false/null` |
| BOM | Strip UTF-8 BOM |

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## Security

If you discover a security vulnerability, please report it by creating a private security advisory on GitHub. Do not open a public issue.

---

## License

MIT License - see [LICENSE](LICENSE) for details.
