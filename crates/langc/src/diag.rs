use langue_rt::Span;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Severity {
    Error,
    Warning,
}

/// One diagnostic against one file. `file` is the path as the loader saw
/// it; `span` is a byte range into that file's text.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub file: String,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(file: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            file: file.into(),
            span,
            message: message.into(),
        }
    }

    pub fn warning(file: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            file: file.into(),
            span,
            message: message.into(),
        }
    }

    /// Render as `file:line:col: severity: message` given the file text.
    pub fn render(&self, text: &str) -> String {
        let (line, col) = line_col(text, self.span.start);
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        format!("{}:{}:{}: {}: {}", self.file, line, col, sev, self.message)
    }
}

/// 1-based line and column of a byte offset.
pub fn line_col(text: &str, offset: u32) -> (u32, u32) {
    let offset = (offset as usize).min(text.len());
    let mut line = 1;
    let mut line_start = 0;
    for (i, b) in text.bytes().enumerate().take(offset) {
        if b == b'\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    (line, (offset - line_start) as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_counts_newlines() {
        let text = "ab\ncd\nef";
        assert_eq!(line_col(text, 0), (1, 1));
        assert_eq!(line_col(text, 4), (2, 2));
        assert_eq!(line_col(text, 6), (3, 1));
    }

    #[test]
    fn render_shape() {
        let d = Diagnostic::error("x.langue", Span::new(3, 5), "boom");
        assert_eq!(d.render("ab\ncd"), "x.langue:2:1: error: boom");
    }
}
