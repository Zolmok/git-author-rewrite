# CLAUDE.md
This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands
```sh
# Build the project
cargo build

# Build release binary
cargo build --release

# Run all tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Install locally
cargo install --path .

# Run the tool
cargo run
cargo run -- --manual
```

## Code Style

- **Always use braces** for control flow blocks (if, else, match arms, loops), even for single-line bodies
- **Never use `?` operator** - always prefer explicit `match` for error handling

## Architecture

This is a Rust CLI tool that rewrites commit authors across an entire Git repository using interactive rebase.

### Module Structure

- **`src/bin/git-author-rewrite.rs`** - Binary entry point, delegates to `cli::entry()`
- **`src/cli.rs`** - Main CLI logic: argument parsing, workflow orchestration, rebase loop
- **`src/git.rs`** - Git command wrappers (`rev_parse`, `config_get/set`, `rebase_interactive`, `amend_author`, `rebase_continue`)
- **`src/prompt.rs`** - User input abstraction with trait-based prompters (`StringPrompter`, `ConfirmPrompter`) for testability
- **`src/sequence_editor.rs`** - Rewrites rebase todo files, replacing `pick` with `edit`
- **`src/banner.rs`** - Colorized CLI banner with box-drawing characters

### Key Design Patterns

**Self-invoking sequence editor**: The binary is used as `GIT_SEQUENCE_EDITOR` during rebase. When called with `--sequence-editor <path>`, it rewrites the todo file instead of running the normal CLI flow.

**Trait-based prompts**: `StringPrompter` and `ConfirmPrompter` traits allow mocking user input in tests. Production uses `DialoguerStringPrompter` and `DialoguerConfirmPrompter`.

**Rebase loop**: After starting `git rebase -i --root`, the CLI enters a loop that:
1. Checks if rebase is in progress (via `.git/rebase-merge` or `.git/rebase-apply`)
2. Amends the current commit's author
3. Continues the rebase
4. Repeats until complete

### Dependencies

- `console` - Terminal styling and text width measurement
- `dialoguer` - Interactive prompts
- `which` - Finds `git` in PATH
- `tempfile` (dev) - Test fixtures
