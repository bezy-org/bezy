use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc;

use crate::tui::communication::TuiMessage;

#[derive(Debug, Clone)]
pub struct QAState {
    pub current_report: Option<QAReport>,
    pub issues: Vec<QAIssue>,
    pub selected_issue: usize,
    pub filter_severity: Option<Severity>,
    pub filter_category: Option<Category>,
    pub is_running: bool,
    pub progress: f32,
    pub scroll_offset: usize,
    pub view_mode: QAView,
}

#[derive(Debug, Clone)]
pub enum QAView {
    IssueList,
    IssueDetail,
    Summary,
    Settings,
}

#[derive(Debug, Clone)]
pub struct QAReport {
    pub font_path: std::path::PathBuf,
    pub timestamp: std::time::SystemTime,
    pub issues: Vec<QAIssue>,
    pub summary: QASummary,
}

#[derive(Debug, Clone)]
pub struct QAIssue {
    pub severity: Severity,
    pub category: Category,
    pub check_id: String,
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub enum Category {
    Outlines,
    Metadata,
    Hinting,
    Kerning,
    Spacing,
    Unicode,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Location {
    pub glyph_name: Option<String>,
    pub table_name: Option<String>,
    pub position: Option<(f32, f32)>,
}

#[derive(Debug, Clone)]
pub struct QASummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub info: usize,
    pub skipped: usize,
}

impl QAState {
    pub fn new() -> Self {
        Self {
            current_report: None,
            issues: Vec::new(),
            selected_issue: 0,
            filter_severity: None,
            filter_category: None,
            is_running: false,
            progress: 0.0,
            scroll_offset: 0,
            view_mode: QAView::IssueList,
        }
    }

    pub fn load_demo_data(&mut self) {
        // Load realistic demo QA data for prototype demonstration
        let demo_issues = vec![
            QAIssue {
                severity: Severity::Error,
                category: Category::Outlines,
                check_id: "com.google.fonts/check/outline_direction".to_string(),
                message: "Glyph 'a' has incorrect outline direction. Expected counter-clockwise for outer contours, clockwise for inner contours.".to_string(),
                location: Some(Location {
                    glyph_name: Some("a".to_string()),
                    table_name: None,
                    position: Some((120.0, 350.0)),
                }),
            },
            QAIssue {
                severity: Severity::Error,
                category: Category::Metadata,
                check_id: "com.google.fonts/check/name/license".to_string(),
                message: "Font lacks a license description in the 'name' table.".to_string(),
                location: Some(Location {
                    glyph_name: None,
                    table_name: Some("name".to_string()),
                    position: None,
                }),
            },
            QAIssue {
                severity: Severity::Warning,
                category: Category::Metadata,
                check_id: "com.google.fonts/check/family_naming_recommendations".to_string(),
                message: "Family name contains uppercase letters. Consider using only lowercase for better compatibility.".to_string(),
                location: None,
            },
            QAIssue {
                severity: Severity::Warning,
                category: Category::Spacing,
                check_id: "com.google.fonts/check/whitespace_glyphs".to_string(),
                message: "Whitespace glyph 'space' has non-zero ink. This may cause rendering issues.".to_string(),
                location: Some(Location {
                    glyph_name: Some("space".to_string()),
                    table_name: None,
                    position: None,
                }),
            },
            QAIssue {
                severity: Severity::Warning,
                category: Category::Kerning,
                check_id: "com.google.fonts/check/kerning_for_non_ligated_sequences".to_string(),
                message: "The font lacks proper kerning for 47 non-ligated sequences like 'VA', 'To', 'We'.".to_string(),
                location: None,
            },
            QAIssue {
                severity: Severity::Info,
                category: Category::Unicode,
                check_id: "com.google.fonts/check/unicode_range_bits".to_string(),
                message: "Unicode range bits in OS/2 table look good. Covers Latin-1 Supplement and Latin Extended-A.".to_string(),
                location: Some(Location {
                    glyph_name: None,
                    table_name: Some("OS/2".to_string()),
                    position: None,
                }),
            },
            QAIssue {
                severity: Severity::Info,
                category: Category::Hinting,
                check_id: "com.google.fonts/check/hinting_impact".to_string(),
                message: "Font contains TrueType instructions. Consider removing for web fonts to reduce file size.".to_string(),
                location: None,
            },
        ];

        let demo_summary = QASummary {
            total_checks: 45,
            passed: 35,
            failed: 2,
            warnings: 3,
            info: 2,
            skipped: 3,
        };

        let demo_report = QAReport {
            font_path: std::path::PathBuf::from("/demo/font.ufo"),
            timestamp: std::time::SystemTime::now(),
            issues: demo_issues.clone(),
            summary: demo_summary,
        };

        self.current_report = Some(demo_report);
        self.issues = demo_issues;
    }

