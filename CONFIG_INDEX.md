# Configuration Index

This document provides an overview of key configuration files in the project.

## Root-Level Files

- **Cargo.toml**: Workspace definition, lists `server` member.
- **.gitignore**: Ignores build artifacts and generated directories (`target/`, `build/`, `.trunk/`).
- **rust-toolchain**: Specifies Rust toolchain version (currently empty—consider pinning to stable/nightly).
- **.cargo/config.toml**: Customizes build directory (`build/target`) and sets flags/linkers for MUSL targets (x86_64-unknown-linux-musl, aarch64-unknown-linux-musl).
- **.github/workflows/ci.yml**: Defines CI jobs for style, tests, docs, and push/pull triggers.
- **.github/workflows/release.yml**: Builds/tests the workspace and publishes crates on tag pushes.

## Server Module

- **server/Cargo.toml**: Crate metadata for the HTTP/WebSocket server (version, dependencies, features).
- **server/.trunk/trunk.yaml**: Configuration for the Trunk bundler when serving static assets (themes, keysets).

## Workflows Module

- **workflows/Cargo.toml**: Defines a nested workspace with `workflow-types` and `workflows` crates.
- **workflows/script/**: Contains `bootstrap` and `presubmit` scripts for local setup and checks.
- **workflows/workflow-types/**: Houses shared Rust types for workflow definitions.

---
*Last updated: 2025-04-23T16:52:35+06:00*
