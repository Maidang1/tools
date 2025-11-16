use crate::services::todo_service::TodoService;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use std::io::{self, stdout};

pub struct App {
    pub todo_service: TodoService,
    pub list_state: ListState,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            todo_service: TodoService::new(),
            list_state: ListState::default(),
            should_quit: false,
        };

        // Initialize list state to select the first item if available
        if !app.todo_service.todos.is_empty() {
            app.list_state.select(Some(0));
        }

        app
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main loop
        loop {
            terminal.draw(|f| self.draw(f))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)?;
                if self.should_quit {
                    break;
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        let area = f.area();

        // Create a vertical layout
        let chunks = Layout::vertical([
            Constraint::Percentage(100),
        ]).split(area);

        // Create list items from TODOs
        let items: Vec<ListItem> = self.todo_service.todos.iter().enumerate().map(|(i, todo)| {
            let (symbol, style) = if todo.completed {
                ("✓", Style::default().fg(Color::Green))
            } else {
                ("○", Style::default().fg(Color::Yellow))
            };

            // Apply different style to selected item
            let style = if self.list_state.selected() == Some(i) {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            };

            let content = format!(" [{}] {} - {}", symbol, todo.id, todo.text);
            ListItem::new(content).style(style)
        }).collect();

        // Create the list widget
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Todo List"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        // Render the list
        f.render_stateful_widget(list, chunks[0], &mut self.list_state);
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Char(' ') => self.toggle_current_todo(),
            _ => {}
        }
        Ok(())
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.todo_service.todos.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todo_service.todos.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_current_todo(&mut self) {
        if let Some(current_index) = self.list_state.selected() {
            if current_index < self.todo_service.todos.len() {
                let todo = &mut self.todo_service.todos[current_index];
                todo.completed = !todo.completed;
                self.todo_service.save_to_file();
            }
        }
    }
}