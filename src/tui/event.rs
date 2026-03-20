#![allow(clippy::unnested_or_patterns)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, AppMode, StatusLevel};

/// 处理终端事件，更新 App 状态。
pub fn handle_event(app: &mut App, event: &Event) {
    match event {
        Event::Key(key) => handle_key(app, *key),
        Event::Mouse(mouse) => handle_mouse(app, *mouse),
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    match &app.mode.clone() {
        AppMode::Normal => handle_normal(app, key),
        AppMode::Edit { .. } => handle_edit(app, key),
        AppMode::ConfirmStripComments => handle_confirm(app, key),
        AppMode::Search { .. } => handle_search(app, key),
        AppMode::AddNode { .. } => handle_add_node(app, key),
    }
}

// ── 普通模式 ─────────────────────────────────────────────────────────────────

fn handle_normal(app: &mut App, key: KeyEvent) {
    // 先清除上次状态消息
    app.status = None;

    match (key.code, key.modifiers) {
        // 移动
        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => app.move_up(),
        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => app.move_down(),

        // 展开 / 折叠
        (KeyCode::Char('l'), _) | (KeyCode::Right, _) | (KeyCode::Enter, _) => {
            app.expand_or_enter();
        }
        (KeyCode::Char('h'), _) | (KeyCode::Left, _) => {
            app.collapse_or_go_parent();
        }

        // 编辑
        (KeyCode::Char('e'), _) => app.start_edit(),

        // 删除
        (KeyCode::Char('d'), _) => app.delete_current(),

        // 撤销 / 重做
        (KeyCode::Char('u'), _) => app.undo(),
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => app.redo(),

        // 保存
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => app.try_save(),

        // 搜索
        (KeyCode::Char('/'), _) => app.start_search(),

        // 添加节点
        (KeyCode::Char('a'), _) => app.start_add_node(),

        // 退出
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if app.modified {
                app.set_status(
                    "文件已修改未保存。再按 q 强制退出，或 ctrl+s 保存",
                    StatusLevel::Warn,
                );
            } else {
                app.should_quit = true;
            }
        }
        (KeyCode::Char('Q'), _) => {
            // 强制退出，不保存
            app.should_quit = true;
        }

        _ => {}
    }
}

// ── 编辑模式 ─────────────────────────────────────────────────────────────────

fn handle_edit(app: &mut App, key: KeyEvent) {
    let AppMode::Edit {
        buffer, cursor_pos, ..
    } = &mut app.mode
    else {
        return;
    };

    match key.code {
        KeyCode::Enter => {
            app.confirm_edit();
        }
        KeyCode::Esc => {
            app.cancel_edit();
        }
        KeyCode::Char(c) => {
            buffer.insert(*cursor_pos, c);
            *cursor_pos += c.len_utf8();
        }
        KeyCode::Backspace => {
            if *cursor_pos > 0 {
                let prev = prev_char_boundary(buffer, *cursor_pos);
                buffer.drain(prev..*cursor_pos);
                *cursor_pos = prev;
            }
        }
        KeyCode::Delete => {
            if *cursor_pos < buffer.len() {
                let next = next_char_boundary(buffer, *cursor_pos);
                buffer.drain(*cursor_pos..next);
            }
        }
        KeyCode::Left => {
            if *cursor_pos > 0 {
                *cursor_pos = prev_char_boundary(buffer, *cursor_pos);
            }
        }
        KeyCode::Right => {
            if *cursor_pos < buffer.len() {
                *cursor_pos = next_char_boundary(buffer, *cursor_pos);
            }
        }
        KeyCode::Home => {
            *cursor_pos = 0;
        }
        KeyCode::End => {
            *cursor_pos = buffer.len();
        }
        _ => {}
    }
}

// ── 确认模式 ─────────────────────────────────────────────────────────────────

fn handle_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_save_strip_comments();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.set_status("已取消保存", StatusLevel::Info);
        }
        _ => {}
    }
}

// ── 搜索模式 ────────────────────────────────────────────────────────────────

