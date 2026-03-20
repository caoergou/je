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

use super::app::{App, AppMode, ContextAction, StatusLevel};
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

    // 编辑覆盖层（值）
    if matches!(app.mode, AppMode::Edit { .. }) {
        render_edit_overlay(frame, app, area);
    }

    // 编辑覆盖层（键名）
    if matches!(app.mode, AppMode::EditKey { .. }) {
        render_edit_key_overlay(frame, app, area);
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

    // 右键菜单覆盖层
    if matches!(app.mode, AppMode::ContextMenu { .. }) {
        render_context_menu(frame, app, area);
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
                " Alt:菜单 ↑↓:移动 ←:折叠 →/Space:展开 Enter:编辑 N:新建 Del:删除 Ctrl+S:保存 Ctrl+F:搜索 "
            }
            AppMode::Edit { .. } => " 输入值  Enter:确认  Esc:取消",
            AppMode::EditKey { .. } => " 输入新键名  Enter:确认  Esc:取消",
            AppMode::Search { .. } => " 输入搜索  Enter:跳转下一匹配  Esc:退出",
            AppMode::AddNode { .. } => " 输入字段名  Enter:确认  Esc:取消",
            AppMode::ConfirmStripComments => " [Y]:确认保存  [N]:取消  ",
            AppMode::ContextMenu { .. } => " ↑↓:选择  Enter:执行  F2:菜单  Esc:退出",
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
        value_type,
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
    let title = format!(" 编辑 {value_type} - {path} ");

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

fn render_edit_key_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let AppMode::EditKey {
        path,
        buffer,
        cursor_pos,
        ..
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
    let title = format!(" 重命名键 {path} ");

    let para = Paragraph::new(display_buf)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
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
        is_array: _,
        key_buffer,
        key_cursor,
    } = &app.mode
    else {
        return;
    };

    // 对象模式：只输入 key
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

    let display_buf = format!(" {key_buffer} ");
    let title = format!(" 添加字段到 {parent_path} ");

    let para = Paragraph::new(display_buf)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(Color::Green)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(para, overlay_area);

    // 光标位置
    let cursor_x = overlay_area.x + 1 + (*key_cursor as u16).min(overlay_area.width - 3);
    let cursor_y = overlay_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

// ── 确认覆盖层 ───────────────────────────────────────────────────────────────

fn render_confirm_overlay(frame: &mut Frame, area: Rect) {
    let overlay_height = 6u16;
    let overlay_width = 50u16;
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

    // 带按钮的确认框
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
        Line::from(""),
        Line::from(vec![
            Span::styled("   [ Y ] 确认    [ N ] 取消   ", Style::default().fg(Color::White)),
        ]),
    ];

    let para = Paragraph::new(msg).block(
        Block::default()
            .title(" 注意 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(para, overlay_area);
}

// ── 右键菜单覆盖层 ───────────────────────────────────────────────────────────

fn render_context_menu(frame: &mut Frame, app: &App, area: Rect) {
    let AppMode::ContextMenu { selected, mouse_x, mouse_y, .. } = &app.mode else {
        return;
    };

    let actions = ContextAction::all();
    let menu_width = 28u16;
    let menu_height = actions.len() as u16 + 2;

    // 菜单位置：鼠标点击位置（减去一些偏移让菜单在点击位置下方/旁边）
    let menu_x = (*mouse_x as u16).saturating_sub(2).min(area.width.saturating_sub(menu_width + 2));
    let menu_y = (*mouse_y as u16).min(area.height.saturating_sub(menu_height + 2));

    if area.height < menu_y + menu_height + 2 || area.width < menu_x + menu_width + 2 {
        return;
    }

    let overlay_area = Rect {
        x: area.x + menu_x,
        y: area.y + menu_y,
        width: menu_width,
        height: menu_height,
    };

    // 填充不透明背景
    let bg = Paragraph::new(" ").style(Style::default().bg(Color::Black));
    frame.render_widget(bg, overlay_area);

    // 悬停效果优先于键盘选中
    let hover_row = app.menu_hover_row;

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_hovered = hover_row == Some(i);
            let is_selected = !hover_row.is_some() && i == *selected;

            let style = if is_hovered {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(action.label(), style))
        })
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .title(" Actions ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black))
        .highlight_style(Style::default());

    frame.render_widget(menu, overlay_area);
}
