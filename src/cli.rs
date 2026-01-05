use crate::{banner::print_banner, git, prompt, sequence_editor};

use console::style;
use std::{env, path::PathBuf};

/// Repository paths needed for the rewrite operation.
struct RepoPaths {
    root: PathBuf,
    git_dir: PathBuf,
}

/// Verifies git is available and returns repository paths.
fn verify_environment() -> Result<RepoPaths, ()> {
    // Ensure `git` is available.
    match which::which("git") {
        Ok(_) => {}
        Err(_) => {
            eprintln!("{}", style("Error: `git` not found in PATH.").red().bold());
            return Err(());
        }
    }

    // Resolve repository root.
    let root = match git::rev_parse("--show-toplevel") {
        Ok(s) => PathBuf::from(s),
        Err(e) => {
            eprintln!(
                "{}",
                style(format!("Error: not inside a git repo ({})", e))
                    .red()
                    .bold()
            );
            return Err(());
        }
    };

    // Resolve .git directory.
    let git_dir = match git::rev_parse("--git-dir") {
        Ok(s) => {
            let p = PathBuf::from(s);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        }
        Err(e) => {
            eprintln!(
                "{}",
                style(format!("Error: unable to locate .git dir ({})", e))
                    .red()
                    .bold()
            );
            return Err(());
        }
    };

    Ok(RepoPaths { root, git_dir })
}

/// Result of prompting for author input.
enum AuthorInput {
    /// New author values (name, email).
    Changed(String, String),
    /// No changes from defaults.
    NoChange,
}

/// Prompts for author name and email, returning trimmed values or indicating no change.
fn get_author_input(repo_name: &str) -> Result<AuthorInput, ()> {
    let default_name = git::config_get("user.name").unwrap_or_default();
    let default_email = git::config_get("user.email").unwrap_or_default();

    let mut string_prompter = prompt::DialoguerStringPrompter;

    let name = match prompt::ask(&mut string_prompter, "Author name", repo_name, &default_name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", style(format!("Prompt error: {}", e)).red().bold());
            return Err(());
        }
    };

    let email = match prompt::ask(&mut string_prompter, "Author email", repo_name, &default_email) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", style(format!("Prompt error: {}", e)).red().bold());
            return Err(());
        }
    };

    // Check for no changes before trimming.
    if should_exit_no_change(&name, &email, &default_name, &default_email) {
        return Ok(AuthorInput::NoChange);
    }

    Ok(AuthorInput::Changed(
        name.trim().to_string(),
        email.trim().to_string(),
    ))
}

/// Updates git config with the new author values.
fn update_git_config(name: &str, email: &str) -> Result<(), ()> {
    match git::config_set("user.name", name) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "{}",
                style(format!("Failed to set user.name: {}", e))
                    .red()
                    .bold()
            );
            return Err(());
        }
    }
    match git::config_set("user.email", email) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "{}",
                style(format!("Failed to set user.email: {}", e))
                    .red()
                    .bold()
            );
            return Err(());
        }
    }
    Ok(())
}

/// Runs the rebase loop, amending each commit with the new author.
fn run_rebase_loop(git_dir: &PathBuf, author: &str) -> Result<(), ()> {
    loop {
        if !git::rebase_in_progress(git_dir) {
            println!(
                "{}",
                style("✅ Successfully rewrote commit authors.")
                    .green()
                    .bold()
            );
            break;
        }

        match git::amend_author(author) {
            Ok(_) => {
                println!("{}", style("Amended current commit author.").green());
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("❌ Failed to amend commit: {}", e))
                        .red()
                        .bold()
                );
                return Err(());
            }
        }

        match git::rebase_continue() {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("❌ `git rebase --continue` failed: {}", e))
                        .red()
                        .bold()
                );
                return Err(());
            }
        }
    }
    Ok(())
}

/// Determines whether the provided name and email are unchanged from the defaults.
///
/// Both values are compared after trimming leading and trailing whitespace.  
/// Returns `true` only if **both** the `name` and `email` match their corresponding
/// defaults exactly after trimming; otherwise returns `false`.
///
/// # Parameters
///
/// * `name` – The proposed author name.
/// * `email` – The proposed author email.
/// * `default_name` – The current default author name (e.g., from `git config`).
/// * `default_email` – The current default author email.
///
/// # Returns
///
/// * `true` if both name and email match the defaults (ignoring surrounding whitespace).  
/// * `false` otherwise.
///
/// # Examples
///
/// ```ignore
/// // Example (function is crate-private):
/// // use git_author_rewrite::cli::should_exit_no_change;
/// // assert!(should_exit_no_change("Alice ", "alice@example.com ", "Alice", "alice@example.com"));
/// ```
pub(crate) fn should_exit_no_change(
    name: &str,
    email: &str,
    default_name: &str,
    default_email: &str,
) -> bool {
    name.trim() == default_name.trim() && email.trim() == default_email.trim()
}

