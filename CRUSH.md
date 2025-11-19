# RustyTodos - TUI Branch

## Project Overview

RustyTodos is a terminal-based (TUI) todo application written in Rust. This codebase is on the **TUI branch** - there is a separate GUI branch that uses Tauri. The TUI version is the original implementation and is installable via `cargo install`.

**Key Features:**
- Interactive terminal UI using ratatui
- Smart natural language date parsing
- Color-coded task status (green=completed, red=overdue, yellow=pending)
- Background daemon for due date notifications
- Cross-platform notification support (Linux, Windows, macOS)
- Search functionality
- Persistent JSON storage

## Project Structure

```
src/
├── main.rs      # Entry point, initializes TUI and daemon
├── app.rs       # App state, data loading/saving, todo operations
├── todo.rs      # Todo data structure
├── tui.rs       # UI rendering and event handling, date parser
└── daemon.rs    # Background notification daemon
```

## Essential Commands

### Build & Run
```bash
# Run in development
cargo run

# Build release binary
cargo build --release
# Binary located at: target/release/RustyTodos

# Install locally (replaces any installed version)
cargo install --path .
```

### Standard Rust Commands
```bash
cargo build       # Development build
cargo test        # Run tests (if any exist)
cargo check       # Fast type checking
cargo clippy      # Linting
cargo fmt         # Code formatting
```

## Code Organization

### Module Structure
- **main.rs**: Spawns daemon thread, initializes terminal (crossterm), runs TUI event loop, saves state on exit
- **app.rs**: Core application logic
  - `App` struct holds todos, input state, search query
  - `InputMode` enum: Normal, EditingDescription, EditingDueDate, Searching
  - CRUD operations: `add_todo()`, `delete_todo()`, `mark_done()`
  - Persistence: `save_to_file()`, `load_from_file()`
- **todo.rs**: Simple `Todo` struct with description, due_date, created_date, done flag
- **tui.rs**: 
  - `run_app()`: Main event loop, polls for keyboard events
  - `ui()`: Renders TUI layout with ratatui
  - `parse_due_date()`: Complex natural language date parser
- **daemon.rs**: Background thread checking for due todos every 60 seconds, sends platform-specific notifications

### Data Storage
- **Linux/macOS**: `~/.local/share/rustytodos/todos.json` (via `directories` crate)
- **Windows**: `%APPDATA%/rustytodos/todos.json`
- Actually uses `ProjectDirs::from("com", "KushalMeghani", "RustyTodos").config_dir()` (see app.rs:12-17)
- Data persisted as JSON using serde

## Key Patterns & Conventions

### State Management
- App state uses Rust ownership model - state passed mutably to `run_app()`
- All state changes happen in `App` methods
- UI is stateless, re-renders each frame based on current app state
- `#[serde(skip)]` on transient fields (input modes, UI state)

### Input Modes
The app uses a modal interface (like vim):
- **Normal**: Navigate with arrow keys, press keys to trigger actions
- **EditingDescription**: Type description, Enter to move to due date
- **EditingDueDate**: Type due date, Enter to create todo
- **Searching**: Type to filter, arrow keys to navigate filtered list, Esc to exit

### Keyboard Shortcuts
- `a` - Add new todo (enters EditingDescription mode)
- `m` - Mark selected todo as done/undone (toggle)
- `d` - Delete selected todo
- `/` - Enter search mode
- `q` - Quit application
- `↑/↓` - Navigate todo list
- `Enter` - Confirm input (context-dependent)
- `Esc` - Cancel current operation, return to Normal mode
- `Backspace` - Delete character in input fields

### Color Coding
Task status is color-coded in the UI (see tui.rs:188-194):
- **Green**: Completed tasks (regardless of due date)
- **Red**: Overdue tasks (not completed, past due date)
- **Yellow**: Pending tasks (not completed, not overdue)

### Date Parsing
The `parse_due_date()` function in tui.rs supports extensive natural language:
- Relative: "now", "today", "tomorrow", "yesterday"
- Weekdays: "friday", "next monday", "this wednesday"
- Offsets: "in 30 minutes", "2 hours", "in 3 days"
- Specific: "2024-12-25", "15:30", "friday 15:30"
- Combinations: "in 1 day 3 hours", "next friday 15:30"

