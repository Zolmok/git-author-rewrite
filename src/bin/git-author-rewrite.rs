/// Entry point for the `git-author-rewrite` binary.
///
/// Delegates to the CLI entry function and exits the process with the
/// returned exit code. If an error occurs, exits with status code 1.
fn main() {
    match git_author_rewrite::cli::entry() {
        Ok(code) => std::process::exit(code),
        Err(_) => std::process::exit(1),
    }
}