/// Prints usage information to stdout.
fn print_help() {
    println!(
        "\
git-author-rewrite {}

Rewrite commit authors across an entire Git repository.

USAGE:
    git-author-rewrite [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
    --manual         Edit the rebase todo list manually instead of auto-marking all commits

DESCRIPTION:
    This tool prompts for a new author name and email, then rewrites all commits
    in the repository to use the new author information via interactive rebase.

    In auto mode (default), all commits are automatically marked for editing.
    In manual mode (--manual), you can choose which commits to edit.",
        env!("CARGO_PKG_VERSION")
    );
}

/// Main CLI entry point for `git-author-rewrite`.
///
/// This function:
/// 1. Handles special `--sequence-editor` invocation.
/// 2. Parses CLI flags (currently only `--manual`).
/// 3. Verifies that `git` is installed and that the current directory is a git repository.
/// 4. Prompts for new author name and email (with defaults from `git config`).
/// 5. Exits early if neither name nor email has changed.
/// 6. Updates local git config with new values.
/// 7. Displays an informational banner.
/// 8. Optionally starts an interactive rebase to rewrite commit authors.
///
/// Returns `Ok(exit_code)` on success, or `Err(())` on error.
///
/// # Errors
///
/// Returns `Err(())` in the following cases:
/// - `git` is not found in `PATH`.
/// - The current directory is not a git repository.
/// - Prompts fail.
/// - Updating `git config` fails.
/// - The rebase cannot be started or continued.
///
/// # Exit Codes
///
/// * `0` – Successful execution (including early exit when no changes detected).
/// * Non-zero – Any failure along the way.
pub fn entry() -> Result<i32, ()> {
    // Parse command-line arguments.
    let args: Vec<String> = env::args().collect();

    // Handle --help flag.
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(0);
    }

    // Handle --version flag.
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("git-author-rewrite {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }

    // Special case: act as `git sequence-editor` if invoked with that flag.
    if args.len() >= 2 && args[1] == "--sequence-editor" {
        let path = args.get(2).map(|s| s.as_str());
        match sequence_editor::run(path) {
            Ok(_) => {
                return Ok(0);
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("Sequence editor error: {}", e)).red().bold()
                );
                return Err(());
            }
        }
    }

    // Parse CLI flags.
    let manual_mode = args.iter().any(|a| a == "--manual");

    // Verify environment and get repository paths.
    let paths = verify_environment()?;

    // Get repository name for prompts.
    let repo_name = paths
        .root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("current repository")
        .to_string();

    // Prompt for author input.
    let (name, email) = match get_author_input(&repo_name)? {
        AuthorInput::Changed(n, e) => (n, e),
        AuthorInput::NoChange => {
            eprintln!(
                "{}",
                style("No changes detected for name or email; exiting without modifying history.")
                    .yellow()
                    .bold()
            );
            return Ok(0);
        }
    };

    // Update local git config.
    update_git_config(&name, &email)?;

    // Show banner with instructions.
    print_banner(&name, &email, manual_mode);

    // Confirm before starting rebase.
    let mut confirm_prompter = prompt::DialoguerConfirmPrompter;
    match prompt::confirm_start(&mut confirm_prompter) {
        Ok(true) => {
            // Start interactive rebase (auto-mark commits unless manual mode).
            let auto_mark_all = !manual_mode;
            match git::rebase_interactive(auto_mark_all) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "{}",
                        style(format!("❌ Rebase failed to start: {}", e))
                            .red()
                            .bold()
                    );
                    return Err(());
                }
            }

            // Run the rebase loop.
            let author = format!("{} <{}>", name, email);
            run_rebase_loop(&paths.git_dir, &author)?;
        }
        Ok(false) => {
            println!(
                "{}",
                style("Canceled by user. No changes made.").yellow().bold()
            );
            return Ok(0);
        }
        Err(e) => {
            eprintln!("{}", style(format!("Prompt error: {}", e)).red().bold());
            return Err(());
        }
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::should_exit_no_change;

    #[test]
    fn unchanged_both_returns_true() {
        let r = should_exit_no_change("A ", "b@example.com ", "A", "b@example.com");
        assert_eq!(r, true);
    }

    #[test]
    fn changed_name_only_returns_false() {
        let r = should_exit_no_change("New", "b@example.com", "Old", "b@example.com");
        assert_eq!(r, false);
    }

    #[test]
    fn changed_email_only_returns_false() {
        let r = should_exit_no_change("A", "new@example.com", "A", "old@example.com");
        assert_eq!(r, false);
    }

    #[test]
    fn both_changed_returns_false() {
        let r = should_exit_no_change("X", "y@z", "A", "b@c");
        assert_eq!(r, false);
    }
}
