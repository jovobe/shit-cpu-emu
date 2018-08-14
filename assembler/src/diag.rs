//! Types and functions to print error messages (diagnostics).

use crate::span::Span;


/// An error message paired with an optional span and possibly a number of
/// additional notes.
pub struct Diag {
    msg: String,
    span: Option<Span>,
    notes: Vec<String>,
}

impl Diag {
    /// Creates a new error diag with the given message.
    #[allow(dead_code)] // TODO: remove once used
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            span: None,
            notes: vec![],
        }
    }

    /// Creates a new error diag with the given message and span.
    pub fn span_error(span: Span, msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            span: Some(span),
            notes: vec![],
        }
    }

    /// Adds the given message as note to this span.
    pub fn add_note(mut self, msg: impl Into<String>) -> Self {
        self.notes.push(msg.into());
        self
    }

    /// Print the diagnostic on the terminal.
    ///
    /// - `line` needs to be the line the span in this diagnostic points to.
    /// - `line_number` is the 0-based number of the line the error originated
    /// in.
    pub fn emit(self, line: &str, line_number: usize) {
        use term_painter::{ToStyle, Color};
        use std::iter;

        // Print error message
        println!(
            "{}: {}",
            Color::Red.bold().paint("error"),
            Color::White.bold().paint(self.msg),
        );


        // Format line number (in our program it's 0-based, but humans like
        // it 1-based)
        let num = (line_number + 1).to_string();
        let num_placeholder = iter::repeat(' ').take(num.len()).collect::<String>();

        // If a span was provided, underline the span in source code
        if let Some(span) = self.span {
            let before_underline = iter::repeat(' ').take(span.lo).collect::<String>();
            let underline = iter::repeat('^').take(span.len()).collect::<String>();

            println!(
                "{} {} {}",
                Color::Blue.bold().paint(num),
                Color::Blue.bold().paint("|"),
                line,
            );
            println!(
                "{} {} {}{}",
                num_placeholder,
                Color::Blue.bold().paint("|"),
                before_underline,
                Color::Red.bold().paint(underline),
            );
        }

        // Print all notes
        for note in self.notes {
            println!(
                "{} {} {}",
                num_placeholder,
                Color::White.bold().paint("= note:"),
                Color::White.paint(note),
            );
        }

        println!("");
    }
}
