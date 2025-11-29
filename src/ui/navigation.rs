use crate::ui::state::AppState;

/// Trait for navigation functionality
pub trait NavigationHandling {
    /// Move selection to the next item in the list
    fn move_next(state: &mut AppState);
    /// Move selection to the previous item in the list
    fn move_previous(state: &mut AppState);
}

pub struct Navigation;

impl NavigationHandling for Navigation {
    /// Move selection to the next item in the list
    fn move_next(state: &mut AppState) {
        let i = match state.list_state.selected() {
            Some(i) => {
                if i >= state.todo_service.todos.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.list_state.select(Some(i));
    }

    /// Move selection to the previous item in the list
    fn move_previous(state: &mut AppState) {
        let i = match state.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    state.todo_service.todos.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.list_state.select(Some(i));
    }
}