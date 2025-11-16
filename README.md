# Todo CLI Application

A simple command-line interface for managing your todos.

## Installation

First, make sure you have Rust and Cargo installed on your system. Then, build the application:

```bash
cargo build --release
```

## Usage

The application supports the following commands:

### Add a new todo
```bash
# Using the full command name
cargo run -- add "Buy groceries"

# Using the alias
cargo run -- a "Buy groceries"
```

### List all todos
```bash
# Using the full command name
cargo run -- list

# Using the alias
cargo run -- l
```

### Mark a todo as completed
```bash
# Using the full command name
cargo run -- complete 1

# Using the alias
cargo run -- c 1
```

### Remove a todo
```bash
# Using the full command name
cargo run -- remove 1

# Using the alias
cargo run -- r 1
```

### Clear all completed todos
```bash
# Using the full command name
cargo run -- clear

# Using the alias
cargo run -- x
```

## Features

- Add new todo items with descriptions
- List all todos with their completion status
- Mark todos as completed
- Remove specific todos
- Clear all completed todos
- Data persistence using JSON file storage
- Cross-platform file storage (uses system config directory)

## Data Storage

The application stores your todos in a JSON file located at:
- Linux/macOS: `~/.config/todo-rs/todos.json`
- Windows: `%APPDATA%\todo-rs\todos.json`

## Command Aliases

For convenience, each command has an alias:
- `add` → `a`
- `list` → `l`
- `complete` → `c`
- `remove` → `r`
- `clear` → `x`