fn handle_search(app: &mut App, key: KeyEvent) {
    let AppMode::Search { query, cursor_pos } = &mut app.mode else {
        return;
    };

    match key.code {
        KeyCode::Enter => {
            app.search_next();
        }
        KeyCode::Esc => {
            app.cancel_search();
        }
        KeyCode::Char(c) => {
            query.insert(*cursor_pos, c);
            *cursor_pos += c.len_utf8();
        }
        KeyCode::Backspace => {
            if *cursor_pos > 0 {
                let prev = prev_char_boundary(query, *cursor_pos);
                query.drain(prev..*cursor_pos);
                *cursor_pos = prev;
            }
        }
        KeyCode::Delete => {
            if *cursor_pos < query.len() {
                let next = next_char_boundary(query, *cursor_pos);
                query.drain(*cursor_pos..next);
            }
        }
        KeyCode::Left => {
            if *cursor_pos > 0 {
                *cursor_pos = prev_char_boundary(query, *cursor_pos);
            }
        }
        KeyCode::Right => {
            if *cursor_pos < query.len() {
                *cursor_pos = next_char_boundary(query, *cursor_pos);
            }
        }
        KeyCode::Home => {
            *cursor_pos = 0;
        }
        KeyCode::End => {
            *cursor_pos = query.len();
        }
        _ => {}
    }
}

// ── 添加节点模式 ─────────────────────────────────────────────────────────────

fn handle_add_node(app: &mut App, key: KeyEvent) {
    let AppMode::AddNode {
        parent_path: _,
        is_array,
        key_buffer,
        value_buffer,
        focus_on_key,
        key_cursor,
        value_cursor,
    } = &mut app.mode
    else {
        return;
    };

    match key.code {
        KeyCode::Enter => {
            app.confirm_add_node();
        }
        KeyCode::Esc => {
            app.cancel_add_node();
        }
        KeyCode::Tab => {
            // 切换焦点（数组模式只能在 value）
            if !*is_array {
                *focus_on_key = !*focus_on_key;
            }
        }
        KeyCode::Char(c) => {
            if *focus_on_key && !*is_array {
                key_buffer.insert(*key_cursor, c);
                *key_cursor += c.len_utf8();
            } else {
                value_buffer.insert(*value_cursor, c);
                *value_cursor += c.len_utf8();
            }
        }
        KeyCode::Backspace => {
            if *focus_on_key && !*is_array {
                if *key_cursor > 0 {
                    let prev = prev_char_boundary(key_buffer, *key_cursor);
                    key_buffer.drain(prev..*key_cursor);
                    *key_cursor = prev;
                }
            } else if *value_cursor > 0 {
                let prev = prev_char_boundary(value_buffer, *value_cursor);
                value_buffer.drain(prev..*value_cursor);
                *value_cursor = prev;
            }
        }
        KeyCode::Delete => {
            if *focus_on_key && !*is_array {
                if *key_cursor < key_buffer.len() {
                    let next = next_char_boundary(key_buffer, *key_cursor);
                    key_buffer.drain(*key_cursor..next);
                }
            } else if *value_cursor < value_buffer.len() {
                let next = next_char_boundary(value_buffer, *value_cursor);
                value_buffer.drain(*value_cursor..next);
            }
        }
        KeyCode::Left => {
            if *focus_on_key && !*is_array {
                if *key_cursor > 0 {
                    *key_cursor = prev_char_boundary(key_buffer, *key_cursor);
                }
            } else if *value_cursor > 0 {
                *value_cursor = prev_char_boundary(value_buffer, *value_cursor);
            }
        }
        KeyCode::Right => {
            if *focus_on_key && !*is_array {
                if *key_cursor < key_buffer.len() {
                    *key_cursor = next_char_boundary(key_buffer, *key_cursor);
                }
            } else if *value_cursor < value_buffer.len() {
                *value_cursor = next_char_boundary(value_buffer, *value_cursor);
            }
        }
        KeyCode::Home => {
            if *focus_on_key && !*is_array {
                *key_cursor = 0;
            } else {
                *value_cursor = 0;
            }
        }
        KeyCode::End => {
            if *focus_on_key && !*is_array {
                *key_cursor = key_buffer.len();
            } else {
                *value_cursor = value_buffer.len();
            }
        }
        _ => {}
    }
}

// ── 鼠标处理 ─────────────────────────────────────────────────────────────────

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) {
    // 只处理左键点击
    if event.kind != crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }

    let lines = app.tree_lines();
    if lines.is_empty() {
        return;
    }

    // 计算点击的是哪一行（树从 y=1 开始，每行高 1）
    let row = event.row as usize;
    let item_row = row.saturating_sub(1);

    if item_row < lines.len() {
        app.cursor = item_row;
        app.list_state.select(Some(item_row));
    }
}

// ── UTF-8 辅助 ───────────────────────────────────────────────────────────────

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    while p > 0 {
        p -= 1;
        if s.is_char_boundary(p) {
            return p;
        }
    }
    0
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos + 1;
    while p <= s.len() {
        if s.is_char_boundary(p) {
            return p;
        }
        p += 1;
    }
    s.len()
}
