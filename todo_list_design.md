# Rust CLI Todo List Application - Design Specification

## Overview
A command-line interface (CLI) application written in Rust for managing a todo list. The application will provide a simple yet powerful way to create, manage, and track tasks from the terminal.

## Data Structures

### TodoItem
The core data structure representing a single todo item:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: u32,
    pub content: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub tags: Vec<String>,
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.completed { "✓" } else { "○" };
        let priority = match self.priority {
            Priority::High => " (!!!)",
            Priority::Medium => " (!!)",
            Priority::Low => " (!)",
            Priority::None => "",
        };
        write!(
            f,
            "[{}] {}{} - {}",
            self.id, status, priority, self.content
        )
    }
}
```

### Priority Enum
Represents the priority level of a todo item:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    None,
    Low,
    Medium,
    High,
}
```

### TodoList
A collection of todo items:

```rust
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub next_id: u32,
}
```

## Features

### Core Features

1. **Add Item**
   - Syntax: `todo add "Task description"`
   - Optional flags:
     - `--priority [high|medium|low|none]` - Set priority level
     - `--due <date>` - Set due date (YYYY-MM-DD format)
     - `--tag <tag>` - Add tags to the task
   - Creates a new todo item with a unique ID

2. **List Items**
   - Syntax: `todo list`
   - Optional flags:
     - `--all` - Show all items (default: shows only incomplete)
     - `--completed` - Show only completed items
     - `--incomplete` - Show only incomplete items
     - `--priority [high|medium|low|none]` - Filter by priority
     - `--sort [id|due|priority|created]` - Sort by specified field
   - Displays all todo items with their status, priority, and due dates

3. **Complete Item**
   - Syntax: `todo complete <id>`
   - Marks the specified todo item as completed
   - Updates the `updated_at` timestamp

4. **Uncomplete Item**
   - Syntax: `todo uncomplete <id>`
   - Marks the specified todo item as incomplete
   - Updates the `updated_at` timestamp

5. **Delete Item**
   - Syntax: `todo delete <id>`
   - Removes the specified todo item from the list

6. **Edit Item**
   - Syntax: `todo edit <id> "New description"`
   - Optional flags:
     - `--priority [high|medium|low|none]` - Change priority
     - `--due <date>` - Change due date
   - Modifies the content or other properties of an existing todo item

7. **Clear All**
   - Syntax: `todo clear`
   - Optional flags:
     - `--completed` - Clear only completed items (default behavior)
     - `--all` - Clear all items
   - Removes specified todo items from the list

### Advanced Features

8. **Search Items**
   - Syntax: `todo search <query>`
   - Finds todo items containing the specified query in their content or tags

9. **Tag Management**
   - Syntax: `todo tag <id> <tag>`
   - Allows adding tags to todo items for better organization

10. **Filter by Due Date**
    - Syntax: `todo list --due [today|week|month|overdue]`
    - Filters items based on due date

11. **Statistics**
    - Syntax: `todo stats`
    - Shows statistics like total items, completed items, completion rate, etc.

## CLI Design

### Command Structure
```
todo <SUBCOMMAND>

SUBCOMMANDS:
    add        Add a new todo item
    list       List all todo items
    complete   Mark a todo item as completed
    uncomplete Mark a todo item as incomplete
    delete     Delete a todo item
    edit       Edit a todo item
    clear      Clear completed or all todo items
    search     Search for todo items
    tag        Add tags to a todo item
    stats      Show statistics about todo items
    help       Print this message or the help of the given subcommand(s)
```

### CLI Arguments Structure
Using `clap` for argument parsing:

```rust
use clap::Parser;

#[derive(Parser)]
#[clap(name = "Todo", about = "A CLI todo list application")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new todo item
    Add {
        content: String,
        #[clap(long, value_enum)]
        priority: Option<Priority>,
        #[clap(long)]
        due: Option<String>,  // Parse to DateTime later
        #[clap(long)]
        tag: Option<Vec<String>>,
    },
    /// List all todo items
    List {
        #[clap(long, conflicts_with_all = &["completed", "incomplete"])]
        all: bool,
        #[clap(long, conflicts_with_all = &["all", "incomplete"])]
        completed: bool,
        #[clap(long, conflicts_with_all = &["all", "completed"])]
        incomplete: bool,
        #[clap(long, value_enum)]
        priority: Option<Priority>,
        #[clap(long, value_enum)]
        sort: Option<SortBy>,
        #[clap(long)]
        due: Option<DueFilter>,
    },
    // ... other commands
}
```

## Persistence

### Storage Format
- Store todo data in JSON format in a file (e.g., `~/.todo.json`)
- Use `serde` for serialization/deserialization
- Implement proper error handling for file operations

### File Locations
- Default: `~/.todo.json`
- Configurable via environment variable or CLI flag

## Dependencies

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }  # For CLI parsing
serde = { version = "1.0", features = ["derive"] }  # For serialization
serde_json = "1.0"  # For JSON operations
chrono = { version = "0.4", features = ["serde"] }  # For date/time handling
directories = "5.0"  # For getting config directory
```

## Implementation Considerations

1. **Error Handling**: Use Result types and proper error handling throughout the application
2. **Performance**: Efficient data structures for common operations (listing, filtering)
3. **Usability**: Clear, intuitive CLI with helpful error messages
4. **Cross-platform**: Handle file paths and line endings appropriately across platforms
5. **Testing**: Unit tests for core functionality and CLI command handling

## Security Considerations

- Minimal permissions required (just file access for persistence)
- Input validation for all user-provided values (content, tags, etc.)
- Sanitize file paths to prevent directory traversal attacks

## Future Enhancements

- Synchronization with cloud storage
- Web UI companion application
- Import/export functionality (CSV, JSON)
- Recurring tasks
- Project/collection support
- Notification system