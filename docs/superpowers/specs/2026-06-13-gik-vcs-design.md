# Gik - Modern Version Control System in Rust

**Date:** 2026-06-13
**Status:** Approved

## Goal
Build a Git-like version control system (VCS) written in Rust that prioritizes architectural clarity, performance, and developer experience (UX). It should be reliable, safe, and easy to install across different operating systems.

## 1. Core Architecture

### Data Model (DAG)
The system uses a Directed Acyclic Graph (DAG) to represent history, similar to Git:
- **Blob:** Compressed (Zstd) and hashed (SHA-256) file content.
- **Tree:** A snapshot of a directory, mapping filenames to Blob or Tree hashes.
- **Commit:** Metadata (author, date, message) pointing to a Root Tree and one or more parent Commit hashes.
- **Refs:** Named pointers to commit hashes (e.g., branches like `main`).

### Storage Strategy
Instead of thousands of loose files, `gik` uses an embedded Key-Value database:
- **Engine:** `redb` (Rust-native, ACID compliant).
- **Format:** A single `.gik/data.redb` file containing all objects and references.
- **Benefits:** Atomic transactions (prevents corruption), fast indexed lookups, no filesystem "stat" overhead.

### Technologies
- **Language:** Rust (Stable).
- **Hashing:** `sha2` (SHA-256).
- **Compression:** `zstd`.
- **Database:** `redb`.
- **CLI:** `clap` (parsing), `colored` (UI), `comfy-table` (logs).

## 2. User Workflow (Hybrid Staging)

`gik` implements a user-friendly "Hybrid Staging" approach:
1. **Default:** `gik commit -m "msg"` automatically includes all modified and new files.
2. **Control:** Users can use `gik stage <file>` to explicitly select changes, in which case `gik commit` only includes those staged files.
3. **Transparency:** `gik status` clearly shows what will be committed.

## 3. CLI Commands (MVP)

| Command | Description |
|---------|-------------|
| `gik init` | Initialize a new repository and create the `.gik` database. |
| `gik status` | Show changes in the working directory compared to the last commit. |
| `gik stage <path>` | Manually add a file to the next commit. |
| `gik commit -m <msg>` | Save a snapshot of the project. |
| `gik log` | Display history in a clean, readable format. |
| `gik checkout <target>` | Switch the working directory to a specific commit or branch. |

## 4. Distribution & Safety

### Installation
- **Automated CI/CD:** GitHub Actions build binaries for Linux, macOS, and Windows.
- **Install Script:** A `curl | sh` script (and a PowerShell equivalent) to download the correct binary from GitHub Releases.

### Safety & Reliability
- **Memory Safety:** Written in safe Rust (no `unsafe` blocks).
- **Data Integrity:** ACID transactions in `redb` ensure the database is never in a half-written state.
- **Verification:** All objects are verified by their SHA-256 hash upon reading.

## 5. Implementation Stages
1. **Foundation:** Setup Rust project, `redb` integration, and Blob storage.
2. **Structure:** Implement Trees and the hashing/compression pipeline.
3. **History:** Implement Commits and the basic `init`, `status`, `commit` loop.
4. **UX:** Add `log`, `checkout`, and colored CLI output.
5. **Release:** Setup GitHub Actions and the installation script.
