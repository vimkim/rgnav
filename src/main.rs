mod rg_matches;

use rg_matches::get_rg_matches;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self};
use std::process::Command;

// Struct to ensure the terminal is restored on exit
struct TerminalCleanup;

impl TerminalCleanup {
    fn new() -> Result<Self> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(io::stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;
        Ok(Self)
    }
}

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    // Initialize the TerminalCleanup struct to manage terminal state

    // Enter alternate screen and enable raw mode
    enable_raw_mode().context("Failed to enable raw mode")?;

    let _cleanup = TerminalCleanup::new()?;
    let rg_matches = get_rg_matches()?;

    let mut terminal = setup_terminal()?;

    let mut selected_idx = 0;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.area());

            // Define highlight style for the selected item
            let highlight_style = Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);

            let items: Vec<ListItem> = rg_matches
                .iter()
                .map(|m| {
                    ListItem::new(
                        m.data
                            .as_ref()
                            .map(|data| data.path.text.clone())
                            .unwrap_or_default(),
                    )
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Search Results"),
                )
                .highlight_style(highlight_style); // Apply highlight style

            f.render_stateful_widget(list, chunks[0], &mut create_list_state(selected_idx));

            if let Some(data) = rg_matches.get(selected_idx).and_then(|m| m.data.as_ref()) {
                // Capture `bat` output for file preview with context around the match line
                let preview_text = get_file_preview(&data.path.text, data.line_number)
                    .unwrap_or_else(|_| "Error loading preview".into());

                let preview = Paragraph::new(preview_text)
                    .block(Block::default().borders(Borders::ALL).title("Code Preview"));
                f.render_widget(preview, chunks[1]);
            }
        })?;

        // Handle key events
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected_idx > 0 {
                            selected_idx -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_idx < rg_matches.len() - 1 {
                            selected_idx += 1;
                        }
                    }
                    KeyCode::Char('q') => break, // Exit on 'q' key
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

// Function to create the list state with the selected index
fn create_list_state(selected_idx: usize) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(selected_idx));
    state
}

use ansi_to_tui::IntoText;
use ratatui::text::Text;

// Maximum number of characters per line
const MAX_LINE_LENGTH: usize = 80;

// Function to get preview of file content around the specific line using `bat`
fn get_file_preview(file_path: &str, line_number: usize) -> Result<Text> {
    let start_line = if line_number > 15 {
        line_number - 15
    } else {
        1
    };
    let end_line = line_number + 15;

    // Use `bat` with color enabled
    let output = Command::new("bat")
        .args([
            "--style",
            "plain",
            "--paging",
            "never",
            "--color",
            "always", // Enable color for ANSI escape sequences
            "--line-range",
            &format!("{}:{}", start_line, end_line), // Context range around the match
            file_path,
        ])
        .output()
        .context("Failed to execute bat")?;

    if output.status.success() {
        // Process each line to truncate it to the max length
        let preview_text = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| {
                let mut truncated = line.to_string();
                truncated.truncate(MAX_LINE_LENGTH); // Limit each line length
                truncated
            })
            .collect::<Vec<_>>()
            .join("\n");

        preview_text
            .into_text()
            .map_err(|e| anyhow::anyhow!("Failed to parse ANSI: {}", e))
    } else {
        Err(anyhow::anyhow!(
            "Error running bat: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
