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

pub fn draw(f: &mut Frame, app: &App) {
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
        .block(Block::default().borders(Borders::ALL).title("Bezy TUI"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.current_tab)
        .divider("│");

    f.render_widget(tabs, area);
}

fn draw_tab_content(f: &mut Frame, app: &App, area: Rect) {
    match &app.tabs[app.current_tab].state {
        TabState::Glyphs(state) => {
            draw_glyphs_tab(f, app, state, area);
        }
        TabState::FontInfo => {
            draw_font_info_tab(f, app, area);
        }
        TabState::Logs(state) => {
            draw_logs_tab(f, app, state, area);
        }
        TabState::Help => {
            draw_help_tab(f, area);
        }
    }
}

fn draw_glyphs_tab(
    f: &mut Frame,
    app: &App,
    state: &crate::tui::tabs::glyphs::GlyphsState,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    // Glyph list
    let items: Vec<ListItem> = app
        .glyphs
        .iter()
        .enumerate()
        .map(|(i, glyph)| {
            let content = format!(
                "U+{:04X} {} {}",
                glyph.unicode.unwrap_or(0),
                glyph.codepoint,
                glyph.name.as_deref().unwrap_or("(unnamed)")
            );

            let style = if i == state.selected_index {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Glyphs"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(list, chunks[0]);

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

fn draw_font_info_tab(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(font_info) = &app.font_info {
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

fn draw_logs_tab(
    f: &mut Frame,
    app: &App,
    state: &crate::tui::tabs::logs::LogsState,
    area: Rect,
) {
    let visible_lines = area.height.saturating_sub(2) as usize; // Account for borders
    let total_lines = app.logs.len();

    let start_index = if state.auto_scroll && total_lines > visible_lines {
        total_lines - visible_lines
    } else {
        state.scroll_offset.min(total_lines.saturating_sub(visible_lines))
    };

    let end_index = (start_index + visible_lines).min(total_lines);

    let log_lines: Vec<ListItem> = app.logs[start_index..end_index]
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
            Span::styled("Glyphs Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  ↑/↓ or j/k     - Navigate glyph list"),
        Line::from("  Page Up/Down   - Navigate by page"),
        Line::from("  Enter          - Select glyph in editor"),
        Line::from("  /              - Search glyphs"),
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