//! Defines `Span` and `Spanned`. A span is a pair of indices usually denoting
//! a region in the source code.

use std::{fmt, ops};


/// Represents a region in the source text.
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// Start of the span, inclusive
    pub lo: usize,

    /// End of the span, exclusive
    pub hi: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Self {
        Self { lo, hi }
    }

    /// Returns how many bytes this span spans.
    pub fn len(&self) -> usize {
        self.hi - self.lo
    }
}


/// Wraps any value and pairs it with a span.
#[derive(Clone, Copy)]
pub struct Spanned<T> {
    pub data: T,
    pub span: Span,
}

impl<T> ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.fmt(f)?;
        write!(f, " @ {}..{}", self.span.lo, self.span.hi)?;
        Ok(())
    }
}
