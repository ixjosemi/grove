# Contributing to filetree

Thank you for your interest in contributing to filetree! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- A terminal with Nerd Font support (for testing icons)
- Git

### Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/filetree.git
cd filetree

# Build the project
cargo build

# Run in development mode
cargo run
```

## How to Contribute

### Reporting Bugs

Before submitting a bug report:

1. Check if the issue already exists in [GitHub Issues](https://github.com/ixjosemi/filetree/issues)
2. Try to reproduce with the latest version

When submitting a bug report, include:

- Your OS and terminal emulator
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior
- Any error messages

### Suggesting Features

Feature requests are welcome! Please:

1. Check existing issues for similar suggestions
2. Describe the use case clearly
3. Explain why this would benefit most users

### Pull Requests

1. **Fork** the repository
2. **Create a branch** for your feature/fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** following the code style below
4. **Test** your changes manually
5. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add new feature"
   git commit -m "fix: resolve issue with X"
   ```
6. **Push** and create a Pull Request

## Code Style

### Rust Conventions

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Prefer `anyhow::Result` for error handling
- Use meaningful variable names
- Keep functions focused and small

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Formatting, no code change
- `refactor:` - Code restructuring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

### Project Structure

```
src/
├── main.rs      # Entry point, event handlers
├── app.rs       # Application state
├── icons.rs     # File type icons
├── fs/          # Filesystem operations
└── ui/          # Terminal UI rendering
```

## Development Guidelines

### Adding Features

1. Keep the UI minimal and keyboard-driven
2. Follow Vim conventions for keybindings
3. Ensure responsiveness at different terminal sizes
4. Update help text and documentation

### Performance

- Lazy load directory contents
- Avoid blocking the main thread
- Profile with large directories (~10k files)

### Compatibility

- Test on macOS and Linux
- Use `#[cfg(unix)]` for platform-specific code
- Avoid external runtime dependencies

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help newcomers learn
- Focus on the code, not the person

## Questions?

Feel free to open an issue for questions or join discussions on existing issues.

Thank you for contributing!
