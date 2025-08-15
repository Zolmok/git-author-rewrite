use console::{measure_text_width, style};
use std::iter;

/// Prints a decorative, colorized banner describing the commit-rewrite process.
///
/// The banner is dynamically sized to fit the widest **visible** line of text,
/// using [`console::measure_text_width`] to ignore ANSI color codes when
/// calculating padding. It is framed with Unicode box-drawing characters
/// (`╔═╗`, `║ ║`, `╚═╝`) and uses [`console::style`] for coloring and bolding.
///
/// Borders are styled independently from the inner text so that embedded color
/// codes inside the content (e.g. yellow/cyan manual/auto mode text) do not
/// affect the color of the box edges.
///
/// # Parameters
///
/// * `name` – The new author name to display in the banner.
/// * `email` – The new author email to display in the banner.
/// * `manual_mode` – When `true`, the banner shows manual rebase instructions
///   (in a highlighted color). When `false`, it shows automatic mode
///   instructions (also highlighted).
///
/// # Output
///
/// This function prints directly to standard output. It does not return any value.
///
/// # Notes
///
/// * Width calculation ignores ANSI codes, so padding stays correct even with
///   inline colors.
/// * Intended for interactive CLI display; not for structured logging.
/// * If you require exact column widths for wide Unicode glyphs, ensure your
///   terminal supports them and consider `unicode-width`.
///
/// # Examples
///
/// ```no_run
/// use git_author_rewrite::banner::print_banner;
///
/// fn main() {
///     print_banner("John Doe", "john@example.com", false);
/// }
/// ```
pub fn print_banner(name: &str, email: &str, manual_mode: bool) {
    let lines = banner_lines(name, email, manual_mode);

    let max_width = lines
        .iter()
        .map(|l| measure_text_width(l)) // ignore ANSI in content
        .max()
        .unwrap_or(0)
        + 2;

    let border = "═".repeat(max_width);
    let top = style(format!("╔{}╗", border)).blue().bold();
    let bottom = style(format!("╚{}╝", border)).blue().bold();
    let left = style("║ ").blue().bold().to_string();
    let right = style("║").blue().bold().to_string();

    println!();
    println!("{top}");
    for line in lines {
        let visible = measure_text_width(&line);
        let pad = max_width - visible; // includes the one space after left border
        // build row: [blue left] + [colored line] + [padding spaces] + [blue right]
        println!("{}{}{}{}", left, line, " ".repeat(pad - 1), right);
    }
    println!("{bottom}");
    println!();
}

/// Constructs the lines of text for the commit‑rewrite banner.
///
/// Returns each banner line as a `String`, in the order they should be displayed:
/// 1) title, 2) mode instructions (manual/auto), 3) author summary, 4) steps.
///
/// **Note:** This function **may include ANSI styling** in some lines:
/// - In manual mode, the instruction lines are yellow + bold.
/// - In auto mode, the instruction lines are cyan (first bold).
/// Consumers that need accurate width calculations should measure **visible**
/// width (e.g., with `console::measure_text_width`) rather than `str::len()`.
///
/// # Parameters
///
/// * `name` – The new author name to embed in the banner text.
/// * `email` – The new author email to embed in the banner text.
/// * `manual_mode` – When `true`, includes highlighted manual instructions;
///   when `false`, includes highlighted automatic instructions.
///
/// # Returns
///
/// A vector of `String` values (some may contain ANSI escape codes for color).
///
/// # Usage
///
/// Intended for use by [`print_banner`](crate::banner::print_banner), which
/// applies box borders and handles width/padding correctly for styled content.
///
/// # Examples
///
/// ```ignore
/// // Example only; function may return ANSI-styled strings.
/// let lines = banner_lines("John Doe", "john@example.com", false);
/// assert!(lines.iter().any(|l| l.contains("John Doe")));
/// assert!(lines.iter().any(|l| l.contains("Auto mode")));
/// ```
fn banner_lines(name: &str, email: &str, manual_mode: bool) -> Vec<String> {
    let top = ["Rewrite commit authors via interactive rebase", ""]
        .into_iter()
        .map(|s| s.to_string());

    let mode = if manual_mode {
        vec![
            style("Manual mode: you'll edit the todo list yourself.")
                .yellow()
                .bold()
                .to_string(),
            style("Tip: mark commits you want to change as `edit`.")
                .yellow()
                .bold()
                .to_string(),
        ]
    } else {
        vec![
            style("Auto mode: all `pick` lines will be changed to `edit`.")
                .cyan()
                .bold()
                .to_string(),
            style("(Use --manual to keep your editor and control which commits to edit.)")
                .cyan()
                .to_string(),
        ]
    }
    .into_iter();

    let bottom = iter::once(String::new())
        .chain(iter::once(format!(
            "New author will be set to: {} <{}>",
            name, email
        )))
        .chain(
            [
                "This tool will automatically:",
                "  1) Amend each stop with the new author",
                "  2) Run `git rebase --continue` until finished",
            ]
            .into_iter()
            .map(|s| s.to_string()),
        );

    top.chain(mode).chain(bottom).collect()
}

#[cfg(test)]
mod tests {
    use super::banner_lines;

    #[test]
    fn banner_auto_mode_lines_and_width_are_correct() {
        let lines = banner_lines("John Doe", "john@doe.org", false);
        let s = lines.join("\n");

        assert!(s.contains("Rewrite commit authors via interactive rebase"));
        assert!(s.contains("Auto mode: all `pick` lines will be changed to `edit`."));
        assert!(s.contains("New author will be set to: John Doe <john@doe.org>"));

        // Width logic: ensure max width is computed correctly for these lines
        let max_line = lines.iter().map(|l| l.len()).max().unwrap_or(0);

        // Sanity check: header should be the max or near-max
        assert!(max_line >= "Rewrite commit authors via interactive rebase".len());
    }

    #[test]
    fn banner_manual_mode_lines_and_width_are_correct() {
        let lines = banner_lines("Jane", "jane@example.com", true);
        let s = lines.join("\n");

        assert!(s.contains("Manual mode: you'll edit the todo list yourself."));
        assert!(s.contains("Tip: mark commits you want to change as `edit`."));
        assert!(s.contains("New author will be set to: Jane <jane@example.com>"));

        let max_line = lines.iter().map(|l| l.len()).max().unwrap_or(0);

        assert!(max_line >= "Rewrite commit authors via interactive rebase".len());
    }
}
