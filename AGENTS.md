# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml` defines the `tools-rs` binary crate and dependencies.
- Application entry point lives in `src/main.rs`, wiring CLI commands to modules.
- Reminder logic (CRUD, storage helpers) resides in `src/reminder.rs`; scheduler and desktop notifications sit in `src/scheduler.rs`.
- Built binaries live under `target/`; runtime reminder data persists in `~/.tools-rs/reminders.json`.

## Build, Test, and Development Commands
- `cargo fmt` formats the codebase with Rustfmt defaults.
- `cargo clippy --all-targets --all-features` surfaces lint warnings before review.
- `cargo test` runs the unit/integration suite (add tests as you extend functionality).
- `cargo run -- reminder add "Pay bills" --due "2024-12-31 09:00"` exercises the CLI; `cargo run -- daemon` starts the scheduler.

## Coding Style & Naming Conventions
- Follow Rust 2021 edition defaults: 4-space indentation, `snake_case` for modules/functions, `CamelCase` for types.
- Keep command descriptions concise and leverage `clap` attributes for CLI UX.
- Persist new reminder fields through `Reminder`, `RemindersData`, and serialization helpers in `src/reminder.rs`.

## Testing Guidelines
- Place fast unit tests alongside modules under `#[cfg(test)]`; integration tests belong in `tests/` and run via `cargo test`.
- Include table-driven cases for parsing (dates, cron expressions) and state transitions (complete/remove).
- For async behaviors, prefer `#[tokio::test]` to mirror the production runtime.

## Commit & Pull Request Guidelines
- Use Conventional Commit prefixes (`feat:`, `fix:`, `refactor:`) consistent with existing history.
- Squash trivial fixup commits locally to keep history clean.
- Pull requests should describe user-facing behavior changes, list test commands executed, and link tracker issues when available.
- Provide CLI examples (`cargo run -- reminder list --pending`) or screenshots when behavior affects UX/notifications.

## Scheduler & Notification Notes
- The daemon relies on desktop notification support (`notify-rust`); document platform-specific setup in your PR if additional packages are required.
- Long-running jobs execute under Tokio—avoid blocking calls inside reminder handlers and schedule updates via async utilities.
