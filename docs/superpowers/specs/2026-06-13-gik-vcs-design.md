# Gik (Rust VCS) Design Specification

## 1. Overview
Gik is a minimal viable product (MVP) of a Git-like transactional version control system written in Rust. It utilizes an embedded key-value store (`redb`) to ensure ACID compliance for all repository operations. The CLI is designed to be minimalistic (Unix-like), providing feedback only on errors.

## 2. Architecture & Components (Layered)
The codebase will be a single Rust crate structured into logical modules to balance simplicity with future extensibility:
- `cli`: Command-line interface definition using `clap`. Maps user input to business commands.
- `commands`: Business logic implementation for `init`, `stage`, `commit`, `log`, and `undo`.
- `storage`: Database abstractions. Handles initialization of the `.gik.db` file, manages `redb` transactions, and encapsulates all reads/writes.
- `objects`: Core VCS logic including hashing (`sha1`), compression (`flate2`), and Git-canonical blob/commit object formatting.
- `models`: Data structures and schema definitions for the `redb` tables (e.g., `CommitMeta`, `Transaction`).

## 3. Data Storage & Schema
The embedded `redb` database utilizes the following tables:
1. `OBJECTS`: Stores blob contents and commit objects (Key: SHA1, Value: zlib compressed bytes).
2. `COMMITS_METADATA`: Stores structured commit information (`parent_hashes`, `tree_hash`, `timestamp`).
3. `HEADS`: Tracks the active commit hashes (active branches/anonymous heads).
4. `STAGE_INDEX`: Maps file paths to their currently staged SHA1 hash.
5. `TRANSACTION_LOG`: Auto-incrementing log of operations for the `undo` command.

## 4. Data Flow & ACID Transactions
All mutable operations (e.g., `commit`) are strictly atomic. 
When `gik commit` is executed:
1. An atomic `WriteTransaction` is opened via `redb`.
2. Target files are scanned, hashed, and compressed, then written to the `OBJECTS` table.
3. The `COMMITS_METADATA`, `HEADS`, and `STAGE_INDEX` tables are updated accordingly.
4. A serialized inverse action is appended to the `TRANSACTION_LOG` for future `undo` operations.
5. The `redb` transaction is committed. Any error during this process results in a complete rollback, leaving the repository state untouched.

## 5. Error Handling & UX
- Errors will be strongly typed using the `thiserror` crate, categorized into domain-specific variants (e.g., `IoError`, `DbError`, `HashError`, `SerializationError`).
- The CLI UX will follow traditional Unix philosophy: silent on success. Errors will be printed cleanly to stderr.

## 6. Distribution & CI/CD
- **CI/CD:** A GitHub Actions workflow will automatically cross-compile binaries for Linux, macOS, and Windows on release tags.
- **Install Script:** An `install.sh` script will be provided. It will detect the host OS and architecture, download the appropriate pre-compiled binary via `curl`, and install it into a standard bin directory.

## 7. MVP Scope Limitations
- Nested directories are not supported in the MVP; files are treated as a flat list.
- Only the 5 specified commands are implemented (`init`, `stage`, `commit`, `log`, `undo`).
- Hashes are strictly `sha1` for potential forward compatibility with Git objects.