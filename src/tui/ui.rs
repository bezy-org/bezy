use crate::tui::{
    app::App,
    tabs::TabState,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    // Draw tab bar
    draw_tabs(f, app, chunks[0]);

    // Draw current tab content
    draw_tab_content(f, app, chunks[1]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let title = format!("{}. {}", i + 1, tab.tab_type.title());
            Line::from(title)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Bezy"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.current_tab)
        .divider("│");

    f.render_widget(tabs, area);
}

fn draw_tab_content(f: &mut Frame, app: &mut App, area: Rect) {
    let current_tab_idx = app.current_tab;
    let glyphs = app.glyphs.clone(); // Clone the data to avoid borrowing issues
    let font_info = app.font_info.clone();
    let logs = app.logs.clone();

    match &mut app.tabs[current_tab_idx].state {
        TabState::Codepoints(state) => {
            draw_codepoints_tab_with_data(f, &glyphs, state, area);
        }
        TabState::FontInfo => {
            draw_font_info_tab_with_data(f, &font_info, area);
        }
        TabState::Logs(state) => {
            draw_logs_tab_with_data(f, &logs, state, area);
        }
        TabState::Help => {
            draw_help_tab(f, area);
        }
    }
}

fn draw_codepoints_tab_with_data(
    f: &mut Frame,
    glyphs: &[crate::tui::communication::GlyphInfo],
    state: &mut crate::tui::tabs::glyphs::GlyphsState,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    // Calculate scroll position to keep selected item visible
    let visible_lines = chunks[0].height.saturating_sub(2) as usize; // Account for borders

    // Update scroll offset if selection is outside visible area
    if state.selected_index < state.scroll_offset {
        state.scroll_offset = state.selected_index;
    } else if state.selected_index >= state.scroll_offset + visible_lines {
        state.scroll_offset = state.selected_index.saturating_sub(visible_lines - 1);
    }

    // Create list items with proper scrolling
    let items: Vec<ListItem> = glyphs
        .iter()
        .enumerate()
        .map(|(i, glyph)| {
            let content = if let Some(name) = &glyph.name {
                if name == &glyph.codepoint {
                    // Don't show duplicate name
                    format!("U+{:04X} {}", glyph.unicode.unwrap_or(0), glyph.codepoint)
                } else {
                    // Show both codepoint and different name
                    format!("U+{:04X} {} ({})", glyph.unicode.unwrap_or(0), glyph.codepoint, name)
                }
            } else {
                format!("U+{:04X} {}", glyph.unicode.unwrap_or(0), glyph.codepoint)
            };

            let style = if i == state.selected_index {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Codepoints"));

    // Create list state for proper scrolling
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.selected_index));

    f.render_stateful_widget(list, chunks[0], &mut list_state);

    // Status/search bar
    let status_text = if state.is_searching {
        format!("Search: {}", state.search_query)
    } else {
        format!(
            "Selected: {} | Use ↑↓ or j/k to navigate, Enter to select, / to search",
            state.selected_index
        )
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));

    f.render_widget(status, chunks[1]);
}

fn draw_font_info_tab_with_data(f: &mut Frame, font_info: &Option<crate::tui::communication::FontInfo>, area: Rect) {
    let content = if let Some(font_info) = font_info {
        let mut text = Vec::new();

        if let Some(family) = &font_info.family_name {
            text.push(Line::from(vec![
                Span::styled("Family: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(family),
            ]));
        }

        if let Some(style) = &font_info.style_name {
            text.push(Line::from(vec![
                Span::styled("Style: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(style),
            ]));
        }

        if let Some(version) = &font_info.version {
            text.push(Line::from(vec![
                Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(version),
            ]));
        }

        if let Some(units_per_em) = font_info.units_per_em {
            text.push(Line::from(vec![
                Span::styled("Units per EM: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", units_per_em)),
            ]));
        }

        if let Some(ascender) = font_info.ascender {
            text.push(Line::from(vec![
                Span::styled("Ascender: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", ascender)),
            ]));
        }

        if let Some(descender) = font_info.descender {
            text.push(Line::from(vec![
                Span::styled("Descender: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", descender)),
            ]));
        }

        Text::from(text)
    } else {
        Text::from("No font information available")
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Font Information"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_logs_tab_with_data(
    f: &mut Frame,
    logs: &[String],
    state: &crate::tui::tabs::logs::LogsState,
    area: Rect,
) {
    let visible_lines = area.height.saturating_sub(2) as usize; // Account for borders
    let total_lines = logs.len();

    let start_index = if state.auto_scroll && total_lines > visible_lines {
        total_lines - visible_lines
    } else {
        state.scroll_offset.min(total_lines.saturating_sub(visible_lines))
    };

    let end_index = (start_index + visible_lines).min(total_lines);

    let log_lines: Vec<ListItem> = logs[start_index..end_index]
        .iter()
        .map(|line| ListItem::new(line.as_str()))
        .collect();

    let logs = List::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Logs ({}/{} lines{})",
            end_index - start_index,
            total_lines,
            if state.auto_scroll { " - auto-scroll" } else { "" }
        )));

    f.render_widget(logs, area);
}

fn draw_help_tab(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("Global Controls:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  Ctrl+Q         - Quit application"),
        Line::from("  Tab            - Next tab"),
        Line::from("  Shift+Tab      - Previous tab"),
        Line::from("  1-4            - Jump to tab by number"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Codepoints Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  ↑/↓ or j/k     - Navigate codepoint list"),
        Line::from("  Page Up/Down   - Navigate by page"),
        Line::from("  Enter          - Select codepoint in editor"),
        Line::from("  /              - Search codepoints"),
        Line::from("  Esc            - Exit search"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Logs Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  ↑/↓ or j/k     - Scroll logs"),
        Line::from("  Home           - Jump to top"),
        Line::from("  End            - Jump to bottom (auto-scroll)"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Font Info Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  (Read-only display)"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}