Date formats returned:
- Date only: `%Y-%m-%d` (e.g., "2024-12-25")
- Date + time: `%Y-%m-%d %H:%M` (e.g., "2024-12-25 15:30")

### Platform-Specific Code
Uses conditional compilation for notifications:
- **Linux**: `notify-rust` crate
- **Windows**: `notifica` crate
- **macOS**: `mac-notification-sys` crate (dependency declared but may not be used in daemon.rs)

Example pattern:
```rust
#[cfg(target_os = "linux")]
use notify_rust::Notification;

#[cfg(target_os = "windows")]
use notifica::notify;
```

## Important Gotchas

### Terminal State Management
- **CRITICAL**: Terminal is put in raw mode (crossterm) and alternate screen on startup
- On exit, MUST restore terminal state or it will be broken
- main.rs handles this with proper error handling and cleanup
- If app panics, terminal may need manual reset (type `reset` command)

### Daemon Thread
- Spawned in main.rs before TUI starts
- Runs indefinitely in background, checking todos every 60 seconds
- Error handling: daemon errors printed to stderr but don't crash main app
- No graceful shutdown mechanism - thread just dies when process exits

### Date Validation
- `validate_not_past()` prevents creating todos with past due dates
- Returns error message shown in UI
- Only applies during creation, not for existing todos

### Selection State
- `app.selected` is an index into filtered todos (when searching)
- Can go out of bounds if todos deleted or search changes
- UI handles with `app.selected.min(todos.len() - 1)` pattern
- Selected resets to 0 when entering/changing search

### Search Behavior
- Search filters by description AND due date fields (case-insensitive)
- Empty search shows all todos
- Search query persists until cleared or Esc pressed
- While searching, can still navigate, mark done, delete (operates on filtered list)

### File I/O
- No file locking - concurrent instances could corrupt data
- Save happens only on clean exit (Ctrl+C may not save)
- If JSON is corrupted, falls back to empty app (App::new())

## Dependencies

Core UI:
- **ratatui** 0.20: Terminal UI framework (formerly tui-rs)
- **crossterm** 0.26: Cross-platform terminal manipulation

Data:
- **serde** + **serde_json**: JSON serialization
- **chrono** 0.4: Date/time handling with serde support
- **directories** 5.0: Cross-platform config/data directories

Notifications (platform-specific):
- **notify-rust** 4: Linux desktop notifications
- **notifica** 3.0.2: Windows toast notifications
- **mac-notification-sys** 0.5: macOS notification center

## Branches

- **TUI**: Original terminal app (current branch)
- **gui**: Tauri-based GUI application (separate codebase)

These are essentially different applications sharing the same name. The TUI branch is the one that matches the published crate on crates.io.

## Testing Notes

No test suite currently exists in the codebase. If adding tests:
- Unit test date parsing logic in tui.rs (many edge cases)
- Test App state transitions
- Mock file I/O for persistence tests
- Consider testing daemon notification logic

## Known Issues

1. **README has merge conflict markers** (lines 38-59) - needs cleanup
2. **Unused import** in app.rs:7 (`std::f32::consts::PI`)
3. **macOS notification dependency** declared but implementation in daemon.rs may be incomplete
4. **No graceful daemon shutdown** - thread runs until process kill
5. **No file locking** - race conditions possible with multiple instances
6. **Terminal state corruption** if app panics before cleanup

## Development Workflow

1. Make changes to source files
2. Test with `cargo run`
3. Check compilation with `cargo check`
4. Format code with `cargo fmt`
5. Install locally for testing: `cargo install --path .`
6. Run installed version: `rustytodos` (or `RustyTodos` depending on platform)

### Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>: <description>

[optional body]

💘 Generated with Crush

Co-Authored-By: Crush <crush@charm.land>
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `style:` - Code style/formatting (no logic change)
- `refactor:` - Code restructuring (no feature/fix)
- `perf:` - Performance improvement
- `test:` - Add/modify tests
- `chore:` - Maintenance tasks (dependencies, build, etc.)

## License

GPL-3.0-or-later (changed from MIT in commit d139107)

## Contributors

- Original author: Kushal Meghani
- Contributor mentioned in README: [Kivooeo](https://github.com/Kivooeo)
