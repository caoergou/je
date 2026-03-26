//! Integration tests for command layer
//!
//! Tests the full workflow: CLI args → command execution → file I/O

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_jzen_binary() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("target/release/jzen")
}

fn create_temp_json(content: &str) -> tempfile::NamedTempFile {
    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    fs::write(tmp.path(), content).expect("Failed to write temp file");
    tmp
}

#[test]
fn test_get_string_value() {
    let tmp = create_temp_json(r#"{"name": "test", "value": 42}"#);
    let output = Command::new(get_jzen_binary())
        .args(["get", ".name", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"));
}

#[test]
fn test_get_nested_value() {
    let tmp = create_temp_json(r#"{"server": {"host": "localhost", "port": 8080}}"#);
    let output = Command::new(get_jzen_binary())
        .args(["get", ".server.host", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("localhost"));
}

#[test]
fn test_get_array_element() {
    let tmp = create_temp_json(r#"{"items": ["a", "b", "c"]}"#);
    let output = Command::new(get_jzen_binary())
        .args(["get", ".items[1]", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('b'));
}

#[test]
fn test_get_nonexistent_path() {
    let tmp = create_temp_json(r#"{"name": "test"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["get", ".nonexistent", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_set_string_value() {
    let tmp = create_temp_json(r#"{"name": "old"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["set", ".name", "\"new\"", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(content.contains("new"));
}

#[test]
fn test_set_number_value() {
    let tmp = create_temp_json(r#"{"count": 1}"#);
    let output = Command::new(get_jzen_binary())
        .args(["set", ".count", "42", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(content.contains("42"));
}

#[test]
fn test_set_creates_new_key() {
    let tmp = create_temp_json(r"{}");
    let output = Command::new(get_jzen_binary())
        .args([
            "set",
            ".newKey",
            "\"newValue\"",
            tmp.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(content.contains("newKey"));
}

#[test]
fn test_del_key() {
    let tmp = create_temp_json(r#"{"name": "test", "toDelete": "value"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["del", ".toDelete", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(!content.contains("toDelete"));
}

#[test]
fn test_add_to_array() {
    let tmp = create_temp_json(r#"{"items": [1, 2]}"#);
    let output = Command::new(get_jzen_binary())
        .args(["add", ".items", "3", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(content.contains('3'));
}

#[test]
fn test_fix_trailing_comma() {
    let tmp = create_temp_json(r#"{"items": [1, 2, 3,]}"#);
    let output = Command::new(get_jzen_binary())
        .args(["fix", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    let _: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON after fix");
}

#[test]
fn test_fix_single_quotes() {
    let tmp = create_temp_json(r"{'name': 'test'}");
    let output = Command::new(get_jzen_binary())
        .args(["fix", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    let _: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON after fix");
}

#[test]
fn test_fmt_pretty_print() {
    let tmp = create_temp_json(r#"{"name":"test","value":42}"#);
    let output = Command::new(get_jzen_binary())
        .args(["fmt", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let content = fs::read_to_string(tmp.path()).expect("Failed to read file");
    assert!(content.contains('\n'));
}

#[test]
fn test_schema_output() {
    let tmp = create_temp_json(
        r#"{"name": "test", "count": 42, "active": true, "items": [1], "nested": {}}"#,
    );
    let output = Command::new(get_jzen_binary())
        .args(["schema", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("string"));
    assert!(stdout.contains("number"));
}

#[test]
fn test_exists_true() {
    let tmp = create_temp_json(r#"{"name": "test"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["exists", ".name", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
}

#[test]
fn test_exists_false() {
    let tmp = create_temp_json(r#"{"name": "test"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["exists", ".nonexistent", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_check_valid_json() {
    let tmp = create_temp_json(r#"{"name": "test"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["check", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
}

#[test]
fn test_check_invalid_json() {
    let tmp = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    fs::write(tmp.path(), r#"{"broken": }"#).expect("Failed to write temp file");

    let output = Command::new(get_jzen_binary())
        .args(["check", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(!output.status.success());
}

#[test]
fn test_json_output_mode() {
    let tmp = create_temp_json(r#"{"name": "test"}"#);
    let output = Command::new(get_jzen_binary())
        .args(["--json", "get", ".name", tmp.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute jzen");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
}
