//! Everything related to parsing the raw text.
//!
//! Since the input grammar is rather easy, functions in this module are not
//! very clever about lexing and parsing. It's just not worth it to implement
//! a proper LR-parser or something like that.

use crate::{
    diag::Diag,
    instr::Instruction,
    span::{Span, Spanned},
};


/// The output of parsing: the representation of a program.
#[derive(Debug, Clone)]
pub struct Program {
    pub lines: Vec<Spanned<Line>>,
}

/// A single line of the program.
#[derive(Debug, Clone)]
pub enum Line {
    /// For example: `.foo:`
    Label(String),

    /// For example: `.byte`
    Directive(Directive),

    /// For example `sti $27 [$0]`
    Instruction(Instruction),
}

/// A directive a command to the assembler that gets special treatment.
#[derive(Debug, Clone)]
pub enum Directive {
    /// Tell the assembler to put this exact byte in this position of the
    /// assembled binary.
    Byte(u8),
}

/// Parse a string into a program.
///
/// If any errors occur, the errors are printed an `Err(())` is returned. Empty
/// lines (including comment only lines) are not represented in the returned
/// program.
pub fn parse(input: &str) -> Result<Program, ()> {
    /// Get the span of the string `line` in a larger buffer `input`
    fn line_span(input: &str, line: &str) -> Span {
        let start = line.as_ptr() as usize - input.as_ptr() as usize;
        let end = start + line.len();
        Span::new(start, end)
    }

    // We remember if we have encountered an error.
    let mut error = false;

    let lines = input
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            tokenize(line)
                .and_then(parse_line)
                .unwrap_or_else(|e| {
                    // Print errors and convert them into `None`.
                    e.emit(line, line_number);
                    error = true;
                    None
                })
                .map(|x| {
                    // Add correct span to the line
                    Spanned {
                        span: line_span(input, line),
                        data: x,
                    }
                })
        })
        .collect();

    if error {
        Err(())
    } else {
        Ok(Program { lines })
    }
}

/// Convert a line into a list of tokens.
///
/// If the line is illformed, the first error is returned as `Err()`.
fn tokenize(line: &str) -> Result<Vec<Spanned<Token>>, Diag> {
    let mut chars = line.char_indices().peekable();
    let mut tokens = Vec::new();

    loop {
        // If we reached the end of the line, we can stop.
        let (start, c) = match chars.next() {
            Some((i, c)) => (i, c),
            None => break,
        };

        let token = match c {
            '.' => Token::Dot,
            ':' => Token::Colon,
            '[' => Token::BracketOpen,
            ']' => Token::BracketClose,

            // Literals
            '$' => {
                // Find the end of the literal
                let mut end = start + c.len_utf8();
                while chars.peek().map(|(_, c)| c.is_digit(16)).unwrap_or(false) {
                    let (i, c) = chars.next().unwrap();
                    end = i + c.len_utf8();
                }

                // Try to parse
                match u8::from_str_radix(&line[start + 1..end], 16) {
                    Ok(v) => Token::Literal(v),
                    Err(_) => {
                        // We know all digits are valid, so the problem is that
                        // the literal is too big for `u8`.
                        let msg = "this literal's value overflows `u8`";
                        let diag = Diag::span_error(Span::new(start, end), msg)
                            .add_note("only values between 0 and 255 (`$FF`) are allowed")
                            .add_note("numbers are specified in hexadecimal");

                        return Err(diag);
                    }
                }
            }

            // Idents
            c if is_ident_start(c) => {
                // Find the end of the ident
                let mut end = start + c.len_utf8();
                while chars.peek().map(|(_, c)| is_ident_char(*c)).unwrap_or(false) {
                    let (i, c) = chars.next().unwrap();
                    end = i + c.len_utf8();
                }

                Token::Ident(&line[start..end])
            }

            // Ignore whitespace
            s if s.is_whitespace() => continue,

            // A comment ends with the line break, so we can stop here
            ';' => break,

            // Everything else is an illegal character to start a token
            c => {
                let span = Span::new(start, start + c.len_utf8());
                return Err(Diag::span_error(span, "invalid token start character"));
            }
        };

        // Combine the token with a span and push it to our token list.
        let end = chars.peek().map(|(i, _)| *i).unwrap_or(line.len());
        tokens.push(Spanned {
            data: token,
            span: Span::new(start, end),
        });
    }

    Ok(tokens)
}

