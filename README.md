# filetree

<p align="center">
  <img src="assets/header.png" alt="filetree header" width="100%">
</p>

A fast, minimal terminal file explorer with Vim-style navigation and Nerd Font icons.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

## Features

- **Vim-style navigation** - `hjkl` keys, `g`/`G` for top/bottom
- **File operations** - create, rename, delete, copy, cut, paste
- **Search** - incremental search with `/`, navigate with `n`/`N`
- **Nerd Font icons** - beautiful file type icons (requires [Nerd Font](https://www.nerdfonts.com/))
- **Responsive UI** - adapts to terminal size
- **Fast** - built in Rust, handles large directories efficiently
- **Minimal** - single binary, no config files needed

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/ixjosemi/filetree.git
cd filetree

# Build and install
cargo build --release
cp target/release/filetree ~/.local/bin/ft
```

### Requirements

- Rust 1.70+
- A [Nerd Font](https://www.nerdfonts.com/) installed and configured in your terminal

## Usage

```bash
# Open current directory
ft

# Open specific directory
ft ~/projects
ft /etc
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse directory / go to parent |
| `l` / `→` / `Enter` | Expand directory / open file in `$EDITOR` |
| `g` | Go to top |
| `G` | Go to bottom |

### File Operations

| Key | Action |
|-----|--------|
| `a` | Create file |
| `A` | Create directory |
| `r` | Rename |
| `d` | Delete (with confirmation) |
| `y` | Copy (yank) |
| `x` | Cut |
| `p` | Paste |

### Other

| Key | Action |
|-----|--------|
| `/` | Search |
| `n` / `N` | Next / previous search result |
| `H` | Toggle hidden files |
| `E` | Expand all directories |
| `W` | Collapse all directories |
| `R` | Refresh tree |
| `?` | Show help |
| `q` | Quit |

### Mouse

| Action | Effect |
|--------|--------|
| Left click | Select item |
| Right click | Open file / toggle directory |
| Scroll | Navigate up/down |

## Configuration

filetree uses your system's `$EDITOR` environment variable to open files. If not set, it defaults to `vim`.

```bash
# Set your preferred editor
export EDITOR=nvim
```

## Performance

- **Lazy loading** - directories are only loaded when expanded
- **Optimized binary** - LTO enabled, single codegen unit
- **Minimal dependencies** - fast startup time

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

## License

MIT License - see [LICENSE](LICENSE) for details.
