//! # git-author-rewrite
//!
//! A CLI tool to rewrite commit authors across an entire Git repository.
//!
//! This crate provides functionality to:
//! - Prompt for new author name and email
//! - Start an interactive rebase from the root commit
//! - Automatically mark all commits for editing
//! - Amend each commit with the new author information
//!
//! ## Usage
//!
//! ```bash
//! # Auto mode: automatically rewrite all commits
//! git-author-rewrite
//!
//! # Manual mode: choose which commits to edit
//! git-author-rewrite --manual
//! ```
//!
//! ## Modules
//!
//! - [`cli`] - Command-line interface and main entry point
//! - [`git`] - Git command wrappers
//! - [`prompt`] - User input abstractions
//! - [`sequence_editor`] - Rebase todo file transformation
//! - [`banner`] - Decorative CLI banner

pub mod banner;
pub mod cli;
pub mod git;
pub mod prompt;
pub mod sequence_editor;
