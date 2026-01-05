use std::{
    fs::{File, read_to_string},
    io::Write,
    path::Path,
};

/// Entry point to rewrite a todo file by replacing every leading `pick` with `edit`.
///
/// # Arguments
///
/// * `todo_path` - Optional path to the todo file.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err(String)` if the file path is missing or an I/O operation fails.
pub fn run(todo_path: Option<&str>) -> Result<(), String> {
    match todo_path {
        Some(p) => rewrite(Path::new(p)),
        None => Err(String::from("missing todo file path")),
    }
}

/// Reads the file at `path`, replaces every line that starts with `pick`
/// (ignoring leading whitespace and non-comment lines) with `edit`,
/// and writes the updated content back to the file.
///
/// # Arguments
///
/// * `path` - Path to the todo file.
///
/// # Returns
///
/// * `Ok(())` on successful rewrite.
/// * `Err(String)` if an I/O error occurs during reading or writing.
pub fn rewrite(path: &Path) -> Result<(), String> {
    let body = match read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("read failed: {}", e)),
    };

    let transformed = body
        .lines()
        .map(transform_line)
        .collect::<Vec<String>>()
        .join("\n")
        + "\n";

    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("create failed: {}", e)),
    };

    match file.write_all(transformed.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("write failed: {}", e)),
    }
}

/// Converts a single line from a Git rebase todo file.
///
/// - Comment lines (starting with `#`) are returned unchanged.
/// - Lines starting with `pick` (ignoring leading whitespace) are
///   replaced with `edit`, preserving original indentation.
/// - All other lines are returned as-is.
///
/// # Arguments
///
/// * `line` - A single line from the input file.
///
/// # Returns
///
/// * A transformed version of the line, possibly modified.
fn transform_line(line: &str) -> String {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        return line.to_string();
    }

    if trimmed.starts_with("pick ") {
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];

        return format!("{}edit {}", indent, &trimmed[5..]);
    }

    line.to_string()
}

#[cfg(test)]
mod tests {
    use super::{run, transform_line};
    use std::io::{Read, Write};

    #[test]
    fn no_picks_is_noop() {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        writeln!(file, "# comment").expect("failed to write comment");
        writeln!(file, "exec echo ok").expect("failed to write exec line");
        let path = file.path().to_path_buf();

        run(path.to_str()).expect("sequence_editor run failed");

        let mut s = String::new();
        let mut f = std::fs::File::open(&path).expect("failed to open file");
        f.read_to_string(&mut s).expect("failed to read file");

        assert!(s.contains("exec echo ok"));
        assert!(s.contains("# comment"));
    }

    #[test]
    fn transform_line_converts_pick_to_edit() {
        let result = transform_line("pick abc123 Commit message");
        assert_eq!(result, "edit abc123 Commit message");
    }

    #[test]
    fn transform_line_preserves_space_indent() {
        let result = transform_line("  pick abc123 Commit message");
        assert_eq!(result, "  edit abc123 Commit message");
    }

    #[test]
    fn transform_line_preserves_tab_indent() {
        let result = transform_line("\tpick abc123 Commit message");
        assert_eq!(result, "\tedit abc123 Commit message");
    }

    #[test]
    fn transform_line_preserves_mixed_indent() {
        let result = transform_line("\t  pick abc123 Commit message");
        assert_eq!(result, "\t  edit abc123 Commit message");
    }

    #[test]
    fn transform_line_leaves_comments_unchanged() {
        let result = transform_line("# pick abc123 Commit message");
        assert_eq!(result, "# pick abc123 Commit message");
    }

    #[test]
    fn transform_line_leaves_other_commands_unchanged() {
        let result = transform_line("squash abc123 Commit message");
        assert_eq!(result, "squash abc123 Commit message");
    }

    #[test]
    fn run_none_returns_error() {
        let result = run(None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing todo file path");
    }

    #[test]
    fn empty_file_produces_single_newline() {
        let file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = file.path().to_path_buf();

        run(path.to_str()).expect("sequence_editor run failed");

        let mut s = String::new();
        let mut f = std::fs::File::open(&path).expect("failed to open file");
        f.read_to_string(&mut s).expect("failed to read file");

        assert_eq!(s, "\n");
    }
}
