use crate::engine::value::JsonValue;

/// 两个 JSON 文档之间的单条结构差异。
pub struct DiffEntry {
    /// jed 路径表达式（如 `.name`、`.servers[0].host`）
    pub path: String,
    /// 差异类型
    pub kind: DiffKind,
}

/// 差异类型。
pub enum DiffKind {
    /// 存在于 `a` 但不存在于 `b`（已删除）
    Removed(JsonValue),
    /// 存在于 `b` 但不存在于 `a`（已添加）
    Added(JsonValue),
    /// 路径相同但标量值不同
    Changed { from: JsonValue, to: JsonValue },
}

/// 计算 `a`（旧文档）和 `b`（新文档）之间的结构差异。
///
/// 返回按路径描述变更的 [`DiffEntry`] 列表。
/// 相同的值不会出现在结果中。
#[must_use]
pub fn structural_diff(a: &JsonValue, b: &JsonValue) -> Vec<DiffEntry> {
    let mut out = Vec::new();
    diff_at(a, b, ".", &mut out);
    out
}

fn diff_at(a: &JsonValue, b: &JsonValue, path: &str, out: &mut Vec<DiffEntry>) {
    match (a, b) {
        (JsonValue::Object(a_map), JsonValue::Object(b_map)) => {
            // 删除或修改的 key
            for (key, a_val) in a_map {
                let child = key_path(path, key);
                match b_map.get(key.as_str()) {
                    Some(b_val) => diff_at(a_val, b_val, &child, out),
                    None => out.push(DiffEntry {
                        path: child,
                        kind: DiffKind::Removed(a_val.clone()),
                    }),
                }
            }
            // 新增的 key
            for (key, b_val) in b_map {
                if !a_map.contains_key(key.as_str()) {
                    out.push(DiffEntry {
                        path: key_path(path, key),
                        kind: DiffKind::Added(b_val.clone()),
                    });
                }
            }
        }
        (JsonValue::Array(a_arr), JsonValue::Array(b_arr)) => {
            let len = a_arr.len().max(b_arr.len());
            for i in 0..len {
                let child = idx_path(path, i);
                match (a_arr.get(i), b_arr.get(i)) {
                    (Some(av), Some(bv)) => diff_at(av, bv, &child, out),
                    (Some(av), None) => out.push(DiffEntry {
                        path: child,
                        kind: DiffKind::Removed(av.clone()),
                    }),
                    (None, Some(bv)) => out.push(DiffEntry {
                        path: child,
                        kind: DiffKind::Added(bv.clone()),
                    }),
                    (None, None) => {} // 不可达：i < max(a.len(), b.len())
                }
            }
        }
        (a_val, b_val) if a_val != b_val => {
            out.push(DiffEntry {
                path: path.to_string(),
                kind: DiffKind::Changed {
                    from: a_val.clone(),
                    to: b_val.clone(),
                },
            });
        }
        _ => {} // 值相同，无差异
    }
}

fn key_path(parent: &str, key: &str) -> String {
    if parent == "." {
        format!(".{key}")
    } else {
        format!("{parent}.{key}")
    }
}

fn idx_path(parent: &str, idx: usize) -> String {
    format!("{parent}[{idx}]")
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    fn obj(pairs: &[(&str, JsonValue)]) -> JsonValue {
        let mut map = IndexMap::new();
        for (k, v) in pairs {
            map.insert((*k).to_string(), v.clone());
        }
        JsonValue::Object(map)
    }

    #[test]
    fn identical_docs_produce_no_entries() {
        let a = obj(&[("name", JsonValue::String("Alice".into()))]);
        let b = a.clone();
        assert!(structural_diff(&a, &b).is_empty());
    }

    #[test]
    fn changed_scalar_produces_changed_entry() {
        let a = obj(&[("name", JsonValue::String("Alice".into()))]);
        let b = obj(&[("name", JsonValue::String("Bob".into()))]);
        let entries = structural_diff(&a, &b);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, ".name");
        assert!(matches!(entries[0].kind, DiffKind::Changed { .. }));
    }

    #[test]
    fn removed_key_produces_removed_entry() {
        let a = obj(&[
            ("name", JsonValue::String("Alice".into())),
            ("age", JsonValue::Number(30.0)),
        ]);
        let b = obj(&[("name", JsonValue::String("Alice".into()))]);
        let entries = structural_diff(&a, &b);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, ".age");
        assert!(matches!(entries[0].kind, DiffKind::Removed(_)));
    }

    #[test]
    fn added_key_produces_added_entry() {
        let a = obj(&[("name", JsonValue::String("Alice".into()))]);
        let b = obj(&[
            ("name", JsonValue::String("Alice".into())),
            ("version", JsonValue::Number(2.0)),
        ]);
        let entries = structural_diff(&a, &b);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, ".version");
        assert!(matches!(entries[0].kind, DiffKind::Added(_)));
    }

    #[test]
    fn array_element_change_uses_index_path() {
        let a = JsonValue::Array(vec![
            JsonValue::String("rust".into()),
            JsonValue::String("cli".into()),
        ]);
        let b = JsonValue::Array(vec![
            JsonValue::String("rust".into()),
            JsonValue::String("go".into()),
        ]);
        let entries = structural_diff(&a, &b);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, ".[1]");
    }

    #[test]
    fn nested_path_is_correct() {
        let inner_a = obj(&[("host", JsonValue::String("localhost".into()))]);
        let inner_b = obj(&[("host", JsonValue::String("remotehost".into()))]);
        let a = obj(&[("server", inner_a)]);
        let b = obj(&[("server", inner_b)]);
        let entries = structural_diff(&a, &b);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, ".server.host");
    }
}
