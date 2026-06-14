# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.9](https://github.com/Kyryloloshka/Gik/releases/tag/v0.1.9) - 2026-06-14

### Added

- add GIK_VERSION constant for version reporting
- gitignore
- implement log and undo commands with transaction logging support
- implement commit command with tree and commit object support
- implement stage command
- add contains_object check to storage
- implement init command
- add CLI skeleton with clap subcommands
- implement git-compatible streaming object formatting
- add data models and storage layer
- initialize project and define core error types

### Fixed

- add missing dist profile to Cargo.toml
- move cargo-dist metadata to workspace and add --allow-dirty to CI commands
- enforce workspace structure and add allow-dirty flag to cargo-dist commands
- simplify cargo-dist config and workflow to ensure artifacts are built
- refine release workflow and add manual trigger support
- use PAT for release-plz to allow triggering distribution workflow
- adjust release workflow for better reliability

### Other

- bump version to 0.1.9 for final release attempt
- bump version to 0.1.8 for release
- release v0.1.7
- bump version to 0.1.7 for release
- release v0.1.6
- bump version to 0.1.6 to sync with release automation
- release v0.1.5
- implement automated semantic releases with release-plz
- version to 0.1.5 to match release tag
- pass tag to cargo dist plan to fix empty build matrix
- fix publish-release skip by ensuring GITHUB_TOKEN is used in plan step
- allow dirty CI and align workflow with cargo-dist expectations
- update docs
- remove DISTRIBUTION.md
- replace DISTRIBUTION.md with a user-facing README.md
- update installer URLs in DISTRIBUTION.md
- fix cargo-dist config placement and ensure correct repository URL
- remove internal project documentation and agent definitions from the repository
- implement automated release pipeline with cargo-dist and github actions
- implement dependency injection for storage and clean up tests
- modularize commands into individual files
- introduce strong Hash type and update core components
- introduce Hash type
- break down commit command into helpers and use author constants
- use DB_PATH constant in commands and tests
- centralize configuration and constants
- extract unit tests from objects facade to a dedicated tests.rs file
- modularize objects module into specialized blob, tree, and commit submodules
- improve project structure by separating tests, fixing naming, and ensuring clippy compliance
- update .gitignore formatting and add VSCode settings; add initial repro script for SHA1 hashing
- update architectural rules to reflect new modular structure
- restructure project into core, commands, and cli modules for better scalability
- restructure commands into a dedicated module and align with architectural standards
- add project architectural standards and conventions
- add foundation implementation plan for gik
- add AGENTS.md definition
- refine design with git-compatibility and streaming requirements
- actually add gik vcs design spec
- add gik vcs design spec
- ignore .worktrees directory
- add project design and foundation plan
- add project design and foundation plan

## [0.1.7](https://github.com/Kyryloloshka/Gik/releases/tag/v0.1.7) - 2026-06-14

### Added

- add GIK_VERSION constant for version reporting
- gitignore
- implement log and undo commands with transaction logging support
- implement commit command with tree and commit object support
- implement stage command
- add contains_object check to storage
- implement init command
- add CLI skeleton with clap subcommands
- implement git-compatible streaming object formatting
- add data models and storage layer
- initialize project and define core error types

### Fixed

- simplify cargo-dist config and workflow to ensure artifacts are built
- refine release workflow and add manual trigger support
- use PAT for release-plz to allow triggering distribution workflow
- adjust release workflow for better reliability

### Other

- bump version to 0.1.7 for release
- release v0.1.6
- bump version to 0.1.6 to sync with release automation
- release v0.1.5
- implement automated semantic releases with release-plz
- version to 0.1.5 to match release tag
- pass tag to cargo dist plan to fix empty build matrix
- fix publish-release skip by ensuring GITHUB_TOKEN is used in plan step
- allow dirty CI and align workflow with cargo-dist expectations
- update docs
- remove DISTRIBUTION.md
- replace DISTRIBUTION.md with a user-facing README.md
- update installer URLs in DISTRIBUTION.md
- fix cargo-dist config placement and ensure correct repository URL
- remove internal project documentation and agent definitions from the repository
- implement automated release pipeline with cargo-dist and github actions
- implement dependency injection for storage and clean up tests
- modularize commands into individual files
- introduce strong Hash type and update core components
- introduce Hash type
- break down commit command into helpers and use author constants
- use DB_PATH constant in commands and tests
- centralize configuration and constants
- extract unit tests from objects facade to a dedicated tests.rs file
- modularize objects module into specialized blob, tree, and commit submodules
- improve project structure by separating tests, fixing naming, and ensuring clippy compliance
- update .gitignore formatting and add VSCode settings; add initial repro script for SHA1 hashing
- update architectural rules to reflect new modular structure
- restructure project into core, commands, and cli modules for better scalability
- restructure commands into a dedicated module and align with architectural standards
- add project architectural standards and conventions
- add foundation implementation plan for gik
- add AGENTS.md definition
- refine design with git-compatibility and streaming requirements
- actually add gik vcs design spec
- add gik vcs design spec
- ignore .worktrees directory
- add project design and foundation plan
- add project design and foundation plan

