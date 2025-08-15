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
        let indent = " ".repeat(indent_len);

        return format!("{}edit {}", indent, &trimmed[5..]);
    }

    line.to_string()
}

#[cfg(test)]
mod more_tests {
    use crate::sequence_editor::run;
    use std::io::{Read, Write};

    #[test]
    fn no_picks_is_noop() {
        let tmp = tempfile::NamedTempFile::new();
        match tmp {
            Ok(mut file) => {
                // todo file with only comments and exec lines
                let _ = writeln!(file, "# comment");
                let _ = writeln!(file, "exec echo ok");
                let path = file.path().to_path_buf();

                let r = run(path.to_str());

                match r {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("sequence_editor run failed: {}", e);
                        assert!(false);
                    }
                }

                let mut s = String::new();
                let rd = std::fs::File::open(&path);

                match rd {
                    Ok(mut f) => match f.read_to_string(&mut s) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("read failed: {}", e);
                            assert!(false);
                        }
                    },
                    Err(_) => {
                        assert!(false);
                    }
                }

                assert!(s.contains("exec echo ok"));
                assert!(s.contains("# comment"));
            }
            Err(_) => assert!(false),
        }
    }
}
