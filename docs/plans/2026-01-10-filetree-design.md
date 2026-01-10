# FileTree TUI - Design Document

## Overview

Terminal file explorer with Vim-style navigation, inspired by VSCode's file tree. Built in Rust for maximum performance.

## Stack

- **Rust** with `ratatui` for TUI rendering
- **crossterm** as terminal backend (cross-platform)
- **ignore** crate for efficient `.gitignore` handling
- **tokio** for async file operations

## Project Structure

```
filetree/
├── src/
│   ├── main.rs           # Entry point, event loop
│   ├── app.rs            # Application state
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── tree.rs       # Tree rendering
│   │   └── input.rs      # Search/rename modal
│   ├── fs/
│   │   ├── mod.rs
│   │   ├── entry.rs      # File/directory model
│   │   └── operations.rs # Create, move, delete, copy
│   └── icons.rs          # Extension → Nerd Font icon mapping
├── Cargo.toml
└── README.md
```

## Controls

### Navigation (Normal Mode)

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `h` / `←` | Collapse directory / go to parent |
| `l` / `→` / `Enter` | Expand directory / open file in `$EDITOR` |
| `g` | Go to top |
| `G` | Go to bottom |
| `H` | Toggle hidden files (dotfiles) |

### File Operations

| Key | Action |
|-----|--------|
| `a` | Create file (opens input) |
| `A` | Create directory |
| `r` | Rename |
| `d` | Delete (with `y/n` confirmation) |
| `y` | Copy (yank) to internal clipboard |
| `x` | Cut to internal clipboard |
| `p` | Paste from clipboard |

### Search

| Key | Action |
|-----|--------|
| `/` | Activate search mode |
| `n` | Next result |
| `N` | Previous result |
| `Esc` | Exit search / cancel operation |

### General

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `?` | Show help |

## Visual Design

### Layout

```
┌─ filetree ─────────────────────────────────┐
│ 󰉋 src/                                     │
│   󰉋 ui/                                    │
│      mod.rs                               │
│      tree.rs                              │
│ >  input.rs   ← cursor (highlight)       │
│   󰉋 fs/                                    │
│    󰈙 Cargo.toml                            │
│    README.md                              │
├────────────────────────────────────────────┤
│ [a]dd [r]ename [d]elete [/]search [?]help  │
└────────────────────────────────────────────┘
```

### Colors (ANSI - adapts to terminal theme)

- **Directories**: Blue
- **Executables**: Green
- **Symlinks**: Cyan
- **Cursor/selection**: Inverted background
- **Hidden files**: Dimmed/gray

### Nerd Font Icons

- Directories: `󰉋` (expanded) / `󰉖` (collapsed)
- By extension: `.rs` → ``, `.py` → ``, `.js` → ``, `.md` → ``
- Fallback: `` for unknown files

## File Operations

### Create (`a`/`A`)

- Input appears in bottom bar
- Creates in current directory (where cursor is)
- If cursor on file, creates in parent directory

### Delete (`d`)

- Shows confirmation: `Delete "filename"? [y/N]`
- Directories deleted recursively
- No trash (permanent deletion)

### Copy/Cut/Paste (`y`/`x`/`p`)

- Internal clipboard (not system)
- Visual feedback: "Copied: filename" in status bar
- If destination exists, asks: `Overwrite? [y/N]`

## Error Handling

- Permission denied → Show error in status bar (red), no crash
- File in use → Descriptive message
- Invalid path → Validation before creation
- All errors display for 3 seconds then disappear

## Performance

- Lazy loading: only load directory contents on expand
- Icon cache by extension (HashMap)
- Incremental rendering (only redraw changes)
- Handles trees of ~100k files without issues
