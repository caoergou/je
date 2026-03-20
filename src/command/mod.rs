pub mod read;
pub mod repair;
pub mod write;

use std::{fs, path::Path, process};

use crate::{
    cli::Command,
    engine::{JsonValue, parse_lenient},
};

/// 命令模式的统一退出码。
pub mod exit_code {
    pub const OK: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const NOT_FOUND: i32 = 2;
    pub const TYPE_MISMATCH: i32 = 3;
}

/// 执行命令并以适当的退出码退出。
pub fn run(file: &Path, cmd: Command, json_output: bool) {
    let result = dispatch(file, cmd, json_output);
    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            print_error(&format!("{e}"), json_output);
            process::exit(exit_code::ERROR);
        }
    }
}

fn dispatch(
    file: &Path,
    cmd: Command,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    match cmd {
        Command::Get { path } => read::cmd_get(file, &path, json_output),
        Command::Keys { path } => read::cmd_keys(file, &path, json_output),
        Command::Len { path } => read::cmd_len(file, &path, json_output),
        Command::Type { path } => read::cmd_type(file, &path, json_output),
        Command::Exists { path } => read::cmd_exists(file, &path, json_output),
        Command::Schema => read::cmd_schema(file, json_output),
        Command::Check => read::cmd_check(file, json_output),
        Command::Set { path, value } => write::cmd_set(file, &path, &value, json_output),
        Command::Del { path } => write::cmd_del(file, &path, json_output),
        Command::Add { path, value } => write::cmd_add(file, &path, &value, json_output),
        Command::Patch { operations } => write::cmd_patch(file, &operations, json_output),
        Command::Mv { src, dst } => write::cmd_mv(file, &src, &dst, json_output),
        Command::Fmt { indent } => repair::cmd_fmt(file, indent, json_output),
        Command::Fix {
            dry_run,
            strip_comments,
        } => repair::cmd_fix(file, dry_run, strip_comments, json_output),
        Command::Minify => repair::cmd_minify(file, json_output),
        Command::Diff { other } => read::cmd_diff(file, &other, json_output),
        Command::Completions { .. } => unreachable!("completions is handled in main"),
    }
}

// ── 输出帮助函数 ──────────────────────────────────────────────────────────────

/// 输出一个 JSON 值。json 模式下包装为 `{"ok":true,"value":...}`。
pub(crate) fn print_json_value(value: &JsonValue, json_output: bool) {
    use crate::engine::format_compact;
    let compact = format_compact(value);
    if json_output {
        println!("{{\"ok\":true,\"value\":{compact}}}");
    } else {
        println!("{compact}");
    }
}

/// 输出纯字符串值。json 模式下包装为 `{"ok":true,"value":"..."}` (字符串类型)。
pub(crate) fn print_str(value: &str, json_output: bool) {
    if json_output {
        println!("{}", serde_json::json!({"ok": true, "value": value}));
    } else {
        println!("{value}");
    }
}

/// 输出整数值。json 模式下包装为 `{"ok":true,"value":n}`。
pub(crate) fn print_usize(n: usize, json_output: bool) {
    if json_output {
        println!("{{\"ok\":true,\"value\":{n}}}");
    } else {
        println!("{n}");
    }
}

/// 输出字符串列表。json 模式下包装为 `{"ok":true,"value":[...]}`。
pub(crate) fn print_string_list(lines: &[String], json_output: bool) {
    if json_output {
        let arr: Vec<serde_json::Value> = lines
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        println!("{}", serde_json::json!({"ok": true, "value": arr}));
    } else {
        for line in lines {
            println!("{line}");
        }
    }
}

/// 输出成功消息。json 模式下只输出 `{"ok":true}`。
pub(crate) fn print_ok(msg: &str, json_output: bool) {
    if json_output {
        println!("{{\"ok\":true}}");
    } else {
        println!("{msg}");
    }
}

/// 输出错误消息。json 模式下输出到 stdout，普通模式输出到 stderr。
pub(crate) fn print_error(msg: &str, json_output: bool) {
    if json_output {
        println!("{}", serde_json::json!({"ok": false, "error": msg}));
    } else {
        eprintln!("{msg}");
    }
}

// ── 文件 I/O 帮助函数 ─────────────────────────────────────────────────────────

/// 读取文件内容，返回错误信息若文件不存在。
pub(crate) fn read_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    fs::read_to_string(path).map_err(|e| format!("无法读取 '{}': {e}", path.display()).into())
}

/// 读取并宽松解析文件，返回文档和修复列表。
pub(crate) fn load_lenient(
    path: &Path,
) -> Result<(JsonValue, Vec<crate::engine::Repair>), Box<dyn std::error::Error>> {
    let content = read_file(path)?;
    let output =
        parse_lenient(&content).map_err(|e| format!("解析失败 '{}': {e}", path.display()))?;
    Ok((output.value, output.repairs))
}

/// 原子写入文件：写临时文件 → fsync → 重命名。
pub(crate) fn write_file_atomic(
    path: &Path,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content).map_err(|e| format!("写入临时文件失败: {e}"))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("重命名文件失败: {e}"))?;
    Ok(())
}
