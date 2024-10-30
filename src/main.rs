use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use serde::Deserialize;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
struct RgMatch {
    data: Option<MatchData>,
}

#[derive(Debug, Deserialize)]
struct MatchData {
    path: PathInfo,
    lines: LineInfo,
}

#[derive(Debug, Deserialize)]
struct PathInfo {
    text: String,
}

#[derive(Debug, Deserialize)]
struct LineInfo {
    text: String,
}

fn main() -> Result<()> {
    // Enter alternate screen and enable raw mode
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All))
        .context("Failed to enter alternate screen")?;

    let mut terminal = setup_terminal()?;
    let rg_matches = get_rg_matches()?;

    let mut selected_idx = 0;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(f.area());

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
                .highlight_style(Style::default().bg(Color::LightGreen));

            f.render_widget(list, chunks[0]);

            if let Some(data) = rg_matches.get(selected_idx).and_then(|m| m.data.as_ref()) {
                let preview = Paragraph::new(data.lines.text.clone())
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

    // Restore the terminal to its previous state
    restore_terminal()?;
    execute!(stdout, LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")
}

fn get_rg_matches() -> Result<Vec<RgMatch>> {
    let rg_command = Command::new("rg")
        .args(["--json", "fn"]) // Customize search term
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start ripgrep")?;

    let rg_output = rg_command
        .stdout
        .context("Failed to capture ripgrep output")?;
    let rg_reader = io::BufReader::new(rg_output);

    let mut matches = Vec::new();
    for line in rg_reader.lines() {
        let line = line.context("Failed to read line from ripgrep output")?;
        if let Ok(rg_match) = serde_json::from_str::<RgMatch>(&line) {
            matches.push(rg_match);
        }
    }

    Ok(matches)
}
