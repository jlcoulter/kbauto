//! TUI missing-value form using ratatui.
//!
//! Renders a scrollable form for the user to fill in missing placeholder values.
//! On submit (Enter), returns the entered values. On cancel (Esc/q), returns
//! an error to abort generation.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::collections::HashMap;
use std::io;

use kbauto_template::{MissingValue, MissingValueForm};

/// Error type for TUI operations.
#[derive(Debug)]
pub enum TuiError {
    /// The user cancelled the form (pressed Esc or q).
    Cancelled,
    /// A terminal I/O error occurred.
    Io(io::Error),
}

impl From<io::Error> for TuiError {
    fn from(e: io::Error) -> Self {
        TuiError::Io(e)
    }
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuiError::Cancelled => write!(f, "Form cancelled by user"),
            TuiError::Io(e) => write!(f, "TUI I/O error: {e}"),
        }
    }
}

impl std::error::Error for TuiError {}

/// Run the missing-value TUI form.
///
/// Displays a scrollable list of missing placeholder keys with descriptions
/// and text input fields. On submit, returns the entered values as a HashMap.
/// On cancel, returns `TuiError::Cancelled`.
pub fn run_missing_value_form(missing: Vec<MissingValue>) -> Result<HashMap<String, String>, TuiError> {
    if missing.is_empty() {
        return Ok(HashMap::new());
    }

    // Setup terminal
    enable_raw_mode().map_err(TuiError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(TuiError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(TuiError::Io)?;

    let result = run_app(&mut terminal, &missing);

    // Restore terminal
    disable_raw_mode().map_err(TuiError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(TuiError::Io)?;
    terminal.show_cursor().map_err(TuiError::Io)?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    missing: &[MissingValue],
) -> Result<HashMap<String, String>, TuiError> {
    let mut form = MissingValueForm::new(missing.to_vec());
    let mut current_index: usize = 0;
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let area = f.area();

            // Layout: header (3 rows) | list (rest - 3 rows) | footer (3 rows)
            let chunks = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(area);

            // Header
            let header = Paragraph::new("Missing Values — Fill in required placeholders")
                .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
                .block(Block::default().borders(Borders::BOTTOM));
            f.render_widget(header, chunks[0]);

            // List of missing values with current input
            let items: Vec<ListItem> = missing
                .iter()
                .enumerate()
                .map(|(i, mv)| {
                    let value = form.values.get(&mv.key).cloned().unwrap_or_default();
                    let default_hint = mv
                        .default
                        .as_ref()
                        .map(|d| format!(" (default: {d})"))
                        .unwrap_or_default();
                    let style = if i == current_index {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let line = format!("{}: {}{}", mv.key, value, if value.is_empty() && mv.default.is_some() { default_hint } else { String::new() });
                    ListItem::new(line).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::NONE).title("Fields (Tab/Shift+Tab to navigate, Esc to cancel)"))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Footer
            let footer = Paragraph::new("Enter: Submit | Esc/q: Cancel | Tab: Next field")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        }).map_err(TuiError::Io)?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100)).map_err(TuiError::Io)? {
            if let Event::Key(key) = event::read().map_err(TuiError::Io)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Err(TuiError::Cancelled);
                    }
                    KeyCode::Enter => {
                        // Submit — fill in defaults for any empty fields
                        let resolved = form.resolve();
                        return Ok(resolved);
                    }
                    KeyCode::Tab => {
                        if !missing.is_empty() {
                            current_index = (current_index + 1) % missing.len();
                            list_state.select(Some(current_index));
                        }
                    }
                    KeyCode::BackTab => {
                        if !missing.is_empty() {
                            current_index = if current_index == 0 {
                                missing.len() - 1
                            } else {
                                current_index - 1
                            };
                            list_state.select(Some(current_index));
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(mv) = missing.get(current_index) {
                            if let Some(val) = form.values.get_mut(&mv.key) {
                                val.pop();
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(mv) = missing.get(current_index) {
                            form.values
                                .entry(mv.key.clone())
                                .and_modify(|v| v.push(c))
                                .or_insert_with(|| c.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}