use crate::services::todo_service::TodoService;
use ratatui::widgets::ListState;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Adding,
}

pub struct AppState {
    pub todo_service: TodoService,
    pub list_state: ListState,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input_text: String,
}

impl AppState {
    pub fn new() -> AppState {
        let mut state = AppState {
            todo_service: TodoService::new(),
            list_state: ListState::default(),
            should_quit: false,
            input_mode: InputMode::Normal,
            input_text: String::new(),
        };

        // Initialize list state to select the first item if available
        if !state.todo_service.todos.is_empty() {
            state.list_state.select(Some(0));
        }

        state
    }

    pub fn toggle_current_todo(&mut self) {
        if let Some(current_index) = self.list_state.selected() {
            if current_index < self.todo_service.todos.len() {
                let todo = &mut self.todo_service.todos[current_index];
                todo.completed = !todo.completed;
                self.todo_service.save_to_file();
            }
        }
    }

    pub fn add_todo(&mut self) {
        if !self.input_text.trim().is_empty() {
            self.todo_service.add_todo(&self.input_text);

            // Update the list state to select the newly added item
            if !self.todo_service.todos.is_empty() {
                let new_index = self.todo_service.todos.len() - 1;
                self.list_state.select(Some(new_index));
            }
        }

        self.input_mode = InputMode::Normal;
        self.input_text.clear();
    }

    pub fn start_adding_todo(&mut self) {
        self.input_mode = InputMode::Adding;
        self.input_text.clear();
    }

    pub fn cancel_adding_todo(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_text.clear();
    }

    pub fn delete_current_todo(&mut self) {
        if let Some(current_index) = self.list_state.selected() {
            if current_index < self.todo_service.todos.len() {
                let todo_id = self.todo_service.todos[current_index].id;
                self.todo_service.delete_todo(todo_id);

                // Update the list state after deletion
                if !self.todo_service.todos.is_empty() {
                    // If we deleted the last item and the list isn't empty, select the new last item
                    if current_index >= self.todo_service.todos.len() {
                        self.list_state
                            .select(Some(self.todo_service.todos.len().saturating_sub(1)));
                    } else {
                        // Otherwise, select the item at the current index (which now contains the next item)
                        self.list_state.select(Some(current_index));
                    }
                } else {
                    // If the list is now empty, clear the selection
                    self.list_state.select(None);
                }
            }
        }
    }
}