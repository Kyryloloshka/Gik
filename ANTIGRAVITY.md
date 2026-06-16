# Gik: Antigravity Transition Context 🛸

This document serves as a comprehensive handoff for Antigravity CLI. It summarizes the current state of the **Gik** project (v0.1.39) to ensure seamless continuity.

## 🏗 Project Essence
**Gik** is a Git-compatible transactional version control system built in Rust on top of the `redb` ACID database. It uses canonical Git formats for interoperability but focuses on modern workflows like Jujutsu-style bookmarks.

## 🗺 Current Architecture (Phase: Solid Foundation)
We use a **Layered Clean Architecture** (Service/Repository pattern):
- **Command Layer (`src/commands/`)**: Modularized subdirectories (one per command). Each has its own `mod.rs` (logic) and `tests.rs`.
- **Core Layer (`src/core/`)**:
  - `storage/`: `Repository` (raw DB access) and `services/` (decomposed business logic: Index, Commit, Undo, Object, Ref, Config, Session).
  - `objects/`: Git-canonical hashing and compression (Blobs, Trees, Commits).
  - `workspace/`: File system operations, recursive scanning, and restoration logic.
- **CLI Layer (`src/cli.rs`)**: `clap`-based parsing.
- **Main Router (`src/main.rs`)**: Handles repo-root discovery and command dispatch.

## 🚀 Recent Accomplishments
1. **Smart Branching (Jujutsu-style)**:
   - Floating bookmarks that move automatically on commit.
   - Session-based "bookmark hints" to prevent accidental multi-branch movement.
   - `gik log --graph` for visual DAG representation.
2. **Time Travel & Recovery**:
   - `gik checkout <hash/name>`: Full workspace restoration.
   - `gik restore <path>`: Discard local changes.
   - `gik undo`: Transactional rollback that also synchronizes files on disk.
3. **Smart Staging**:
   - `gik stage .` and directory support.
   - Explicit staging of file deletions.
4. **Professional UX**:
   - Subdirectory support (repo root discovery).
   - Independent config system with `gik config --import-git`.
   - Clean, non-wrapped error messages.

## 🛠 Engineering Standards
- **Idiomatic Rust**: Strict adherence to `cargo clippy`, zero-cost abstractions, and borrowing (`&str`, `&[u8]`).
- **RAII**: Used for database transactions and test environment isolation (`TestEnv`).
- **Safety**: No `panic!`, `unwrap()`, or `expect()` in production code; everything is handled via `Result`.
- **Tests**: 44 passing integration tests with full environment isolation.

## 📋 Next in Roadmap (Phase 2: Joining the Dots)
- [ ] **Step 1: Simple Merge**: Fast-forward merging between bookmarks.
- [ ] **Step 2: 3-way Merge**: Automatic merging of non-conflicting changes.
- [ ] **Step 3: Garbage Collection**: `gik gc` to clean up orphaned objects from the DB.
- [ ] **Step 4: Refined Visuals**: Enhancing the ASCII graph and status output.

## 🔗 Migration Note
To start using Antigravity CLI in this repo:
1. Install: `irm https://antigravity.google/cli/install.ps1 | iex` (Windows)
2. Run: `agy` in the project root.
3. The `agy` agent will read this file and be fully briefed.

---
*Signed by: Gemini CLI Assistant*
