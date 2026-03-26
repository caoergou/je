# Jzen — JSON 配置编辑器

[English Version](./README.md)

无需费力编辑 JSON — **面向人类的交互式 TUI**，**面向 AI Agent 的优化 CLI**。

- **面向人类**: 可视化树形导航、内联编辑、语法高亮、自动修复
- **面向 AI Agent**: 最小 Token 输出、原子写入、批量操作

同一二进制文件。两种模式。一个引擎。

---

## 为什么需要 Jzen？

### 场景 1: 修改 Claude Code MCP 配置

当需要修改 Claude Code 的 `settings.json` 配置 MCP servers 时，传统方式需要：

- 将整个配置文件加载到上下文窗口（经常 100+ 行）
- 手动定位需要修改的字段
- 修改后重新写入整个文件
- **结果**: Token 消耗高，手动编辑容易出错

使用 Jzen，只需为你修改的部分付费：

```bash
# 检查结构而不读取值
jzen schema ~/.claude/settings.json

# 只获取你需要的具体值
jzen get .mcpServers.github.command ~/.claude/settings.json

# 原子更新单个字段
jzen set .mcpServers.github.env.TOKEN '"ghp_xxxx"' ~/.claude/settings.json

# 一次调用批量更新（最少往返）
jzen patch '[
  {"op": "replace", "path": ".defaultMode", "value": "acceptEdits"},
  {"op": "add", "path": ".mcpServers.github.enabled", "value": true}
]' ~/.claude/settings.json
```

**Token 节省**: 90%+ — 只读取查询的内容，而非整个文件。

---

### 场景 2: 修改 OpenClaw Agent 配置

OpenClaw 使用 JSON 配置 agent 行为。传统工具需要：

- 打开整个文件来了解其结构
- 手动编辑并保存整个文件
- 格式错误导致 agent 崩溃的风险

Jzen 让这一切变得简单：

```bash
# 一目了然地查看结构
jzen tree ~/.config/openclaw/agent.json

# 更新模型配置
jzen set .model.provider '"openai"' ~/.config/openclaw/agent.json
jzen set .model.name '"gpt-4o"' ~/.config/openclaw/agent.json

# 添加新的 MCP server
jzen set .mcpServers.github '{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-github"]
}' ~/.config/openclaw/agent.json

# 自动修复常见 JSON 错误
jzen fix --strip-comments ~/.config/openclaw/agent.json
```

**优势**: 原子写入、崩溃安全操作、自动格式修复。

---

## 对比：传统方式 vs Jzen

| 任务 | 传统方式 | Jzen |
|------|----------|------|
| 读取配置结构 | 加载整个文件 | `schema` → 仅类型输出 |
| 读取具体值 | 解析完整 JSON | `get .key` → 单个值 |
| 修改一个字段 | 重写整个文件 | `set .key val` → 原子操作 |
| 多个修改 | 多次往返 | `patch` → 单次调用 |
| 修复 JSON 错误 | 手动修复 | `fix` → 自动修复 |
| Token 成本 | 完整文件在上下文中 | 仅查询的值 |

---

## 快速开始

```bash
# TUI 模式（人类）
jzen config.json

# 命令模式（agent / 脚本）
jzen get .name config.json
jzen set .name '"Bob"' config.json
jzen fix --strip-comments config.json

# 两种参数顺序都支持
jzen config.json get .name
jzen get .name config.json
```

---

## 安装

### 包管理器（推荐）

```bash
# macOS / Linux (Homebrew)
brew install caoergou/jzen/jzen

# Debian / Ubuntu
sudo dpkg -i jzen_*.deb

# Fedora / RHEL / CentOS
sudo rpm -i jzen-*.rpm
```

### 安装脚本

```bash
# Linux / macOS — 自动检测平台，安装到 /usr/local/bin
curl -fsSL https://github.com/caoergou/jzen/releases/latest/download/install.sh | sh

# 跳过自动安装 shell 补全
SKIP_COMPLETIONS=1 curl -fsSL https://github.com/caoergou/jzen/releases/latest/download/install.sh | sh

# 自定义安装目录
INSTALL_DIR=~/.local/bin curl -fsSL https://github.com/caoergou/jzen/releases/latest/download/install.sh | sh
```

安装脚本会：
1. 为你的平台下载正确的二进制文件
2. 检测你的 shell (bash/zsh/fish)
3. 自动将 shell 补全安装到适当的位置
4. 如果需要手动设置，会给出提示

### 预编译二进制

