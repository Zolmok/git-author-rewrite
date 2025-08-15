use dialoguer::{Confirm, Input, theme::ColorfulTheme};

/// Abstraction over a string input prompt.
///
/// Implementors define how string input is collected from the user,
/// including any styling or interactivity. This trait enables testability
/// by decoupling user input from the logic that consumes it.
pub trait StringPrompter {
    /// Prompt the user for a string input.
    ///
    /// # Parameters
    /// - `prompt`: The message shown to the user.
    /// - `default`: Default value if the user presses Enter without input.
    ///
    /// # Returns
    /// `Ok(String)` if input is successfully collected, or an `Err(String)` describing the failure.
    fn prompt(&mut self, prompt: &str, default: &str) -> Result<String, String>;
}

/// Abstraction over a boolean (yes/no) confirmation prompt.
///
/// This trait allows interactive confirmation to be injected or mocked,
/// promoting testability in CLI workflows.
pub trait ConfirmPrompter {
    /// Prompt the user for a yes/no confirmation.
    ///
    /// # Parameters
    /// - `prompt`: The confirmation message.
    /// - `default`: The default answer if the user presses Enter.
    ///
    /// # Returns
    /// `Ok(true)` if confirmed, `Ok(false)` if declined, or `Err(String)` on input failure.
    fn confirm(&mut self, prompt: &str, default: bool) -> Result<bool, String>;
}

/// Default implementation of `StringPrompter` using `dialoguer::Input`.
///
/// Uses the `ColorfulTheme` for user-friendly styling.
pub struct DialoguerStringPrompter;

impl StringPrompter for DialoguerStringPrompter {
    fn prompt(&mut self, prompt: &str, default: &str) -> Result<String, String> {
        let theme = ColorfulTheme::default();
        let input = Input::<String>::with_theme(&theme)
            .with_prompt(prompt)
            .default(default.to_string());
        match input.interact_text() {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Default implementation of `ConfirmPrompter` using `dialoguer::Confirm`.
///
/// Displays a yes/no dialog with styling from `ColorfulTheme`.
pub struct DialoguerConfirmPrompter;

impl ConfirmPrompter for DialoguerConfirmPrompter {
    fn confirm(&mut self, prompt: &str, default: bool) -> Result<bool, String> {
        let theme = ColorfulTheme::default();
        let confirm = Confirm::with_theme(&theme)
            .with_prompt(prompt)
            .default(default);
        match confirm.interact() {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Prompt the user for an input string, including context from a repository name.
///
/// Wraps the `StringPrompter` trait and constructs a prompt of the form:
/// `"Author name for my-repo"`, using the provided default if input is empty.
///
/// # Parameters
/// - `prompter`: A mutable reference to a `StringPrompter` implementation.
/// - `label`: A short description of what is being requested (e.g., `"Author name"`).
/// - `repo_name`: The name of the current repository, shown for context.
/// - `default_value`: A fallback if the user presses Enter without typing.
///
/// # Returns
/// - `Ok(String)` containing user input or the default.
/// - `Err(String)` if the input could not be collected.
pub fn ask<P: StringPrompter>(
    prompter: &mut P,
    label: &str,
    repo_name: &str,
    default_value: &str,
) -> Result<String, String> {
    let prompt = format!("{} for {}", label, repo_name);
    prompter.prompt(&prompt, default_value)
}

/// Ask the user to confirm whether to begin rewriting commit history.
///
/// Wraps the `ConfirmPrompter` trait with a specific prompt about rewriting commits.
///
/// # Parameters
/// - `prompter`: A mutable reference to a `ConfirmPrompter` implementation.
///
/// # Returns
/// - `Ok(true)` if the user confirmed.
/// - `Ok(false)` if the user declined.
/// - `Err(String)` if input failed.
pub fn confirm_start<P: ConfirmPrompter>(prompter: &mut P) -> Result<bool, String> {
    let prompt = "Start now? (will auto-mark all picks as edit and amend each stop)";
    prompter.confirm(prompt, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStringPrompter {
        pub response: Result<String, String>,
        pub expected_prompt: String,
        pub expected_default: String,
    }

    impl StringPrompter for MockStringPrompter {
        fn prompt(&mut self, prompt: &str, default: &str) -> Result<String, String> {
            assert_eq!(prompt, self.expected_prompt);
            assert_eq!(default, self.expected_default);
            self.response.clone()
        }
    }

    struct MockConfirmPrompter {
        pub response: Result<bool, String>,
        pub expected_prompt: String,
        pub expected_default: bool,
    }

    impl ConfirmPrompter for MockConfirmPrompter {
        fn confirm(&mut self, prompt: &str, default: bool) -> Result<bool, String> {
            assert_eq!(prompt, self.expected_prompt);
            assert_eq!(default, self.expected_default);
            self.response.clone()
        }
    }

    #[test]
    fn test_ask_returns_user_input() {
        let mut prompter = MockStringPrompter {
            response: Ok("Alice".to_string()),
            expected_prompt: "Author name for my-repo".to_string(),
            expected_default: "Jane Doe".to_string(),
        };
        let result = ask(&mut prompter, "Author name", "my-repo", "Jane Doe");
        assert_eq!(result.unwrap(), "Alice");
    }

    #[test]
    fn test_ask_returns_default_on_empty_input() {
        let mut prompter = MockStringPrompter {
            response: Ok("".to_string()),
            expected_prompt: "Author name for test-repo".to_string(),
            expected_default: "John Doe".to_string(),
        };
        let result = ask(&mut prompter, "Author name", "test-repo", "John Doe");
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_ask_returns_error() {
        let mut prompter = MockStringPrompter {
            response: Err("input failed".to_string()),
            expected_prompt: "Label for repo".to_string(),
            expected_default: "default".to_string(),
        };
        let result = ask(&mut prompter, "Label", "repo", "default");
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_start_true() {
        let mut prompter = MockConfirmPrompter {
            response: Ok(true),
            expected_prompt: "Start now? (will auto-mark all picks as edit and amend each stop)"
                .to_string(),
            expected_default: true,
        };
        let result = confirm_start(&mut prompter);
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_confirm_start_false() {
        let mut prompter = MockConfirmPrompter {
            response: Ok(false),
            expected_prompt: "Start now? (will auto-mark all picks as edit and amend each stop)"
                .to_string(),
            expected_default: true,
        };
        let result = confirm_start(&mut prompter);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_confirm_start_error() {
        let mut prompter = MockConfirmPrompter {
            response: Err("confirm failed".to_string()),
            expected_prompt: "Start now? (will auto-mark all picks as edit and amend each stop)"
                .to_string(),
            expected_default: true,
        };
        let result = confirm_start(&mut prompter);
        assert!(result.is_err());
    }
}
