# Gik Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Setup the basic Rust project structure, integrate the `redb` database, and implement the storage of `Blob` objects with SHA-256 hashing and Zstd compression.

**Architecture:** 
- A modular Rust application using `clap` for CLI parsing.
- A central `Database` module that manages `redb` transactions.
- An `Object` storage layer that handles serialization, hashing, and compression.

**Tech Stack:** Rust, redb, sha2, zstd, clap, anyhow, colored.

---

### Task 1: Project Setup and Dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Initialize Cargo project**

Run: `cargo init`

- [ ] **Step 2: Add dependencies to Cargo.toml**

```toml
[package]
name = "gik"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
redb = "1.0"
sha2 = "0.10"
zstd = "0.13"
anyhow = "1.0"
colored = "2.0"
hex = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- [ ] **Step 3: Verify build**

Run: `cargo build`
Expected: Success

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "chore: initial project setup and dependencies"
```

---

### Task 2: Database Layer

**Files:**
- Create: `src/db.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Define the Database structure**

Create `src/db.rs` with basic `redb` initialization logic.

```rust
use anyhow::Result;
use redb::{Database, TableDefinition, WriteStrategy};
use std::path::Path;

const OBJECTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("objects");
const REFS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("refs");

pub struct GikDb {
    db: Database,
}

impl GikDb {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::builder()
            .set_write_strategy(WriteStrategy::TwoPhase)
            .create(path.join("data.redb"))?;
        
        // Ensure tables exist
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(OBJECTS_TABLE)?;
            let _ = write_txn.open_table(REFS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    pub fn put_object(&self, hash: &str, data: &[u8]) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(OBJECTS_TABLE)?;
            table.insert(hash, data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_object(&self, hash: &str) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS_TABLE)?;
        let value = table.get(hash)?;
        Ok(value.map(|v| v.value().to_vec()))
    }
}
```

- [ ] **Step 2: Add test for Database**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_db_put_get() -> Result<()> {
        let dir = tempdir()?;
        let db = GikDb::open(dir.path())?;
        db.put_object("abc", b"data")?;
        let data = db.get_object("abc")?;
        assert_eq!(data, Some(b"data".to_vec()));
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test db`

- [ ] **Step 4: Commit**

```bash
git add src/db.rs
git commit -m "feat: add redb storage layer"
```

---

### Task 3: Object Storage (Blob)

**Files:**
- Create: `src/objects.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement Hashing and Compression**

Create `src/objects.rs` to handle Blob creation.

```rust
use sha2::{Sha256, Digest};
use anyhow::Result;

pub struct Blob {
    pub hash: String,
    pub data: Vec<u8>, // Compressed data
}

impl Blob {
    pub fn new(content: &[u8]) -> Result<Self> {
        // 1. Hash the raw content
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());

        // 2. Compress the content
        let compressed = zstd::encode_all(content, 3)?;

        Ok(Self { hash, data: compressed })
    }

    pub fn decompress(&self) -> Result<Vec<u8>> {
        let decompressed = zstd::decode_all(&self.data[..])?;
        Ok(decompressed)
    }
}
```

- [ ] **Step 2: Add test for Blob**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_compression() -> Result<()> {
        let content = b"hello world";
        let blob = Blob::new(content)?;
        assert_eq!(blob.decompress()?, content);
        Ok(())
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test objects`

- [ ] **Step 4: Commit**

```bash
git add src/objects.rs
git commit -m "feat: add blob objects with hashing and compression"
```

---

### Task 4: CLI - Init Command

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Define CLI structure with Clap**

```rust
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

mod db;
mod objects;

#[derive(Parser)]
#[command(name = "gik")]
#[command(about = "A modern VCS in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new gik repository
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let path = Path::new(".gik");
            if path.exists() {
                println!("Repository already initialized.");
                return Ok(());
            }

            fs::create_dir(path).context("Failed to create .gik directory")?;
            db::GikDb::open(path).context("Failed to initialize database")?;
            
            println!("Initialized empty Gik repository in .gik/");
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Manual verification**

Run: `cargo run -- init`
Check: `ls -a .gik/`

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement gik init command"
```
