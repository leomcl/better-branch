# Better Branch

Better version of git branch IMO.

![Better Branch Demo](demo.gif)

Better Branch provides an interactive fuzzy search interface for managing Git branches — now with multi-select deletion.

## Features

- **Fuzzy Search**: Filter branches instantly as you type.
- **Multi-Select**: Select multiple branches across multiple searches, then delete them all at once.
- **Safe & Force Delete**: Safe delete by default (`git branch -d`), with optional force delete (`-D`) for unmerged branches.
- **Protected Branches**: Automatically prevents deletion of `main`, `master`, `develop`, and `trunk`.
- **Vim Mode**: Optional Vim-style navigation.
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

### 2. Set up Git Alias (recommended)

Set up a short alias to launch the tool quickly.

**Shell alias** (e.g. `.zshrc` or `.bashrc`):
```bash
alias bb='better-branch'
```

**Git alias**:
```bash
git config --global alias.b '!better-branch'
```

## Usage

1. Open any Git repository.
2. Run `bb` (if alias is set) or `better-branch`.
3. **Type** to filter branches instantly.
4. **Navigate** with arrow keys or Vim keys.
5. **Select** branches you want to delete.
6. **Delete** with `Ctrl + D` (safe) or `Ctrl + F` / `Ctrl + X` (force).

## Keybindings

### Navigation

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection up / down |
| `Ctrl + P` / `Ctrl + N` | Move selection up / down |
| `←` / `→` | Move cursor in search bar |

### Selection

| Key | Action |
|---|---|
| `Space` | Toggle selection of highlighted branch |

### Deletion

| Key | Action |
|---|---|
| `Ctrl + D` | Safe delete all selected branches (`git branch -d`) |
| `Ctrl + F` | Force delete all selected branches (`git branch -D`) |
| `Ctrl + X` | Same as `Ctrl + F` |

### Search & Mode

| Key | Action |
|---|---|
| `Esc` | Clear search query. Press again to clear all selections. Press again to quit. |
| `Ctrl + C` | Quit immediately |

### Vim Mode

| Key | Action |
|---|---|
| `Esc` | Enter Vim mode |
| `j` / `k` | Move selection down / up |
| `h` / `l` | Move cursor in search bar |
| `i` / `a` | Return to typing (Insert mode) |
| `q` | Quit |

## Workflow Example

1. **Search** for branches: type `feature` — the list filters down.
2. **Select** branches: navigate with arrows and hit `Space` on two branches. The counter shows `[ 2 branches selected ]`.
3. **Search again** for more: clear `feature` and type `stale` — the previous selections stay in memory but are hidden.
4. **Select more**: hit `Space` on another branch. Counter updates to `[ 3 branches selected ]`.
5. **Delete**: press `Ctrl + D`.
   - Confirmation prompt: `Delete 3 selected branches? (y/N)`
   - Press `y` to confirm.
   - If any branch has unmerged changes, a follow-up prompt appears: `1 branch has unmerged changes. Force delete? (y/N)`

## License

MIT / Apache-2.0

(do what u want just dont blame me for anything)