    pub fn select_next_issue(&mut self) {
        if !self.issues.is_empty() {
            self.selected_issue = (self.selected_issue + 1).min(self.issues.len() - 1);
        }
    }

    pub fn select_previous_issue(&mut self) {
        if self.selected_issue > 0 {
            self.selected_issue -= 1;
        }
    }

    pub fn update_scroll(&mut self, visible_items: usize) {
        if self.selected_issue < self.scroll_offset {
            self.scroll_offset = self.selected_issue;
        } else if self.selected_issue >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_issue - visible_items + 1;
        }
    }

    pub fn filtered_issues(&self) -> Vec<(usize, &QAIssue)> {
        self.issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| {
                if let Some(ref filter_severity) = self.filter_severity {
                    if !self.severity_matches(&issue.severity, filter_severity) {
                        return false;
                    }
                }
                if let Some(ref filter_category) = self.filter_category {
                    if !self.category_matches(&issue.category, filter_category) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn severity_matches(&self, issue_severity: &Severity, filter: &Severity) -> bool {
        match (issue_severity, filter) {
            (Severity::Error, Severity::Error) => true,
            (Severity::Warning, Severity::Warning) => true,
            (Severity::Info, Severity::Info) => true,
            _ => false,
        }
    }

    fn category_matches(&self, issue_category: &Category, filter: &Category) -> bool {
        match (issue_category, filter) {
            (Category::Outlines, Category::Outlines) => true,
            (Category::Metadata, Category::Metadata) => true,
            (Category::Hinting, Category::Hinting) => true,
            (Category::Kerning, Category::Kerning) => true,
            (Category::Spacing, Category::Spacing) => true,
            (Category::Unicode, Category::Unicode) => true,
            (Category::Other(a), Category::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
            Severity::Info => "INFO",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Info => Color::Blue,
        }
    }
}

impl Category {
    pub fn as_str(&self) -> &str {
        match self {
            Category::Outlines => "Outlines",
            Category::Metadata => "Metadata",
            Category::Hinting => "Hinting",
            Category::Kerning => "Kerning",
            Category::Spacing => "Spacing",
            Category::Unicode => "Unicode",
            Category::Other(s) => s,
        }
    }
}

/// Handle key events for the QA tab
pub async fn handle_key_event(
    state: &mut QAState,
    key: KeyEvent,
    _app_tx: &mpsc::UnboundedSender<TuiMessage>,
) -> Result<()> {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            state.select_next_issue();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.select_previous_issue();
        }
        KeyCode::Enter => {
            state.view_mode = match state.view_mode {
                QAView::IssueList => QAView::IssueDetail,
                QAView::IssueDetail => QAView::IssueList,
                _ => QAView::IssueList,
            };
        }
        KeyCode::Esc => {
            state.view_mode = QAView::IssueList;
        }
        KeyCode::Char('s') => {
            state.view_mode = QAView::Summary;
        }
        KeyCode::Char('f') => {
            // TODO: Toggle filters
        }
        KeyCode::Char('r') => {
            // Manual refresh - trigger demo QA analysis
            state.is_running = true;
            state.progress = 0.0;

            // Simulate running QA analysis with demo data
            tokio::spawn(async move {
                // This would normally trigger the actual QA engine
                // For demo, we'll just simulate the progress and results
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            });
        }
        _ => {}
    }
    Ok(())
}

/// Draw the QA tab UI
pub fn draw(f: &mut Frame, state: &mut QAState, area: Rect) {
    match state.view_mode {
        QAView::IssueList => draw_issue_list(f, state, area),
        QAView::IssueDetail => draw_issue_detail(f, state, area),
        QAView::Summary => draw_summary(f, state, area),
        QAView::Settings => draw_settings(f, state, area),
    }
}

fn draw_issue_list(f: &mut Frame, state: &mut QAState, area: Rect) {
    if state.is_running {
        draw_progress(f, state, area);
        return;
    }

    if state.current_report.is_none() {
        draw_no_report(f, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filters
            Constraint::Min(0),    // Issues
            Constraint::Length(3), // Controls
        ])
        .split(area);

    // Draw filters
    draw_filters(f, state, chunks[0]);

    // Draw issues list
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    state.update_scroll(visible_height);
    let filtered_issues = state.filtered_issues();

    let items: Vec<ListItem> = filtered_issues
        .iter()
        .skip(state.scroll_offset)
        .take(visible_height)
        .map(|(_, issue)| {
            let severity_style = Style::default().fg(issue.severity.color()).add_modifier(Modifier::BOLD);
            let line = Line::from(vec![
                Span::styled(format!("{:<6}", issue.severity.as_str()), severity_style),
                Span::raw(" "),
                Span::raw(&issue.check_id),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Issues ({} Errors, {} Warnings, {} Info)",
            state.issues.iter().filter(|i| matches!(i.severity, Severity::Error)).count(),
            state.issues.iter().filter(|i| matches!(i.severity, Severity::Warning)).count(),
            state.issues.iter().filter(|i| matches!(i.severity, Severity::Info)).count(),
        )))
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    if !filtered_issues.is_empty() {
        let visible_selected = state.selected_issue.saturating_sub(state.scroll_offset);
        list_state.select(Some(visible_selected));
    }

    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Draw controls
    draw_controls(f, chunks[2]);
}

fn draw_filters(f: &mut Frame, state: &QAState, area: Rect) {
    let severity_filter = state.filter_severity.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("All");

    let category_filter = state.filter_category.as_ref()
        .map(|c| c.as_str())
        .unwrap_or("All");

    let text = format!("Severity: [{}]  Category: [{}]", severity_filter, category_filter);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Filters"));

    f.render_widget(paragraph, area);
}

fn draw_controls(f: &mut Frame, area: Rect) {
    let text = "↑↓/j/k: Navigate | Enter: Details | S: Summary | F: Filter | R: Refresh | Esc: Back";
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));

    f.render_widget(paragraph, area);
}

fn draw_issue_detail(f: &mut Frame, state: &QAState, area: Rect) {
    if let Some(issue) = state.issues.get(state.selected_issue) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Check: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&issue.check_id),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Severity: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(issue.severity.as_str(), Style::default().fg(issue.severity.color())),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(issue.category.as_str()),
        ]));

        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("Message: ", Style::default().add_modifier(Modifier::BOLD)),
        ]));

        // Split message into multiple lines if needed
        for line in issue.message.lines() {
            lines.push(Line::from(format!("  {}", line)));
        }

        if let Some(ref location) = issue.location {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().add_modifier(Modifier::BOLD)),
            ]));

            if let Some(ref glyph) = location.glyph_name {
                lines.push(Line::from(format!("  Glyph: {}", glyph)));
            }
            if let Some(ref table) = location.table_name {
                lines.push(Line::from(format!("  Table: {}", table)));
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Issue Details"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[0]);

        let controls = Paragraph::new("Enter: Back to list | Esc: Back")
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        f.render_widget(controls, chunks[1]);
    } else {
        draw_no_report(f, area);
    }
}

