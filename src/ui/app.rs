use crate::ui::state::AppState;
use crate::ui::renderer::{Renderer, UiRenderer};
use crate::ui::event_handler::{EventHandler, EventHandling};
use crate::ui::UiError;
use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;

pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> App {
        App {
            state: AppState::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), UiError> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main loop
        loop {
            terminal.draw(|f| Renderer::draw(f, &mut self.state))?;
            if let event::Event::Key(key) = event::read()? {
                EventHandler::handle_key_event(&mut self.state, key)?;
                if self.state.should_quit {
                    break;
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}