use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Builds the value for the `GIT_SEQUENCE_EDITOR` environment variable.
///
/// Wraps `exe_path` in quotes if it contains spaces, and appends the `--sequence-editor`
/// argument.
///
/// # Examples
///
/// ```ignore
/// let path = "/usr/local/bin/git-author-rewrite";
/// assert_eq!(
///     build_sequence_editor_env(path),
///     "/usr/local/bin/git-author-rewrite --sequence-editor"
/// );
///
/// let path_with_space = "/path/with space/git-author-rewrite";
/// assert_eq!(
///     build_sequence_editor_env(path_with_space),
///     "\"/path/with space/git-author-rewrite\" --sequence-editor"
/// );
/// ```
pub(crate) fn build_sequence_editor_env(exe_path: &str) -> String {
    let quoted = if exe_path.contains(' ') {
        format!("\"{}\"", exe_path)
    } else {
        exe_path.to_string()
    };

    format!("{quoted} --sequence-editor")
}

/// Runs a Git (or other) command and returns only its exit status.
///
/// This function executes the provided [`std::process::Command`] and:
/// - Returns `Ok(())` if the command exits successfully (status code `0`).
/// - Returns `Err("non-zero exit")` if the command exits with a non-zero status.
/// - Returns `Err` containing the I/O error message if the process fails to start.
///
/// # Parameters
///
/// * `cmd` — A fully configured [`std::process::Command`] to run.
///
/// # Returns
///
/// * `Ok(())` if the command succeeded.
/// * `Err(String)` with either `"non-zero exit"` or an error message if it failed.
///
/// # Examples
///
/// ```ignore
/// use std::process::Command;
///
/// let cmd = Command::new("git").arg("status");
/// match run_status(cmd) {
///     Ok(()) => println!("Git command succeeded"),
///     Err(e) => eprintln!("Git command failed: {}", e),
/// }
/// ```
fn run_status(mut cmd: Command) -> Result<(), String> {
    let status_res = cmd.status();

    match status_res {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(String::from("non-zero exit"))
            }
        }
        Err(e) => Err(format!("{}", e)),
    }
}

/// Runs a command and returns its trimmed standard output on success,  
/// or its standard error as an `Err` on failure.
///
/// This function executes the provided [`std::process::Command`] and:
/// - If the command exits with a zero status, its `stdout` is captured,
///   converted to UTF-8 (lossy), trimmed, and returned as `Ok(String)`.
/// - If the command exits non-zero, its `stderr` is captured,
///   converted to UTF-8 (lossy), trimmed, and returned as `Err(String)`.
/// - If the process fails to spawn, the I/O error message is returned as `Err(String)`.
///
/// # Parameters
///
/// * `cmd` — A fully configured [`std::process::Command`] ready to execute.
///
/// # Returns
///
/// * `Ok(String)` containing trimmed `stdout` if the command succeeded.
/// * `Err(String)` containing trimmed `stderr` or I/O error message otherwise.
///
/// # Examples
///
/// ```ignore
/// // This example is illustrative only; it won't run in doctests because
/// // this function is crate-private and may depend on environment state.
/// use std::process::Command;
/// let cmd = Command::new("git").arg("rev-parse").arg("--show-toplevel");
/// match run_output(cmd) {
///     Ok(path) => println!("Repo root: {}", path),
///     Err(err) => eprintln!("Git error: {}", err),
/// }
/// ```
fn run_output(mut cmd: Command) -> Result<String, String> {
    let out_res = cmd.output();
    match out_res {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
            }
        }
        Err(e) => Err(format!("{}", e)),
    }
}

/// Runs `git rev-parse <flag>` and returns its output as a trimmed string.
///
/// This is a convenience wrapper around `git rev-parse` that captures `stdout`
/// or returns `stderr` as an error. It is typically used to query repository
/// metadata such as the repository root or `.git` directory path.
///
/// # Parameters
///
/// * `flag` — The argument to pass to `git rev-parse`, e.g. `--show-toplevel`
///   or `--git-dir`.
///
/// # Returns
///
/// * `Ok(String)` containing the trimmed standard output if the command
///   completed successfully.
/// * `Err(String)` containing the trimmed standard error or an I/O error message
///   if the command failed.
///
/// # Examples
///
/// ```ignore
/// // This example is ignored because it depends on being inside a Git repository.
/// use mycrate::git::rev_parse;
///
/// match rev_parse("--show-toplevel") {
///     Ok(path) => println!("Repository root: {}", path),
///     Err(err) => eprintln!("Git error: {}", err),
/// }
/// ```
pub fn rev_parse(flag: &str) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("rev-parse").arg(flag);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    run_output(cmd)
}