## [0.1.6](https://github.com/Kyryloloshka/Gik/releases/tag/v0.1.6) - 2026-06-14

### Added

- add GIK_VERSION constant for version reporting
- gitignore
- implement log and undo commands with transaction logging support
- implement commit command with tree and commit object support
- implement stage command
- add contains_object check to storage
- implement init command
- add CLI skeleton with clap subcommands
- implement git-compatible streaming object formatting
- add data models and storage layer
- initialize project and define core error types

### Fixed

- use PAT for release-plz to allow triggering distribution workflow
- adjust release workflow for better reliability

### Other

- bump version to 0.1.6 to sync with release automation
- release v0.1.5
- implement automated semantic releases with release-plz
- version to 0.1.5 to match release tag
- pass tag to cargo dist plan to fix empty build matrix
- fix publish-release skip by ensuring GITHUB_TOKEN is used in plan step
- allow dirty CI and align workflow with cargo-dist expectations
- update docs
- remove DISTRIBUTION.md
- replace DISTRIBUTION.md with a user-facing README.md
- update installer URLs in DISTRIBUTION.md
- fix cargo-dist config placement and ensure correct repository URL
- remove internal project documentation and agent definitions from the repository
- implement automated release pipeline with cargo-dist and github actions
- implement dependency injection for storage and clean up tests
- modularize commands into individual files
- introduce strong Hash type and update core components
- introduce Hash type
- break down commit command into helpers and use author constants
- use DB_PATH constant in commands and tests
- centralize configuration and constants
- extract unit tests from objects facade to a dedicated tests.rs file
- modularize objects module into specialized blob, tree, and commit submodules
- improve project structure by separating tests, fixing naming, and ensuring clippy compliance
- update .gitignore formatting and add VSCode settings; add initial repro script for SHA1 hashing
- update architectural rules to reflect new modular structure
- restructure project into core, commands, and cli modules for better scalability
- restructure commands into a dedicated module and align with architectural standards
- add project architectural standards and conventions
- add foundation implementation plan for gik
- add AGENTS.md definition
- refine design with git-compatibility and streaming requirements
- actually add gik vcs design spec
- add gik vcs design spec
- ignore .worktrees directory
- add project design and foundation plan
- add project design and foundation plan

## [0.1.5](https://github.com/Kyryloloshka/Gik/releases/tag/v0.1.5) - 2026-06-14

### Added

- gitignore
- implement log and undo commands with transaction logging support
- implement commit command with tree and commit object support
- implement stage command
- add contains_object check to storage
- implement init command
- add CLI skeleton with clap subcommands
- implement git-compatible streaming object formatting
- add data models and storage layer
- initialize project and define core error types

### Fixed

- adjust release workflow for better reliability

### Other

- implement automated semantic releases with release-plz
- version to 0.1.5 to match release tag
- pass tag to cargo dist plan to fix empty build matrix
- fix publish-release skip by ensuring GITHUB_TOKEN is used in plan step
- allow dirty CI and align workflow with cargo-dist expectations
- update docs
- remove DISTRIBUTION.md
- replace DISTRIBUTION.md with a user-facing README.md
- update installer URLs in DISTRIBUTION.md
- fix cargo-dist config placement and ensure correct repository URL
- remove internal project documentation and agent definitions from the repository
- implement automated release pipeline with cargo-dist and github actions
- implement dependency injection for storage and clean up tests
- modularize commands into individual files
- introduce strong Hash type and update core components
- introduce Hash type
- break down commit command into helpers and use author constants
- use DB_PATH constant in commands and tests
- centralize configuration and constants
- extract unit tests from objects facade to a dedicated tests.rs file
- modularize objects module into specialized blob, tree, and commit submodules
- improve project structure by separating tests, fixing naming, and ensuring clippy compliance
- update .gitignore formatting and add VSCode settings; add initial repro script for SHA1 hashing
- update architectural rules to reflect new modular structure
- restructure project into core, commands, and cli modules for better scalability
- restructure commands into a dedicated module and align with architectural standards
- add project architectural standards and conventions
- add foundation implementation plan for gik
- add AGENTS.md definition
- refine design with git-compatibility and streaming requirements
- actually add gik vcs design spec
- add gik vcs design spec
- ignore .worktrees directory
- add project design and foundation plan
- add project design and foundation plan
