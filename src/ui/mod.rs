//! UI module for the music player terminal interface.
//!
//! This module provides a modern, responsive terminal user interface with:
//! - Three-layer vertical layout (now playing, main content, playback controls)
//! - Responsive design that adapts to terminal size
//! - Custom widgets for different UI components
//! - Configurable color themes
//!
//! # Architecture
//!
//! The UI is organized into three main submodules:
//! - `layout`: Handles layout calculation and responsive behavior
//! - `widgets`: Custom widget components for different UI regions
//! - `theme`: Color theme and styling system

use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

// Export UI submodules
pub mod layout;
pub mod widgets;
pub mod theme;

/// Initializes the terminal for TUI rendering.
///
/// This function:
/// - Enters alternate screen mode to preserve terminal history
/// - Creates a new terminal instance with crossterm backend
///
/// # Returns
///
/// Returns a `Result` containing the initialized terminal or an error.
///
/// # Errors
///
/// Returns an error if terminal initialization fails.
pub fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let stdout = std::io::stdout();
    execute!(stdout.lock(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// This function:
/// - Shows the cursor
/// - Leaves alternate screen mode
///
/// # Arguments
///
/// * `terminal` - Mutable reference to the terminal to restore
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
///
/// # Errors
///
/// Returns an error if terminal restoration fails.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.show_cursor()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
