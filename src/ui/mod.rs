use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::Terminal;

pub fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let stdout = std::io::stdout();
    execute!(stdout.lock(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.show_cursor()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

pub fn list_state(selected: usize) -> ratatui::widgets::ListState {
    let mut s = ratatui::widgets::ListState::default();
    s.select(Some(selected));
    s
}
