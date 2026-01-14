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
├── tools/     # ユーティリティツール (Rust/Shell)
└── zellij/    # zellij連携設定
    └── layouts/   # zellijレイアウトファイル
```

- `themes/`: Ghosttyの設定ファイル形式のテーマ
- `plugins/`: Ghostty用プラグイン（各プラグインは独立したCargoパッケージ）
- `tools/`: 補助ツール（Rust/Shell）
- `zellij/`: zellij + git worktree連携設定

## Tools

### install-theme.sh
8bit/Retroテーマのインストーラー

### install-zellij-integration.sh
zellij + git worktree並列開発環境のセットアップ

```bash
# インストール
./tools/install-zellij-integration.sh

# アンインストール
./tools/install-zellij-integration.sh --uninstall
```
