# Todo CLI Application

A command-line interface application built with Rust for managing your todos efficiently.

## Overview

This is a simple yet powerful CLI tool that helps you manage your tasks from the terminal. Written in Rust, it offers fast performance and reliable cross-platform functionality.

## Features

- Add new todo items with descriptions
- List all todos with their completion status
- Mark todos as completed
- Remove specific todos
- Clear all completed todos at once
- Persistent storage using JSON files
- Cross-platform support (Linux, macOS, Windows)

## Installation

1. Ensure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system
2. Clone or download this repository
3. Build the application:

```bash
cargo build --release
```

The executable will be available in `target/release/`.

## Usage

Run the application using Cargo:

```bash
cargo run -- <command> [arguments]
```

Or run the built executable directly:

```bash
./target/release/tools <command> [arguments]
```

### Available Commands

#### Add a new todo
```bash
# Using the full command name
cargo run -- add "Buy groceries"

# Using the alias
cargo run -- a "Buy groceries"
```

#### List all todos
```bash
# Using the full command name
cargo run -- list

# Using the alias
cargo run -- l
```

#### Mark a todo as completed
```bash
# Using the full command name
cargo run -- complete 1

# Using the alias
cargo run -- c 1
```

#### Remove a todo
```bash
# Using the full command name
cargo run -- remove 1

# Using the alias
cargo run -- r 1
```

#### Clear all completed todos
```bash
# Using the full command name
cargo run -- clear

# Using the alias
cargo run -- x
```

### Command Aliases

For convenience, each command has a shorter alias:

| Full Command | Alias | Description |
|--------------|-------|-------------|
| `add` | `a` | Add a new todo |
| `list` | `l` | List all todos |
| `complete` | `c` | Mark a todo as completed |
| `remove` | `r` | Remove a specific todo |
| `clear` | `x` | Clear all completed todos |

## Data Storage

The application stores your todos in a JSON file for persistence between sessions:

- **Linux/macOS**: `~/.config/todo-rs/todos.json`
- **Windows**: `%APPDATA%\todo-rs\todos.json`

This location is managed automatically by the application and doesn't require manual file management.

## Project Structure

The application is organized into several modules:

- `src/main.rs` - Application entry point
- `src/types.rs` - Type definitions
- `src/ui.rs` - User interface components
- `src/commands/` - Command implementations
- `src/services/` - Business logic and data handling

## Dependencies

This application requires Rust 1.0 or later and uses the following dependencies defined in `Cargo.toml`:

- Standard Rust libraries for CLI functionality
- File I/O operations
- JSON serialization/deserialization

## Contributing

Contributions are welcome! Feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.