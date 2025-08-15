use crate::{banner::print_banner, git, prompt, sequence_editor};

use console::style;
use std::{env, path::PathBuf};

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
    if name.trim() == default_name.trim() {
        if email.trim() == default_email.trim() {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
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
    // Special case: act as `git sequence-editor` if invoked with that flag.
    // This is used internally by git during interactive rebases.
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        if args[1] == "--sequence-editor".to_string() {
            let path = if args.len() >= 3 {
                Some(args[2].clone())
            } else {
                None
            };
            match sequence_editor::run(path.as_deref()) {
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
    }

    // Parse CLI flags.
    let manual_mode = args.iter().any(|a| a == "--manual");

    // Ensure `git` is available.
    match which::which("git") {
        Ok(_) => {}
        Err(_) => {
            eprintln!("{}", style("Error: `git` not found in PATH.").red().bold());
            return Err(());
        }
    }

    // Resolve repository paths.
    let repo_root = match git::rev_parse("--show-toplevel") {
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

    let git_dir = match git::rev_parse("--git-dir") {
        Ok(s) => {
            let p = PathBuf::from(s);
            if p.is_absolute() {
                p
            } else {
                repo_root.join(p)
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

    // Get default name/email from local git config.
    let default_name = match git::config_get("user.name") {
        Ok(v) => v,
        Err(_) => String::new(),
    };
    let default_email = match git::config_get("user.email") {
        Ok(v) => v,
        Err(_) => String::new(),
    };

    // Prompt user for name/email, using defaults when available.
    let repo_name = repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("current repository")
        .to_string();

    let mut string_prompter = prompt::DialoguerStringPrompter;
    let mut confirm_prompter = prompt::DialoguerConfirmPrompter;

    let name = match prompt::ask(
        &mut string_prompter,
        "Author name",
        &repo_name,
        &default_name,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", style(format!("Prompt error: {}", e)).red().bold());
            return Err(());
        }
    };

    let email = match prompt::ask(
        &mut string_prompter,
        "Author email",
        &repo_name,
        &default_email,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", style(format!("Prompt error: {}", e)).red().bold());
            return Err(());
        }
    };

    // Early exit: if neither value changed, do nothing.
    let name_trimmed = name.trim().to_string();
    let email_trimmed = email.trim().to_string();
    let default_name_trimmed = default_name.trim().to_string();
    let default_email_trimmed = default_email.trim().to_string();

    // Early exit if there are no changes to apply.
    if should_exit_no_change(
        &name_trimmed,
        &email_trimmed,
        &default_name_trimmed,
        &default_email_trimmed,
    ) {
        eprintln!(
            "{}",
            style("No changes detected for name or email; exiting without modifying history.")
                .yellow()
                .bold()
        );
        return Ok(0);
    }

    // Trim whitespace from inputs before storing/using.
    let name = name_trimmed;
    let email = email_trimmed;

    // Update local git config.
    match git::config_set("user.name", &name) {
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
    };
    match git::config_set("user.email", &email) {
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
    };

    // Show banner with instructions.
    print_banner(&name, &email, manual_mode);

    // Confirm before starting rebase.
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

            // Main loop: amend each commit and continue rebase.
            loop {
                if !git::rebase_in_progress(&git_dir) {
                    println!(
                        "{}",
                        style("✅ Successfully rewrote commit authors.")
                            .green()
                            .bold()
                    );
                    break;
                }

                let author = format!("{} <{}>", name, email);
                match git::amend_author(&author) {
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
