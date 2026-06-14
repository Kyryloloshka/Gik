# Gik

**Gik** is a high-performance, transactional version control system written in Rust. It's designed to be Git-compatible on the binary level while providing 100% data safety through an ACID-compliant embedded database.

## Quick Install

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.ps1 | iex
```

## Key Features

- **ACID Transactions**: Every operation is atomic. No more corrupted repository states.
- **Git Binary Compatibility**: Uses canonical Git formats for Blobs, Trees, and Commits.
- **Streaming IO**: Efficiently handles large files without memory overhead.
- **Instant Undo**: Built-in transaction logging allows you to roll back any action instantly.

## Usage & Commands

### 1. Initialize a repository

Creates a new transactional database in the current directory.

```bash
gik init
```

### 2. Stage files

Hash and prepare files for the next commit.

```bash
gik stage <path>
```

### 3. Commit changes

Record a permanent snapshot of the staged changes.

```bash
gik commit -m "Your descriptive message"
```

### 4. View history

Browse the commit graph starting from HEAD.

```bash
gik log
```

### 5. Undo last action

The "Magic Button". Instantly rolls back the last staging or commit operation.

```bash
gik undo
```

## ⚖️ License
Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

## 🤝 Contributing

We welcome contributions! Please feel free to open issues or submit pull requests.