/// Make sure the token at `$idx` is `$expected`. If there is no token or it's
/// another token, an error is returned.
macro_rules! expect_token {
    ($tokens:ident[$idx:expr]; $expected_str:expr; $expected:pat => $body:expr) => {
        match $tokens.get($idx) {
            Some(Spanned { data: $expected, .. }) => $body,
            Some(Spanned { data: invalid, span }) => {
                let msg = format!(
                    concat!("unexpected '{:?}' token, expected ", $expected_str),
                    invalid,
                );
                return Err(Diag::span_error(*span, msg));
            }
            None => {
                let msg = concat!("unexpected end of line, expected ", $expected_str);
                let span = Span::new($tokens[$idx - 1].span.hi, $tokens[$idx - 1].span.hi + 1);
                return Err(Diag::span_error(span, msg));
            }
        }
    }
}

/// Make sure there is no token at `$idx` (which means we reached the end of
/// the line). If there is a token there, an error is returned.
macro_rules! expect_eol {
    ($tokens:ident[$idx:expr], $additional_str:expr) => {
        if let Some(tok) = $tokens.get($idx) {
            let msg = format!(
                concat!("unexpected token '{:?}', expected end of line", $additional_str),
                tok.data,
            );
            return Err(Diag::span_error(tok.span, msg));
        }
    }
}


/// Parses a single line from tokens into a `Line`. Empty lines are returned as
/// `Ok(None)`.
///
/// If the line is illformed, the first error is returned as `Err()`.
fn parse_line(tokens: Vec<Spanned<Token>>) -> Result<Option<Line>, Diag> {
    if tokens.is_empty() {
        return Ok(None);
    }

    // Look at the first token and decide what to do next.
    let line = match &*tokens[0] {
        // A label or a directive.
        Token::Dot => {
            // The next token has to be an ident in any case.
            let name = expect_token!(tokens[1]; "ident"; Token::Ident(s) => *s);

            // Check if the next token is a colon (':') or not. If yes, this is
            // a label, if not, it's a directive.
            let colon_next = tokens.get(2).map(|s| s.data == Token::Colon).unwrap_or(false);
            if colon_next {
                // Make sure we reached the end of the line
                expect_eol!(tokens[3], " after label");
                Line::Label(name.to_owned())
            } else {
                Line::Directive(parse_directive(name, &tokens)?)
            }
        }

        // An instruction
        Token::Ident(name) => Line::Instruction(parse_instruction(name, &tokens)?),

        // Everything else is illegal at the beginning of the line.
        token => {
            let msg = format!("unexpected '{:?}' token at start of line", token);
            let diag = Diag::span_error(tokens[0].span, msg)
                .add_note("expected ident or '.'");

            return Err(diag);
        }
    };

    Ok(Some(line))
}

/// Parses a single instruction from the given tokens. The first token needs to
/// be an ident! The first error encountered is returned.
fn parse_instruction(_name: &str, _tokens: &[Spanned<Token>]) -> Result<Instruction, Diag> {
    // TODO
    Ok(Instruction::Nop)
}

/// Parses the given tokens as directive. The first token needs to be '.' and
/// the second one needs to be an ident! The first error encountered is
/// returned.
fn parse_directive(name: &str, tokens: &[Spanned<Token>]) -> Result<Directive, Diag> {
    match name {
        "byte" => {
            // We need a literal next and don't allow any tokens after that
            let v = expect_token!(tokens[2]; "literal"; Token::Literal(v) => *v);
            expect_eol!(tokens[3], "");

            Ok(Directive::Byte(v))
        }
        invalid => {
            let msg = format!("invalid directive name '{}'", invalid);
            Err(Diag::span_error(tokens[1].span, msg))
        }
    }
}

/// Returns `true` if the character is a valid identifier start.
fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

/// Returns `true` if the character is a valid identifier character.
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// A token in the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'src> {
    /// `.`
    Dot,

    /// `:`
    Colon,

    /// `[`
    BracketOpen,

    /// `]`
    BracketClose,

    /// An identifier: a string consisting of only alphanumeric characters or
    /// `_` where the first character is `_` or an alphabetic one.
    Ident(&'src str),

    /// A number literal already converted to its value.
    Literal(u8),
}
