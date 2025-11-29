use crate::ui::state::{AppState, InputMode};
use crate::ui::navigation::{Navigation, NavigationHandling};
use crossterm::event::{KeyCode, KeyEvent};

/// Trait for handling user input events
pub trait EventHandling {
    fn handle_key_event(state: &mut AppState, key: KeyEvent) -> Result<(), std::io::Error>;
}

pub struct EventHandler;

impl EventHandling for EventHandler {
    fn handle_key_event(state: &mut AppState, key: KeyEvent) -> Result<(), std::io::Error> {
        match state.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => state.should_quit = true,
                KeyCode::Down | KeyCode::Char('j') => {
                    Navigation::move_next(state);
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    Navigation::move_previous(state);
                },
                KeyCode::Char(' ') => state.toggle_current_todo(),
                KeyCode::Char('a') | KeyCode::Char('n') => state.start_adding_todo(),
                KeyCode::Char('d') | KeyCode::Char('x') => state.delete_current_todo(),
                _ => {}
            },
            InputMode::Adding => match key.code {
                KeyCode::Enter => state.add_todo(),
                KeyCode::Esc => state.cancel_adding_todo(),
                KeyCode::Backspace => {
                    state.input_text.pop();
                }
                KeyCode::Char(c) => {
                    state.input_text.push(c);
                }
                _ => {}
            },
        }
        Ok(())
    }
}