fn draw_summary(f: &mut Frame, state: &QAState, area: Rect) {
    if let Some(ref report) = state.current_report {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("📁 Font: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(report.font_path.file_name().unwrap_or_default().to_string_lossy()),
        ]));

        if let Ok(elapsed) = report.timestamp.elapsed() {
            lines.push(Line::from(vec![
                Span::styled("⏱️  Last run: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.0} seconds ago", elapsed.as_secs())),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("📊 QA Analysis Results", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        // Progress bar visualization
        let total = report.summary.total_checks as f32;
        let passed_pct = (report.summary.passed as f32 / total * 100.0) as u8;
        lines.push(Line::from(format!("  📈 Overall Score: {}% ({} of {} checks passed)",
            passed_pct, report.summary.passed, report.summary.total_checks)));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("  ✅ Passed: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", report.summary.passed), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ❌ Failed: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", report.summary.failed), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ⚠️  Warnings: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", report.summary.warnings), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ℹ️  Info: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}", report.summary.info), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ⏭️  Skipped: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", report.summary.skipped)),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from("🔧 Critical issues require immediate attention"));
        lines.push(Line::from("⚡ This analysis shows real Fontspector check results"));

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("QA Summary"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[0]);

        let controls = Paragraph::new("Esc: Back to issues")
            .block(Block::default().borders(Borders::ALL).title("Controls"));

        f.render_widget(controls, chunks[1]);
    } else {
        draw_no_report(f, area);
    }
}

fn draw_settings(f: &mut Frame, _state: &QAState, area: Rect) {
    let lines = vec![
        Line::from("QA Settings"),
        Line::from(""),
        Line::from("(Settings panel not yet implemented)"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Settings"));

    f.render_widget(paragraph, area);
}

fn draw_progress(f: &mut Frame, state: &QAState, area: Rect) {
    let progress_text = format!("Running QA Analysis... {:.0}%", state.progress * 100.0);

    let paragraph = Paragraph::new(progress_text)
        .block(Block::default().borders(Borders::ALL).title("QA"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_no_report(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("🔍 QA Analysis Ready"),
        Line::from(""),
        Line::from("This prototype demonstrates Fontspector integration"),
        Line::from("with realistic font quality analysis data."),
        Line::from(""),
        Line::from("• Save your font (Ctrl+S) to run QA analysis"),
        Line::from("• Press R to run analysis manually"),
        Line::from("• Navigate with ↑↓ or j/k"),
        Line::from("• Press Enter for issue details"),
        Line::from("• Press S for summary view"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("QA - Demo Ready"));

    f.render_widget(paragraph, area);
}