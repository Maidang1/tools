# Qwen Code Project Context

## Project Overview

This is a Rust-based command-line interface (CLI) application for managing todos called `tools-rs`. The application is built with the following key technologies:

- **Language**: Rust
- **CLI Framework**: `clap` for parsing command-line arguments
- **UI Framework**: `ratatui` and `crossterm` for terminal user interface
- **Serialization**: `serde` and `serde_json` for JSON data handling
- **File System**: `dirs` for cross-platform configuration directories

## Project Structure

```
/Users/felixwliu/codes/open-source/tools/
├── Cargo.lock          # Cargo lock file
├── Cargo.toml         # Rust manifest file with dependencies
├── QWEN.md            # This file (project context)
├── README.md          # Project documentation
├── src/               # Source code directory
│   ├── main.rs        # Application entry point
│   ├── types.rs       # Type definitions (TodoItem struct)
│   ├── ui.rs          # TUI (Terminal User Interface) implementation
│   ├── commands/      # CLI command definitions
│   │   ├── mod.rs
│   │   └── todo.rs
│   └── services/      # Business logic and data handling
│       ├── mod.rs
│       └── todo_service.rs
└── target/            # Build artifacts (git-ignored)
```

## Application Architecture

The application is structured with a clear separation of concerns:

1. **Commands Layer** (`src/commands/`): CLI command definitions using `clap`
2. **Services Layer** (`src/services/`): Business logic and data persistence
3. **Types Layer** (`src/types.rs`): Data structures and serialization
4. **UI Layer** (`src/ui.rs`): Terminal user interface using Ratatui
5. **Entry Point** (`src/main.rs`): Main application logic that coordinates components

## Key Features and Functionality

1. **Todo Management Commands**:
   - Add new todos
   - Delete todos by ID
   - List all todos
   - Interactive UI mode

2. **Data Persistence**:
   - Todos are stored in `~/.milk-cap/todo.json`
   - Uses JSON format for serialization

3. **Interactive UI**:
   - Terminal-based user interface
   - Navigation with arrow keys or hjkl
   - Toggle todo completion with spacebar
   - Quit with 'q' or Escape

## Building and Running

1. **Prerequisites**: Rust and Cargo must be installed on the system

2. **Build Commands**:
   ```bash
   # Build in debug mode
   cargo build

   # Build in release mode
   cargo build --release
   ```

3. **Run Commands**:
   ```bash
   # Run directly with cargo
   cargo run -- todo add "My new task"
   cargo run -- todo list
   cargo run -- todo delete 1
   cargo run -- todo interactive

   # Or run the built executable
   ./target/debug/tools-rs todo add "My new task"
   ```

## Development Conventions

1. **Code Style**:
   - Follows Rust idioms and conventions
   - Uses Rust's module system for organization
   - Implements clap derive macros for CLI commands
   - Uses serde derive macros for serialization

2. **Testing**:
   - Unit tests exist in the types.rs file
   - Tests for TodoItem creation and display formatting

3. **Error Handling**:
   - Uses Rust's Result type for error handling
   - Panics on I/O errors when writing to file (could be improved)

## Important Files

- `Cargo.toml`: Defines dependencies and build settings
- `src/main.rs`: Entry point and main CLI logic
- `src/services/todo_service.rs`: Core business logic for todo operations
- `src/ui.rs`: Interactive terminal UI implementation
- `src/types.rs`: Data model definition with serialization support

## Dependencies

Key dependencies in `Cargo.toml`:
- `clap`: Command-line argument parsing
- `serde`/`serde_json`: Serialization and JSON handling
- `dirs`: Cross-platform directory paths
- `ratatui`/`crossterm`: Terminal UI implementation

## Potential Areas for Improvement

1. Error handling in service layer (currently panics on file write errors)
2. More comprehensive test coverage
3. Additional CLI commands based on README.md vs. actual implementation
4. Better documentation for the actual implementation vs. what's documented

## Note on Implementation vs. Documentation

The actual implementation differs from the documented feature set in README.md. The current implementation has:
- Commands: `todo add`, `todo delete`, `todo list`, `todo interactive`
- Rather than the documented: `add`, `list`, `complete`, `remove`, `clear`

This suggests the project may have evolved from its original design or is in a transitional state.