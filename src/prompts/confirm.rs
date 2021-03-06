use std::io;

use crate::theme::{SimpleTheme, TermThemeRenderer, Theme};

use console::Term;

/// Renders a confirm prompt.
///
/// ## Example usage
///
/// ```rust,no_run
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use dialoguer::Confirm;
///
/// if Confirm::new().with_prompt("Do you want to continue?").interact()? {
///     println!("Looks like you want to continue");
/// } else {
///     println!("nevermind then :(");
/// }
/// # Ok(()) } fn main() { test().unwrap(); }
/// ```
pub struct Confirm<'a> {
    prompt: String,
    default: bool,
    show_default: bool,
    disable_default: bool,
    wait_for_newline: bool,
    theme: &'a dyn Theme,
}

impl<'a> Default for Confirm<'a> {
    fn default() -> Confirm<'a> {
        Confirm::new()
    }
}

impl<'a> Confirm<'a> {
    /// Creates a confirm prompt.
    pub fn new() -> Confirm<'static> {
        Confirm::with_theme(&SimpleTheme)
    }

    /// Creates a confirm prompt with a specific theme.
    ///
    /// ## Examples
    /// ```rust,no_run
    /// use dialoguer::{
    ///     Confirm,
    ///     theme::ColorfulTheme
    /// };
    ///
    /// # fn main() -> std::io::Result<()> {
    /// let proceed = Confirm::with_theme(&ColorfulTheme::default())
    ///     .with_prompt("Do you wish to continue?")
    ///     .interact()?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn with_theme(theme: &'a dyn Theme) -> Confirm<'a> {
        Confirm {
            prompt: "".into(),
            default: true,
            show_default: true,
            disable_default: false,
            wait_for_newline: false,
            theme,
        }
    }

    /// Sets the confirm prompt.
    pub fn with_prompt<S: Into<String>>(&mut self, prompt: S) -> &mut Confirm<'a> {
        self.prompt = prompt.into();
        self
    }

    #[deprecated(note = "Use with_prompt() instead", since = "0.6.0")]
    #[inline]
    pub fn with_text(&mut self, text: &str) -> &mut Confirm<'a> {
        self.with_prompt(text)
    }

    /// Sets when to react to user input.
    ///
    /// When `false` (default), we check on each user keystroke immediately as
    /// it is typed. Valid inputs can be one of 'y', 'n', or a newline to accept
    /// the default.
    ///
    /// When `true`, the user must type their choice and hit the Enter key before
    /// proceeding. Valid inputs can be "yes", "no", "y", "n", or an empty string
    /// to accept the default.
    pub fn wait_for_newline(&mut self, wait: bool) -> &mut Confirm<'a> {
        self.wait_for_newline = wait;
        self
    }
    /// Overrides the default output if user pushes enter key without inputing any character.
    /// Character corresponding to the default choice (e.g `Y` if default is `true`) will be uppercased in the displayed prompt.
    ///
    /// The default output is true.
    pub fn default(&mut self, val: bool) -> &mut Confirm<'a> {
        self.default = val;
        self
    }

    /// Disables or enables display of options user can choose from.
    ///
    /// The default is to append `[y/n]` to the prompt to tell the
    /// user which keys to press. This also renders the default choice
    /// in uppercase. The default is selected on enter.
    pub fn show_default(&mut self, val: bool) -> &mut Confirm<'a> {
        self.show_default = val;
        self
    }

    /// Select whether the user must explicitly confirm their selection.
    ///
    /// When `false` (default), the user can input a newline to select the default.
    ///
    /// When `true`, the user must input a letter - a newline will do nothing.
    pub fn disable_default(&mut self, val: bool) -> &mut Confirm<'a> {
        self.disable_default = val;
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// If the user confirms the result is `true`, `false` if declines or default (configured in [default](#method.default)) if pushes enter.
    /// Otherwise function discards input waiting for valid one.
    ///
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<bool> {
        self.interact_on(&Term::stderr())
    }

    /// Like [interact](#method.interact) but allows a specific terminal to be set.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use dialoguer::Confirm;
    /// use console::Term;
    ///
    /// # fn main() -> std::io::Result<()> {
    /// let proceed = Confirm::new()
    ///     .with_prompt("Do you wish to continue?")
    ///     .interact_on(&Term::stderr())?;
    /// #   Ok(())
    /// # }
    /// ```
    pub fn interact_on(&self, term: &Term) -> io::Result<bool> {
        let mut render = TermThemeRenderer::new(term, self.theme);

        let default = if self.show_default {
            Some(self.default)
        } else {
            None
        };
        render.confirm_prompt(&self.prompt, default)?;

        term.hide_cursor()?;
        term.flush()?;

        if self.wait_for_newline {
            // Waits for user input and for the user to hit the Enter key
            // before validation.
            let mut input_buf = String::new();
            loop {
                io::stdin().read_line(&mut input_buf)?;
                let rv = match &*input_buf.trim_end().to_lowercase() {
                    "y" | "yes" => true,
                    "n" | "no" => false,
                    "" if !self.disable_default => self.default,
                    _ => {
                        // On invalid input re-render the user prompt.
                        render.confirm_prompt(&self.prompt, default)?;
                        input_buf.clear();
                        continue;
                    }
                };

                term.show_cursor()?;
                term.flush()?;

                return Ok(rv);
            }
        } else {
            // Default behavior: matches continuously on every keystroke,
            // and does not wait for user to hit the Enter key.
            loop {
                let input = term.read_char()?;
                let rv = match input {
                    'y' | 'Y' => true,
                    'n' | 'N' => false,
                    '\n' | '\r' if !self.disable_default => self.default,
                    _ => {
                        continue;
                    }
                };

                term.clear_line()?;
                render.confirm_prompt_selection(&self.prompt, rv)?;
                term.show_cursor()?;
                term.flush()?;

                return Ok(rv);
            }
        }
    }
}
