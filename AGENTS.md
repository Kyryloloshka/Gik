# Gik VCS Project Agents

This document defines the roles and responsibilities for agents working on the `gik` version control system.

## 1. System Architect (Current Role)
- **Responsibility**: Research, design, and planning.
- **Focus**: Ensuring architectural integrity, Git compatibility, and ACID compliance.
- **Output**: Design specs (`docs/superpowers/specs/`) and Implementation plans (`docs/superpowers/plans/`).

## 2. Core Engineer (Implementation Agent)
- **Responsibility**: Executing the implementation plans task-by-task.
- **Focus**: Writing high-quality Rust code, implementing streaming IO, and ensuring strict binary compatibility with Git formats.
- **Tools**: Rust toolchain, `cargo`, `redb`.

## 3. QA & Validation Agent
- **Responsibility**: Verifying implementation against requirements.
- **Focus**: Integration testing, ensuring rollback safety (ACID), and validating SHA1 compatibility with Git.
- **Tools**: Rust test framework, custom reproduction scripts.

## 4. DevOps Agent
- **Responsibility**: CI/CD and distribution.
- **Focus**: GitHub Actions, cross-compilation, and `install.sh` maintenance.