/// Runs `git config --get <key>` and returns the result as a trimmed string.
///
/// This function retrieves a Git configuration value for the specified key.
/// If the key does not exist or the command fails, it returns an empty string
/// instead of an error.
///
/// # Parameters
///
/// * `key` — The Git configuration key to query (e.g. `"user.name"` or `"user.email"`).
///
/// # Returns
///
/// * `Ok(String)` containing the trimmed config value, or an empty string if the key
///   is missing or the command failed.
/// * `Err(String)` is never returned — errors are converted into `Ok(String::new())`.
///
/// # Examples
///
/// ```ignore
/// // Ignored because it requires a Git repository with a configured user.name.
/// use mycrate::git::config_get;
///
/// match config_get("user.name") {
///     Ok(name) if !name.is_empty() => println!("User name: {}", name),
///     Ok(_) => println!("No user name configured."),
///     Err(_) => unreachable!(), // This function never returns Err
/// }
/// ```
pub fn config_get(key: &str) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("config").arg("--get").arg(key);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let res = run_output(cmd);
    match res {
        Ok(s) => Ok(s),
        Err(_) => Ok(String::new()),
    }
}

/// Sets a Git configuration key to the given value in the local repository.
///
/// This function runs `git config <key> <value>` without specifying `--global`,
/// so the change applies only to the current repository.
///
/// # Parameters
///
/// * `key` — The Git configuration key to set (e.g. `"user.name"`).
/// * `value` — The value to assign to the configuration key.
///
/// # Returns
///
/// * `Ok(())` if the configuration was set successfully.
/// * `Err(String)` containing an error message if the command failed.
///
/// # Notes
///
/// This modifies the repository's **local** `.git/config` file.
/// It does not affect global or system-level configuration.
///
/// # Examples
///
/// ```ignore
/// // Ignored because it requires a Git repository.
/// use mycrate::git::config_set;
///
/// if let Err(err) = config_set("user.name", "Jane Doe") {
///     eprintln!("Failed to set Git config: {}", err);
/// }
/// ```
pub fn config_set(key: &str, value: &str) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.arg("config").arg(key).arg(value);
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());
    run_status(cmd)
}

/// Runs an interactive rebase from the root commit, optionally auto-marking all commits for editing.
///
/// Internally, this executes:
///
/// ```text
/// git rebase -i --root
/// ```
///
/// If `auto_mark_all` is `true`, the `GIT_SEQUENCE_EDITOR` environment variable is set
/// so that every `pick` line in the rebase todo list is replaced with `edit` automatically.
/// This allows for non-interactive author rewriting across all commits.
///
/// # Parameters
///
/// * `auto_mark_all` – If `true`, configure `GIT_SEQUENCE_EDITOR` to mark all commits as `edit`.
///   If `false`, the user will manually choose which commits to edit in their editor.
///
/// # Returns
///
/// * `Ok(())` if the command ran successfully.
/// * `Err(String)` if the executable could not be located or if `git rebase` exited with a non-zero status.
///
/// # Notes
///
/// * This command modifies commit history; it should be run only on branches
///   where rewriting is safe.
/// * The process inherits standard input/output/error so the user can interact with Git normally.
/// * Requires the current working directory to be inside a Git repository.
///
/// # Examples
///
/// ```ignore
/// // Ignored because it requires a Git repository.
/// use mycrate::git::rebase_interactive;
///
/// // Automatically mark all commits for editing
/// if let Err(err) = rebase_interactive(true) {
///     eprintln!("Rebase failed: {}", err);
/// }
/// ```
pub fn rebase_interactive(auto_mark_all: bool) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.arg("rebase").arg("-i").arg("--root");
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    if auto_mark_all {
        let exe_res = std::env::current_exe();
        match exe_res {
            Ok(path) => {
                // Quote the path if it contains spaces to avoid shell parsing issues.
                let p = path.to_string_lossy();
                let se = build_sequence_editor_env(&p);

                cmd.env("GIT_SEQUENCE_EDITOR", se);
            }
            Err(e) => {
                return Err(format!("cannot locate current executable: {}", e));
            }
        }
    }

    run_status(cmd).map_err(|_| String::from("`git rebase -i --root` exited with non-zero status"))
}

