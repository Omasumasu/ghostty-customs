# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

This repository contains custom tools, plugins, and themes for [Ghostty](https://ghostty.org/) terminal emulator.

## Language and Tooling

- **Primary Language**: Rust
- Use Rust for all plugins and tools unless there's a specific reason to use another language
- Themes and configuration files may use Ghostty's native config format

## Build Commands

```bash
# Build a Rust project
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Project Structure

```
ghostty-customs/
├── themes/    # Ghosttyテーマファイル
├── plugins/   # Ghosttyプラグイン (Rust)
└── tools/     # ユーティリティツール (Rust)
```

- `themes/`: Ghosttyの設定ファイル形式のテーマ
- `plugins/`: Ghostty用プラグイン（各プラグインは独立したCargoパッケージ）
- `tools/`: 補助ツール（各ツールは独立したCargoパッケージ）
