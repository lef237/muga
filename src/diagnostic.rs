use std::fmt;

use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub related: Vec<RelatedNote>,
    pub suggestions: Vec<DiagnosticSuggestion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedNote {
    pub message: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticSuggestion {
    pub message: String,
    pub span: Option<Span>,
    pub replacement: Option<String>,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            related: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_related(mut self, message: impl Into<String>, span: Span) -> Self {
        self.related.push(RelatedNote {
            message: message.into(),
            span,
        });
        self
    }

    pub fn with_suggestion(mut self, message: impl Into<String>) -> Self {
        self.suggestions.push(DiagnosticSuggestion {
            message: message.into(),
            span: None,
            replacement: None,
        });
        self
    }

    pub fn with_replacement(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.suggestions.push(DiagnosticSuggestion {
            message: message.into(),
            span: Some(span),
            replacement: Some(replacement.into()),
        });
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} {}",
            self.span.start.line, self.span.start.column, self.code, self.message
        )?;
        for note in &self.related {
            write!(
                f,
                "\n  note: {}:{}: {}",
                note.span.start.line, note.span.start.column, note.message
            )?;
        }
        for suggestion in &self.suggestions {
            match (suggestion.span, suggestion.replacement.as_ref()) {
                (Some(span), Some(replacement)) => write!(
                    f,
                    "\n  help: {}:{}: {}; replace with `{}`",
                    span.start.line, span.start.column, suggestion.message, replacement
                )?,
                (Some(span), None) => write!(
                    f,
                    "\n  help: {}:{}: {}",
                    span.start.line, span.start.column, suggestion.message
                )?,
                (None, Some(replacement)) => {
                    write!(f, "\n  help: {}; use `{}`", suggestion.message, replacement)?
                }
                (None, None) => write!(f, "\n  help: {}", suggestion.message)?,
            }
        }
        Ok(())
    }
}