从 [Releases](https://github.com/caoergou/jzen/releases) 页面下载：

| 平台 | 二进制文件 |
|------|-----------|
| Linux x86_64 | `jzen-linux-x86_64` |
| Linux aarch64 | `jzen-linux-aarch64` |
| macOS x86_64 | `jzen-macos-x86_64` |
| macOS Apple Silicon | `jzen-macos-aarch64` |
| Windows x86_64 | `jzen-windows-x86_64.exe` |

将二进制文件放到你的 `$PATH` 中的某个位置。

### 从 crates.io 安装

```bash
cargo install jzen
```

### 从源码安装（需要 Rust）

```bash
cargo install --git https://github.com/caoergou/jzen
```

---

## Agent Skill

安装 jzen skill 使 AI agents 能够以最小的 token 使用量编辑 JSON：

```bash
# 为 Claude Code、OpenClaw、Codex 等安装
npx skills add caoergou/jzen

# 或从本仓库安装特定 skill
npx skills add caoergou/jzen --skill jzen
```

安装后，agent 将自动使用 jzen 进行 JSON 操作，Token 消耗降低 90%+。

---

## TUI 模式

通过仅传入文件名启动：

```bash
jzen settings.json
```

| 按键 | 操作 |
|------|------|
| `↑/↓` | 上/下移动 |
| `←` | 折叠 / 返回父级 |
| `→` / `Space` | 展开 / 切换 |
| `Enter` | 编辑叶子节点 / 展开容器 |
| `N` / `Insert` | 添加新节点 |
| `Delete` | 删除当前节点 |
| `Ctrl+S` | 保存 |
| `Ctrl+F` / `/` | 搜索 |
| `Ctrl+Z` | 撤销 |
| `Ctrl+Y` | 重做 |
| `F1` | 帮助 |
| `q` | 退出（有未保存更改时提示）|

---

## 命令模式

专为 **AI agents** 设计，以最小的 token 使用量读取和写入 JSON。

### 读取

```bash
jzen get .key file.json              # 获取路径处的值
jzen get '.servers[0].host' file.json
jzen keys . file.json                # 列出所有顶层键
jzen len .tags file.json             # 数组 / 对象长度
jzen type .count file.json           # 类型名称: string|number|boolean|null|object|array
jzen exists .key file.json           # 退出 0=存在, 2=未找到
jzen schema file.json                # 推断结构（无值）
jzen check file.json                 # 验证；错误到 stderr
```

### 写入

```bash
jzen set .name '"Bob"' file.json     # 设置值
jzen del .legacy file.json           # 删除键
jzen add .tags '"go"' file.json      # 追加到数组
jzen mv .oldKey .newKey file.json    # 重命名键

# 批量（JSON Patch RFC 6902）— 一次调用，原子操作
jzen patch '[
  {"op": "replace", "path": ".name",    "value": "Bob"},
  {"op": "add",     "path": ".tags/-",  "value": "go"},
  {"op": "remove",  "path": ".legacy"}
]' file.json
```

### 格式化 / 修复

```bash
jzen fmt file.json                   # 原地美化格式化
jzen fix --strip-comments file.json  # 自动修复 JSONC、尾随逗号等
jzen fix --dry-run file.json         # 预览修复而不写入
jzen minify file.json                # 压缩 JSON
jzen diff old.json new.json          # 结构化 diff
```

### 检查 / 转换

```bash
jzen tree file.json                  # 显示为缩进树
jzen tree -e file.json               # 展开所有节点
jzen tree -p .servers file.json      # 子路径的树视图
jzen query '.users[0]' file.json     # get 的别名，带路径过滤语义
jzen validate schema.json file.json  # 根据 JSON Schema 验证
jzen convert yaml file.json          # 转换为 YAML
jzen convert toml file.json          # 转换为 TOML
```

### 发现

```bash
jzen commands                        # 列出所有可用命令
jzen explain get                     # 特定命令的详细帮助
jzen completions bash                # 生成 shell 补全脚本
jzen completions zsh
jzen completions fish
```

### 全局选项

| 选项 | 描述 |
|------|------|
| `--json` | 将所有输出包装为 `{"ok":...,"value":...}` |
| `--lang <lang>` | 输出语言: `en`, `zh-CN`, `zh-TW` |
| `--quiet` | 抑制信息输出 |
| `-h, --help` | 显示帮助 |
| `-V, --version` | 显示版本 |

### 退出码

| 码 | 含义 |
|----|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 路径未找到 |
| 3 | 类型不匹配 |

---

## AI Agents 为什么选择 Jzen？

| 传统方式 | jzen 命令模式 |
|----------|---------------|
| 将整个文件读入上下文 | `get .key` → 仅目标值 |
| 修改后重写整个文件 | `set .key val` → 返回 `ok` |
| Agent 手动解析 JSON | 路径寻址处理导航 |
| Agent 格式错误时重试 | `fix` 自动修复错误 |
| 多次往返 | `patch` 一次调用批量修改 |

### 示例：配置 Claude Code MCP server

```bash
# 1. 检查文件结构而不读取值
jzen schema ~/.claude/settings.json

# 2. 检查 server 是否存在
jzen exists .mcpServers.github ~/.claude/settings.json

# 3. 只读取需要的具体值
jzen get .mcpServers.github.command ~/.claude/settings.json

# 4. 更新单个字段
jzen set .mcpServers.github.env.TOKEN '"ghp_xxxx"' ~/.claude/settings.json

# 5. 批量更新（一次调用）
jzen patch '[
  {"op": "replace", "path": ".defaultMode", "value": "acceptEdits"},
  {"op": "add",     "path": ".mcpServers.github.enabled", "value": true}
]' ~/.claude/settings.json
```

### 示例：配置 OpenClaw agent

```bash
# 1. 检查配置结构
jzen tree ~/.config/openclaw/agent.json

# 2. 更新模型配置
jzen set .model.provider '"openai"' ~/.config/openclaw/agent.json
jzen set .model.name '"gpt-4o"' ~/.config/openclaw/agent.json

# 3. 添加新的 MCP server
jzen set .mcpServers.github '{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-github"]
}' ~/.config/openclaw/agent.json

# 4. 修复和格式化后再保存
jzen fix --strip-comments ~/.config/openclaw/agent.json
```

---

## 自动修复功能

`jzen fix` 修复最常见的 JSON 格式错误：

| 错误 | 示例 | 修复 |
|------|------|------|
| 尾随逗号 | `{"a": 1,}` | 移除 |
| 单引号 | `{'key': 'val'}` | 替换为双引号 |
| 未加引号的键 | `{key: "val"}` | 添加引号 |
| 缺失逗号 | `{"a": 1 "b": 2}` | 插入逗号 |
| 行注释 | `// comment` | 剥离 |
| 块注释 | `/* comment */` | 剥离 |
| Python 字面量 | `True`, `False`, `None` | 替换为 JSON 等价物 |
| BOM | 领先的 `\uFEFF` | 剥离 |

---

## 路径语法

使用类 jq 的路径语法：

```
.                      # 根
.key                   # 对象字段
.key.nested            # 嵌套字段
.array[0]              # 数组索引
.array[-1]             # 最后一个元素
.key.array[2].field    # 深度路径
```

---

## Stdin / 管道支持

所有读取命令在没有文件参数时接受来自 stdin 的 JSON：

```bash
cat config.json | jzen get .name
echo '{"a":1}' | jzen schema
```

---

## 从源码构建

```bash
git clone https://github.com/caoergou/jzen
cd jzen
cargo build --release
./target/release/jzen --version
```

---

## Shell 补全

为 `jzen` 命令和选项启用 tab 补全。

### Bash

```bash
# 写入 bash-completion 目录（推荐）
jzen completions bash > ~/.local/share/bash-completion/completions/jzen

# 或添加到自定义目录并 source
jzen completions bash > ~/.bash_completion.d/jzen
echo 'source ~/.bash_completion.d/jzen' >> ~/.bashrc
```

### Zsh

```bash
# 写入 fpath 目录
mkdir -p ~/.zfunc
jzen completions zsh > ~/.zfunc/_jzen

# 添加到 .zshrc（在任何 compinit 调用之前）：
# fpath=(~/.zfunc $fpath)

# 重载 shell
exec zsh
```

### 其他 Shell

也支持 Fish、PowerShell 和 Elvish。详见 [CLI_SPEC.md](CLI_SPEC.md#completions-shell)。

---

## 路线图

### v1.x — ✅ 完成（完善和分发）

- [x] Shell 补全 (bash/zsh/fish/powershell/elvish)
- [x] `diff --json` 结构化输出模式
- [x] TOML 转换 (`jzen convert toml`)
- [x] 完整 JSON Schema 验证 (`type`, `required`, `properties`, `minimum`, `maximum`, `minLength`, `maxLength`, `minItems`, `maxItems`, `items`, `enum`)
- [x] 包管理器分发: Homebrew, apt/deb, rpm
- [x] YAML 转换 (`jzen convert yaml`)
- [x] TUI 模式文件监控

### v2.x — 强力功能（进行中）

- [ ] 交互式 shell 模式 (`jzen shell`) — 持久 REPL，无需重新打开文件即可批量编辑
- [ ] 保存时保留 JSONC 注释（基于 CST；当前写入时剥离）
- [ ] TUI 鼠标支持
- [ ] 大文件优化（> 1 MB 文件的虚拟滚动）

### v3.x — 长期

- [ ] TUI 多文件标签
- [ ] JSON Pointer (RFC 6901) 作为替代路径语法

---

## 许可证

MIT
