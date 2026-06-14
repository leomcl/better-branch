# Better Branch

Better version of git branch IMO.

Better Branch provides a fuzzy search interface for your Git branches and keyboard shortcuts.

## Features

- **Fuzzy Search**
- **Vim Mode**: Optional Vim-style navigation
- **Minimal Dependencies**: Self-contained UI logic.

## Quick Start

### 1. Install

The easiest way to install is via Cargo:

```bash
# Install directly from the source
cargo install --path .
```

Or, you can build a release binary and move it to your path manually:

```bash
cargo build --release
sudo cp target/release/better-branch /usr/local/bin/
```

### 2. Set up Git Alias (should do)

Set up a short alias to launch the tool quickly.

Create alias in zshrc:
```bash
alias gb='better-branch'
```

or via git config:

```bash
git config --global alias.b '!better-branch'
```

## Usage & Shortcuts

1. Open any Git repository.
2. Run `gb` (if alias is set) or `better-branch`.
3. **Type** to filter branches instantly.
4. **Navigation**:
    - `Ctrl + N` or `Down Arrow`: Move selection down.
    - `Ctrl + P` or `Up Arrow`: Move selection up.
    - `Enter`: Checkout the selected branch.
    - `Ctrl + C`: Exit without changes.
5. **Vim Mode**:
    - Press `Esc` to enter Vim mode.
    - Use `j` / `k` to move, `h` / `l` to move cursor.
    - Press `i` or `a` to return to typing (Insert mode).

## License

MIT / Apache-2.0

(do what u want just dont blame me for anything)
