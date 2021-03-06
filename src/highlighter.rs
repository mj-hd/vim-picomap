const LINE_CAPACITY: usize = 500;

pub type Highlight = u64;
pub type Highlights = Vec<Highlight>;

pub trait Highlighter {
    fn highlight(&self) -> Highlights;
}

#[derive(Debug)]
pub enum DiagnosticLevel {
    None,
    Warning,
    Danger,
}

impl Default for DiagnosticLevel {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default)]
pub struct Diagnostic {
    pub i: usize,
    pub text: String,
    pub level: DiagnosticLevel,
}

#[derive(Debug)]
pub struct DiagnosticsHighlighter {
    values: Vec<DiagnosticLevel>,
}

impl Default for DiagnosticsHighlighter {
    fn default() -> Self {
        Self {
            values: Vec::with_capacity(LINE_CAPACITY),
        }
    }
}

impl DiagnosticsHighlighter {
    pub fn sync(&mut self, len: usize, diags: Vec<Diagnostic>) {
        self.values.clear();
        self.values.resize_with(len, Default::default);

        for diag in diags {
            if diag.i >= len {
                // TODO report an error
                continue;
            }
            self.values[diag.i] = diag.level;
        }
    }
}

impl Highlighter for DiagnosticsHighlighter {
    fn highlight(&self) -> Highlights {
        self.values
            .iter()
            .map(|val| match val {
                DiagnosticLevel::Warning => 1,
                DiagnosticLevel::Danger => 2,
                _ => 0,
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Change {
    pub i: usize,
    pub len: usize,
}

#[derive(Debug)]
pub struct ChangeHighlighter {
    values: Vec<bool>,
}

impl Default for ChangeHighlighter {
    fn default() -> Self {
        Self {
            values: Vec::with_capacity(LINE_CAPACITY),
        }
    }
}

impl ChangeHighlighter {
    pub fn sync(&mut self, len: usize, changes: Vec<Change>) {
        self.values.clear();
        self.values.resize_with(len, Default::default);

        for change in changes {
            for i in change.i..(change.i + change.len) {
                if i >= len {
                    continue;
                }
                self.values[i] = true;
            }
        }
    }
}

impl Highlighter for ChangeHighlighter {
    fn highlight(&self) -> Highlights {
        self.values
            .iter()
            .map(|val| if *val { 1 } else { 0 })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_highlighter_highlight() {
        let mut highlighter = DiagnosticsHighlighter::default();

        highlighter.sync(
            3,
            vec![
                Diagnostic {
                    i: 1,
                    text: "foo".to_string(),
                    level: DiagnosticLevel::Danger,
                },
                Diagnostic {
                    i: 2,
                    text: "bar".to_string(),
                    level: DiagnosticLevel::Warning,
                },
                Diagnostic {
                    i: 5,
                    text: "hoge".to_string(),
                    level: DiagnosticLevel::Warning,
                },
            ],
        );

        assert_eq!(highlighter.highlight(), vec![0, 2, 1]);
    }

    #[test]
    fn test_change_highlighter_highlight() {
        let mut highlighter = ChangeHighlighter::default();

        highlighter.sync(3, vec![Change { i: 1, len: 2 }]);

        assert_eq!(highlighter.highlight(), vec![0, 1, 1]);
    }
}
