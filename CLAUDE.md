# Claude Code Instructions

## Project Overview

filetree is a terminal file explorer (TUI) written in Rust. It provides Vim-style navigation, file operations, and Nerd Font icons.

## Architecture

```
src/
├── main.rs      # Entry point, event loop, key handlers
├── app.rs       # Application state (App struct, modes, clipboard)
├── icons.rs     # Nerd Font icon mapping by file extension
├── fs/
│   ├── mod.rs
│   ├── entry.rs # FileEntry model (file/directory metadata)
│   └── tree.rs  # Directory traversal and tree building
└── ui/
    ├── mod.rs
    └── tree.rs  # Ratatui rendering (tree view, help bar, overlays)
```

## Key Patterns

### State Management
- `App` struct holds all application state
- `AppMode` enum for modal behavior (Normal, Search, Input, Confirm, Help)
- State mutations happen in `main.rs` handlers, rendering in `ui/`

### Event Loop
```rust
loop {
    terminal.draw(|f| ui::draw(f, app))?;
    if event::poll(...)? {
        handle_key(app, key)?;
    }
    if let Some(path) = app.pending_editor_file.take() {
        open_in_editor(terminal, &path)?;
    }
}
```

### File Operations
- Use `app.pending_editor_file` for deferred editor opening (needs terminal access)
- Always call `app.refresh()` after filesystem changes
- Use `app.set_status()` for user feedback

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build (optimized)
cargo run                # Run debug
cargo run --release      # Run release
```

## Code Style

- Use `anyhow::Result` for error handling
- Prefer `?` operator over explicit match
- Keep handlers in `main.rs`, rendering in `ui/tree.rs`
- Use `#[cfg(unix)]` for platform-specific code

## Common Tasks

### Adding a new keybinding
1. Add handler in `handle_normal_mode()` in `main.rs`
2. Update help text in `render_help_bar()` in `ui/tree.rs`
3. Update help overlay in `render_help_overlay()` in `ui/tree.rs`

### Adding a new file type icon
1. Add entry in `ICONS` HashMap in `icons.rs`

### Adding a new mode
1. Add variant to `AppMode` enum in `app.rs`
2. Add handler function in `main.rs`
3. Update `handle_key()` match in `main.rs`
4. Update `render_help_bar()` for mode-specific help

## Testing

Currently no automated tests. Manual testing:
```bash
ft                    # Test in current directory
ft /tmp               # Test in /tmp
ft ~                  # Test in home directory
```

## Dependencies

- `ratatui` - TUI framework
- `crossterm` - Terminal backend
- `anyhow` - Error handling
- `dirs` - System directories
- `ignore` - Gitignore-aware traversal (unused currently)
- `tokio` - Async runtime (unused currently, for future features)
