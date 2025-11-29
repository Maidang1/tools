use crate::ui::state::{AppState, InputMode};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Trait for UI rendering functionality
pub trait UiRenderer {
    fn draw(f: &mut Frame, state: &mut AppState);
}

pub struct Renderer;

impl UiRenderer for Renderer {
    fn draw(f: &mut Frame, state: &mut AppState) {
        let area = f.area();

        // Create a vertical layout with space for footer
        let chunks = Layout::vertical([
            Constraint::Min(5),    // Main content area (at least 5 rows)
            Constraint::Length(3), // Footer area for key bindings (3 rows)
        ])
        .split(area);

        // Create list items from TODOs for the main area
        let items: Vec<ListItem> = state
            .todo_service
            .todos
            .iter()
            .enumerate()
            .map(|(i, todo)| {
                let (symbol, style) = if todo.completed {
                    ("✓", Style::default().fg(Color::Green))
                } else {
                    ("○", Style::default().fg(Color::Yellow))
                };

                // Apply different style to selected item
                let style = if state.list_state.selected() == Some(i) {
                    style.add_modifier(Modifier::REVERSED)
                } else {
                    style
                };

                let content = format!(" [{}] {} - {}", symbol, todo.id, todo.text);
                ListItem::new(content).style(style)
            })
            .collect();

        // Create the list widget
        let block_title = if state.input_mode == InputMode::Normal {
            "Todo List"
        } else {
            "Todo List - ADDING MODE"
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(block_title))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        // Render the list in the main area
        f.render_stateful_widget(list, chunks[0], &mut state.list_state);

        // Add footer with key bindings
        let footer_text = if state.input_mode == InputMode::Normal {
            "q:quit | a/n:add | d/x:delete | space:toggle | j/k:down/up"
        } else {
            "Enter:confirm | Esc:cancel"
        };

        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().add_modifier(Modifier::DIM));

        f.render_widget(footer, chunks[1]);

        // If in input mode, render the input field on top of other elements
        if state.input_mode == InputMode::Adding {
            // Create a centered popup area
            let popup_width = 60;
            let popup_height = 3;
            let area = ratatui::layout::Rect {
                x: (chunks[0].width.saturating_sub(popup_width)) / 2,
                y: (chunks[0].height.saturating_sub(popup_height)) / 2 + chunks[0].y,
                width: popup_width,
                height: popup_height,
            };

            // Create the input field
            let input_text = format!("Add Todo: {}", state.input_text);
            let input_field = Paragraph::new(input_text)
                .block(Block::default().borders(Borders::ALL).title("Input"));

            // Render a clear area and then the input field
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(input_field, area);
        }
    }
}