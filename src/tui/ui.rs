use crate::tui::{app::App, tabs::TabState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs},
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
            let title = format!("{}.{}", i + 1, tab.tab_type.title());
            Line::from(title)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Bezy"))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.current_tab)
        .divider("│");

    f.render_widget(tabs, area);
}

fn draw_tab_content(f: &mut Frame, app: &mut App, area: Rect) {
    let current_tab_idx = app.current_tab;
    let glyphs = app.glyphs.clone(); // Clone the data to avoid borrowing issues

    match &mut app.tabs[current_tab_idx].state {
        TabState::File(state) => {
            crate::tui::tabs::file::draw(f, state, area);
        }
        TabState::Edit(state) => {
            crate::tui::tabs::edit::draw(f, state, area);
        }
        TabState::Unicode(state) => {
            crate::tui::tabs::unicode::draw(f, &glyphs, state, area);
        }
        TabState::FontInfo(state) => {
            crate::tui::tabs::font_info::draw(f, state, area);
        }
        TabState::QA(state) => {
            crate::tui::tabs::qa::draw(f, state, area);
        }
        TabState::Glyph(state) => {
            crate::tui::tabs::glyph::draw(f, state, area);
        }
        TabState::Path(state) => {
            crate::tui::tabs::path::draw(f, state, area);
        }
        TabState::AI(state) => {
            crate::tui::tabs::ai::draw(f, state, area);
        }
        TabState::Help(state) => {
            crate::tui::tabs::help::draw(f, state, area);
        }
    }
}
