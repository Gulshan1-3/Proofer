use crate::syntax::span::Span;
use crate::syntax::source::SourceMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Info => write!(f, "info"),
            DiagnosticLevel::Hint => write!(f, "hint"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubLabel {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub primary_span: Option<Span>,
    pub sub_labels: Vec<SubLabel>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            primary_span: Some(span),
            sub_labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            primary_span: Some(span),
            sub_labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_sublabel(mut self, span: Span, message: impl Into<String>) -> Self {
        self.sub_labels.push(SubLabel {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn render(&self, sources: &SourceMap) -> String {
        let mut out = String::new();
        if let Some(span) = self.primary_span {
            if let Some(file) = sources.get(span.file) {
                let (line, col) = file.line_col(span.start);
                out.push_str(&format!("{}:{}:{}: {}: {}\n", file.name, line, col, self.level, self.message));
                if let Some(line_text) = file.line_text(line) {
                    out.push_str(&format!(" {:4} | {}\n", line, line_text));
                    out.push_str("      | ");
                    for _ in 1..col {
                        out.push(' ');
                    }
                    let underline_len = if span.end > span.start {
                        let span_len = (span.end - span.start) as usize;
                        let line_rem = line_text.chars().count().saturating_sub(col - 1);
                        span_len.min(line_rem.max(1))
                    } else {
                        1
                    };
                    for _ in 0..underline_len {
                        out.push('^');
                    }
                    out.push('\n');
                }
            } else {
                out.push_str(&format!("{}: {}\n", self.level, self.message));
            }
        } else {
            out.push_str(&format!("{}: {}\n", self.level, self.message));
        }

        for sub in &self.sub_labels {
            if let Some(file) = sources.get(sub.span.file) {
                let (line, col) = file.line_col(sub.span.start);
                out.push_str(&format!("  --> {}:{}:{}: note: {}\n", file.name, line, col, sub.message));
            }
        }

        for note in &self.notes {
            out.push_str(&format!("  = note: {}\n", note));
        }

        out
    }
}
