# Gik

**Gik** is a high-performance, modern transactional version control system written in Rust. It combines the reliability of an ACID-compliant database with the flexibility of contemporary versioning workflows like Jujutsu-style bookmarks.

## Key Features

- **ACID Transactions**: Built on `redb`. Every operation (staging, committing, branching) is atomic. Your repository state can never be corrupted.
- **Floating Bookmarks**: Branching reimagined. Bookmarks are lightweight labels that automatically slide forward when you commit. No more "detached HEAD" nightmares.
- **Smart Staging**: Full support for recursive staging (`gik stage .`), directory-level staging, and explicit staging of file deletions.
- **Seamless Merging & Conflict Resolution**: Advanced interactive `gik merge` that automatically handles three-way merges. Includes a built-in interactive prompt to resolve conflicts or insert standard Git conflict markers (`<<<<<<< HEAD`) to resolve them in your favorite editor.
- **Native Remote Sync**: Full support for `gik pull` and `gik push` communicating directly with GitHub and other Git servers via the Smart HTTP protocol.
- **Instant Time Travel**: Switch between any commit or bookmark instantly with `gik checkout`, featuring built-in safety checks for uncommitted changes.
- **Git Compatibility**: Uses canonical Git binary formats for Blobs, Trees, and Commits, ensuring a familiar data model.
- **Deep Visibility**: Line-by-line `gik diff` and comprehensive `gik log --all` for a clear view of your project's evolution.
- **Subdirectory Support**: Run Gik commands from any folder within your project. It automatically discovers the repository root.
- **Zero-Config Onboarding**: Import your existing identity from Git with a single command: `gik config --import-git`.

## Installation

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.ps1 | iex
```

## Usage & Commands

### 1. Initialize
```bash
gik init
```

### 2. Configure Identity
```bash
# Manual setup
gik config --global user.name "Your Name"
gik config --global user.email "you@example.com"

# OR: Instant import from Git
gik config --import-git
```

### 3. Stage Changes
```bash
gik stage file.txt    # Single file
gik stage src/        # Directory
gik stage .           # Everything (adds new, updates modified, stages deleted)
```

### 4. Commit
By default, `gik commit` will automatically stage all your unstaged changes and commit them. If you want to commit **only** the files you explicitly staged, use the `--staged` flag.

```bash
# Auto-stages all local changes and commits them
gik commit -m "feat: implement magic"

# Commits ONLY the files that were explicitly staged
gik commit -m "fix: bugfix" --staged
```

### 5. Branching (Bookmarks) & Merging
```bash
gik branch feature-x           # Create a bookmark on current HEAD
gik branch                     # List all bookmarks
gik checkout feature-x         # Switch to a bookmark or hash
gik merge main                 # Merge main into current bookmark
gik merge --continue           # Continue merge after resolving conflicts manually
```

### 6. Remote Sync
```bash
gik config remote.origin.url https://github.com/user/repo.git
gik push                       # Push your commits and bookmarks to remote
gik pull                       # Fetch and merge changes from remote
```

### 7. Inspect & Restore
```bash
gik status                     # Current state overview
gik diff                       # See exact changes
gik log --all                  # View entire commit graph
gik restore .                  # Discard all local changes
```

### 8. The Magic Button (Undo)
```bash
gik undo                       # Roll back the last action (physically restores files!)
```

## License

Gik is dual-licensed under the **MIT** and **Apache-2.0** licenses. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## Contributing

We are in the MVP stage and welcome all feedback! Feel free to open issues or submit pull requests.
