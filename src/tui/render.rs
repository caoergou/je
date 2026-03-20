#![allow(clippy::collapsible_else_if)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use super::app::{App, AppMode, StatusLevel};
use super::tree::TreeLine;

/// 每帧的主渲染入口。
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let lines = app.tree_lines();

    // 布局：树形主区域 + 底部状态栏
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_tree(frame, app, chunks[0], &lines);
    render_statusbar(frame, app, chunks[1], &lines);

    // 编辑覆盖层
    if matches!(app.mode, AppMode::Edit { .. }) {
        render_edit_overlay(frame, app, area);
    }

    // 搜索覆盖层
    if matches!(app.mode, AppMode::Search { .. }) {
        render_search_overlay(frame, app, area);
    }

    // 添加节点覆盖层
    if matches!(app.mode, AppMode::AddNode { .. }) {
        render_add_node_overlay(frame, app, area);
    }

    // 确认剥离注释覆盖层
    if matches!(app.mode, AppMode::ConfirmStripComments) {
        render_confirm_overlay(frame, area);
    }
}

// ── 树形视图 ─────────────────────────────────────────────────────────────────

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect, lines: &[TreeLine]) {
    let modified_marker = if app.modified { " [*]" } else { "" };
    let title = format!(
        " je: {}{modified_marker} ",
        app.file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );

    // 搜索模式下高亮匹配项
    let search_query = if let AppMode::Search { query, .. } = &app.mode {
        if query.is_empty() {
            None
        } else {
            Some(query.to_lowercase())
        }
    } else {
        None
    };

    let items: Vec<ListItem> = lines
        .iter()
        .map(|line| make_list_item(line, search_query.as_deref()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn make_list_item<'a>(line: &'a TreeLine, search_query: Option<&'a str>) -> ListItem<'a> {
    let indent = "  ".repeat(line.depth);

    // 展开/折叠指示符
    let indicator = if line.path.starts_with("__close__") {
        "  "
    } else if line.has_children {
        if line.is_expanded { "▼ " } else { "▶ " }
    } else {
        "  "
    };

    // 检查是否匹配搜索
    let is_match = search_query.is_some_and(|q| {
        line.display_key.to_lowercase().contains(q) || line.value_preview.to_lowercase().contains(q)
    });

    // 搜索匹配时的高亮样式
    let match_style = if is_match {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };

    // key 部分的颜色
    let key_span = if line.display_key.is_empty() {
        Span::raw("")
    } else {
        Span::styled(
            format!("{}: ", line.display_key),
            Style::default().fg(Color::Cyan),
        )
    };

    // 值的颜色
    let value_color = match line.value_type {
        "string" => Color::Green,
        "number" => Color::Yellow,
        "boolean" => Color::Magenta,
        "null" => Color::DarkGray,
        _ => Color::White,
    };

    let value_span = Span::styled(line.value_preview.clone(), Style::default().fg(value_color));

    ListItem::new(Line::from(vec![
        Span::styled(format!("{indent}{indicator}"), match_style),
        key_span,
        value_span,
    ]))
    .style(match_style)
}

// ── 状态栏 ───────────────────────────────────────────────────────────────────

fn render_statusbar(frame: &mut Frame, app: &App, area: Rect, _lines: &[TreeLine]) {
    let path = app.current_path();

    let status_text = if let Some((msg, level)) = &app.status {
        let color = match level {
            StatusLevel::Info => Color::Green,
            StatusLevel::Warn => Color::Yellow,
            StatusLevel::Error => Color::Red,
        };
        Line::from(vec![
            Span::styled(format!(" {path} "), Style::default().fg(Color::DarkGray)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {msg} "), Style::default().fg(color)),
        ])
    } else {
        let hints = match &app.mode {
            AppMode::Normal => {
                " j/k:移动  h/l:折叠/展开  e:编辑  d:删除  a:添加  /:搜索  u:撤销  ctrl+s:保存  q:退出"
            }
            AppMode::Edit { .. } => " Enter:确认  Esc:取消",
            AppMode::Search { .. } => " 输入搜索内容  Enter:跳转下一匹配  Esc:退出",
            AppMode::AddNode { is_array, .. } => {
                if *is_array {
                    " 输入值  Tab:切换  Enter:确认  Esc:取消"
                } else {
                    " 输入 key 和 value  Tab:切换焦点  Enter:确认  Esc:取消"
                }
            }
            AppMode::ConfirmStripComments => " y:确认剥离注释并保存  n:取消",
        };
        Line::from(vec![
            Span::styled(format!(" {path} "), Style::default().fg(Color::DarkGray)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(hints, Style::default().fg(Color::DarkGray)),
        ])
    };

    let bar = Paragraph::new(status_text).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

// ── 编辑覆盖层 ───────────────────────────────────────────────────────────────

fn render_edit_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let AppMode::Edit {
        path,
        buffer,
        cursor_pos,
    } = &app.mode
    else {
        return;
    };

    // 覆盖层位置：底部 3 行
    let overlay_height = 3u16;
    if area.height < overlay_height + 2 {
        return;
    }
    let overlay_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - overlay_height - 1,
        width: area.width.saturating_sub(2),
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);

    let display_buf = format!("{buffer} ");
    let title = format!(" 编辑 {path} ");

    let para = Paragraph::new(display_buf)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(para, overlay_area);

    // 设置光标位置（+1 是边框偏移）
    let cursor_x = overlay_area.x + 1 + (*cursor_pos as u16).min(overlay_area.width - 3);
    let cursor_y = overlay_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

// ── 搜索覆盖层 ─────────────────────────────────────────────────────────────

fn render_search_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let AppMode::Search { query, cursor_pos } = &app.mode else {
        return;
    };

    let overlay_height = 3u16;
    if area.height < overlay_height + 2 {
        return;
    }
    let overlay_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - overlay_height - 1,
        width: area.width.saturating_sub(2),
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);

    let display_buf = format!("/ {query} ");
    let title = " 搜索 ";

    let para = Paragraph::new(display_buf)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(para, overlay_area);

    // 光标位置（+3 是因为前面有 "/ "）
    let cursor_x = overlay_area.x + 1 + 2 + (*cursor_pos as u16).min(overlay_area.width - 5);
    let cursor_y = overlay_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

// ── 添加节点覆盖层 ─────────────────────────────────────────────────────────

fn render_add_node_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let AppMode::AddNode {
        parent_path,
        is_array,
        key_buffer,
        value_buffer,
        focus_on_key,
        key_cursor,
        value_cursor,
    } = &app.mode
    else {
        return;
    };

    let overlay_height = if *is_array { 4u16 } else { 5u16 };
    if area.height < overlay_height + 2 {
        return;
    }
    let overlay_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - overlay_height - 1,
        width: area.width.saturating_sub(2),
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);

    if *is_array {
        // 数组模式：只显示 value
        let value_cursor = *value_cursor;
        let value_display = format!("{value_buffer} ");
        let value_para = Paragraph::new(value_display)
            .block(
                Block::default()
                    .title(Span::styled(
                        " 添加到数组 ",
                        Style::default().fg(Color::Green),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::White));

        // 需要手动渲染来获取正确的区域
        let value_area = overlay_area;
        frame.render_widget(value_para, value_area);

        // 显示父路径提示
        let parent_hint = format!(" 父路径: {parent_path}");
        let hint_para = Paragraph::new(parent_hint).style(Style::default().fg(Color::DarkGray));
        let hint_area = Rect {
            x: value_area.x + 1,
            y: value_area.y + value_area.height - 1,
            width: value_area.width - 2,
            height: 1,
        };
        frame.render_widget(hint_para, hint_area);

        // 光标
        let cursor_x = value_area.x + 1 + (value_cursor as u16).min(value_area.width - 3);
        let cursor_y = value_area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    } else {
        // 对象模式：显示 key 和 value
        let key_cursor = *key_cursor;
        let value_cursor = *value_cursor;

        // key 行
        let key_label = if *focus_on_key { ">" } else { " " };
        let key_display = format!("{key_label} {key_buffer} ");
        let key_para = Paragraph::new(key_display)
            .block(
                Block::default()
                    .title(" key ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if *focus_on_key {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    })),
            )
            .style(Style::default().fg(Color::White));

        // value 行
        let value_label = if *focus_on_key { " " } else { ">" };
        let value_display = format!("{value_label} {value_buffer} ");
        let value_para = Paragraph::new(value_display)
            .block(
                Block::default()
                    .title(" value ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if *focus_on_key {
                        Color::DarkGray
                    } else {
                        Color::Yellow
                    })),
            )
            .style(Style::default().fg(Color::White));

        // 使用垂直布局
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(overlay_area);

        frame.render_widget(key_para, inner_chunks[0]);
        frame.render_widget(value_para, inner_chunks[1]);

        // 光标位置
        let (cursor_x, cursor_y, _max_chars) = if *focus_on_key {
            (
                inner_chunks[0].x
                    + 2
                    + (key_cursor as u16).min(inner_chunks[0].width.saturating_sub(4)),
                inner_chunks[0].y + 1,
                inner_chunks[0].width.saturating_sub(4),
            )
        } else {
            (
                inner_chunks[1].x
                    + 2
                    + (value_cursor as u16).min(inner_chunks[1].width.saturating_sub(4)),
                inner_chunks[1].y + 1,
                inner_chunks[1].width.saturating_sub(4),
            )
        };
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

// ── 确认覆盖层 ───────────────────────────────────────────────────────────────

fn render_confirm_overlay(frame: &mut Frame, area: Rect) {
    let overlay_height = 5u16;
    let overlay_width = 60u16;
    if area.height < overlay_height + 2 || area.width < overlay_width + 2 {
        return;
    }
    let overlay_area = Rect {
        x: area.x + (area.width - overlay_width) / 2,
        y: area.y + (area.height - overlay_height) / 2,
        width: overlay_width,
        height: overlay_height,
    };

    frame.render_widget(Clear, overlay_area);

    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  此文件含有注释（JSONC 格式）。",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "  保存后注释将被移除，是否继续？",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "  y 确认  /  n 取消",
            Style::default().fg(Color::White),
        )),
    ];

    let para = Paragraph::new(msg).block(
        Block::default()
            .title(" 注意 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(para, overlay_area);
}
