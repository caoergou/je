use std::path::Path;

use crate::{
    command::{exit_code, load_lenient, print_error, print_ok, read_file, write_file_atomic},
    engine::{FormatOptions, fix_to_value, format_compact, format_pretty},
};

/// `fmt` — 格式化（美化）JSON，原地修改。
pub fn cmd_fmt(
    file: &Path,
    indent: usize,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let (doc, repairs) = load_lenient(file)?;
    if !repairs.is_empty() {
        print_error(
            &format!("文件含 {} 个格式问题，请先使用 `fix` 修复", repairs.len()),
            json_output,
        );
        return Ok(exit_code::ERROR);
    }
    let content = format_pretty(
        &doc,
        &FormatOptions {
            indent,
            trailing_newline: true,
            sort_keys: false,
        },
    );
    write_file_atomic(file, &content)?;
    print_ok("formatted", json_output);
    Ok(exit_code::OK)
}

/// `fix` — 自动修复格式错误，然后格式化。
pub fn cmd_fix(
    file: &Path,
    dry_run: bool,
    strip_comments: bool,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let content = read_file(file)?;

    // 检查文件是否含注释
    let has_comments = content.contains("//") || content.contains("/*");
    if has_comments && !strip_comments {
        print_error(
            "文件含注释，使用 `--strip-comments` 剥离注释或手动处理",
            json_output,
        );
        return Ok(exit_code::ERROR);
    }

    let result = fix_to_value(&content);

    // 有解析错误则无法修复
    if !result.errors.is_empty() {
        for err in &result.errors {
            print_error(&format!("{err}"), json_output);
        }
        return Ok(exit_code::ERROR);
    }

    // dry-run: 只输出修复摘要
    if dry_run {
        if result.repairs.is_empty() {
            print_ok("无需修复", json_output);
        } else {
            let summary: Vec<String> = result
                .repairs
                .iter()
                .map(|r| format!("行 {}: {}", r.line, r.description))
                .collect();
            if json_output {
                let arr: Vec<serde_json::Value> = summary
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect();
                println!("{}", serde_json::json!({"ok": true, "repairs": arr}));
            } else {
                for s in &summary {
                    println!("{s}");
                }
                println!("共 {} 处修复", result.repairs.len());
            }
        }
        return Ok(exit_code::OK);
    }

    // 实际写入
    let value = result.value.ok_or("修复后无有效值")?;
    let content = format_pretty(
        &value,
        &FormatOptions {
            indent: 2,
            trailing_newline: true,
            sort_keys: false,
        },
    );
    write_file_atomic(file, &content)?;
    print_ok(
        &format!("fixed {} issues", result.repairs.len()),
        json_output,
    );
    Ok(exit_code::OK)
}

/// `minify` — 压缩 JSON（移除所有空白），原地修改。
pub fn cmd_minify(file: &Path, json_output: bool) -> Result<i32, Box<dyn std::error::Error>> {
    let (doc, _) = load_lenient(file)?;
    let content = format_compact(&doc);
    write_file_atomic(file, &content)?;
    print_ok("minified", json_output);
    Ok(exit_code::OK)
}
