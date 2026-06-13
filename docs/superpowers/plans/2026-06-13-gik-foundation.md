# Gik Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the project foundation, data models, and database storage layer for Gik.

**Architecture:** Layered Rust project with a focus on `redb` for ACID storage and `thiserror` for robust error handling.

**Tech Stack:** Rust, redb, serde, bincode, sha1, flate2, thiserror.

---

### Task 1: Project Initialization & Error Types

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/error.rs`

- [ ] **Step 1: Initialize Cargo project with dependencies**

```toml
[package]
name = "gik"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
redb = "1.1"
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
sha1 = "0.10"
flate2 = "1.0"
thiserror = "1.0"
chrono = "0.4"
```

- [ ] **Step 2: Define core error types in `src/error.rs`**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GikError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] redb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Invalid hash length")]
    InvalidHash,
    // ... other types
}

pub type Result<T> = std::result::Result<T, GikError>;
```

- [ ] **Step 3: Basic main.rs entry point**

- [ ] **Step 4: Commit**

### Task 2: Data Models & Database Schema

**Files:**
- Create: `src/models.rs`
- Create: `src/storage.rs`

- [ ] **Step 1: Define CommitMeta and Transaction models in `src/models.rs`**

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitMeta {
    pub parent_hashes: Vec<[u8; 20]>,
    pub tree_hash: [u8; 20],
    pub timestamp: u64,
}
```

- [ ] **Step 2: Define redb table definitions in `src/storage.rs`**

- [ ] **Step 3: Implement Database initialization logic**

- [ ] **Step 4: Commit**

### Task 3: Git-Compatible Object Formatting (Streaming)

**Files:**
- Create: `src/objects.rs`

- [ ] **Step 1: Implement streaming SHA1 hasher for files**
- [ ] **Step 2: Implement Git-canonical blob/commit header formatting**
- [ ] **Step 3: Implement streaming Zlib compression using flate2**
- [ ] **Step 4: Write unit tests for hash compatibility**
- [ ] **Step 5: Commit**

### Task 4: CLI Skeleton (Clap)

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Define Subcommands (init, stage, commit, log, undo)**
- [ ] **Step 2: Connect CLI to empty command handlers**
- [ ] **Step 3: Commit**