/// Amends the current commit to set a new author without changing the commit message.
///
/// This runs:
///
/// ```text
/// git commit --amend --author="<author>" --no-edit
/// ```
///
/// The `--no-edit` flag ensures that the commit message remains unchanged.
/// Standard input, output, and error are inherited so the command can prompt
/// the user or show Git's output directly.
///
/// # Parameters
///
/// * `author` – A full author string in the format `"Name <email@example.com>"`.
///
/// # Returns
///
/// * `Ok(())` if the commit was successfully amended.
/// * `Err(String)` if the Git command failed or exited with a non-zero status.
///
/// # Notes
///
/// * Must be run inside a Git repository with a commit to amend.
/// * This rewrites history; only use on branches where rewriting is safe.
/// * The new author will replace the current commit's author.
///
/// # Examples
///
/// ```ignore
/// // Ignored because it requires a Git repository
/// use mycrate::git::amend_author;
///
/// if let Err(err) = amend_author("John Doe <john@example.com>") {
///     eprintln!("Failed to amend author: {}", err);
/// }
/// ```
pub fn amend_author(author: &str) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.arg("commit")
        .arg("--amend")
        .arg(format!("--author={}", author))
        .arg("--no-edit");
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    run_status(cmd).map_err(|_| String::from("`git commit --amend` returned non-zero"))
}

/// Continues an in-progress interactive rebase.
///
/// This runs:
///
/// ```text
/// git rebase --continue
/// ```
///
/// Standard input, output, and error are inherited so that Git can prompt
/// the user or display its normal progress messages.
///
/// # Returns
///
/// * `Ok(())` if the rebase continued successfully.
/// * `Err(String)` if the command failed or exited with a non-zero status.
///
/// # Notes
///
/// * Must be run inside a Git repository with a rebase in progress.
/// * The rebase will continue from the current stop point, applying the next
///   commit in sequence.
///
/// # Examples
///
/// ```ignore
/// // Ignored because it requires a Git repository and an active rebase
/// use mycrate::git::rebase_continue;
///
/// if let Err(err) = rebase_continue() {
///     eprintln!("Failed to continue rebase: {}", err);
/// }
/// ```
pub fn rebase_continue() -> Result<(), String> {
    let mut cmd = Command::new("git");

    cmd.arg("rebase").arg("--continue");
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    run_status(cmd).map_err(|_| String::from("`git rebase --continue` returned non-zero"))
}

/// Detects if a Git rebase is currently in progress.
///
/// This checks for the presence of the `rebase-merge` or `rebase-apply`
/// directories inside `.git/`, which are created by Git during an interactive
/// or apply-style rebase.
///
/// # Parameters
///
/// * `git_dir` – Path to the `.git` directory of the repository.
///
/// # Returns
///
/// * `true` if either `rebase-merge` or `rebase-apply` exists.
/// * `false` otherwise.
///
/// # Notes
///
/// * This is a lightweight check that does not invoke Git directly.
/// * Both interactive (`rebase-merge`) and apply-style (`rebase-apply`) rebases
///   are detected.
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
/// use mycrate::git::rebase_in_progress;
///
/// let git_dir = Path::new(".git");
/// if rebase_in_progress(git_dir) {
///     println!("A rebase is currently in progress.");
/// }
/// ```
pub fn rebase_in_progress(git_dir: &Path) -> bool {
    let merge = PathBuf::from(git_dir).join("rebase-merge");
    let apply = PathBuf::from(git_dir).join("rebase-apply");

    if merge.exists() {
        true
    } else {
        if apply.exists() { true } else { false }
    }
}

#[cfg(test)]
mod tests {
    use super::build_sequence_editor_env;
    use super::rebase_in_progress;
    use std::fs;
    use std::path::Path;

    #[test]
    fn sequence_editor_quotes_when_needed() {
        let s = build_sequence_editor_env("/Users/me/My App/bin");
        assert!(s.starts_with("\"/Users/me/My App/bin\" --sequence-editor"));
    }

    #[test]
    fn sequence_editor_no_quotes_when_no_space() {
        let s = build_sequence_editor_env("/usr/local/bin/myapp");
        assert!(s.starts_with("/usr/local/bin/myapp --sequence-editor"));
    }

    #[test]
    fn rebase_progress_detection_smoke() {
        let tmp = tempfile::tempdir();
        match tmp {
            Ok(dir) => {
                let git_dir = dir.path().join(".git");
                let mk = fs::create_dir_all(&git_dir);
                match mk {
                    Ok(_) => {}
                    Err(_) => {
                        assert!(false);
                    }
                }
                assert_eq!(rebase_in_progress(Path::new(&git_dir)), false);
                let mk2 = fs::create_dir_all(git_dir.join("rebase-merge"));
                match mk2 {
                    Ok(_) => {}
                    Err(_) => {
                        assert!(false);
                    }
                }
                assert_eq!(rebase_in_progress(Path::new(&git_dir)), true);
            }
            Err(_) => assert!(false),
        }
    }
}
