use std::cmp::{max, min};
use std::fmt;

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

struct Block {
    values: [Highlight; 2],
}

impl Block {
    fn new(first: Highlight, last: Highlight) -> Self {
        Block {
            values: [first, last],
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if self.values[0] > 0 && self.values[1] > 0 {
                '▌'
            } else if self.values[0] > 0 {
                '▘'
            } else if self.values[1] > 0 {
                '▖'
            } else {
                ' '
            }
        )
    }
}

pub struct Frame {
    pub top: u64,
    pub bottom: u64,
}

impl Frame {
    fn contains(&self, offset: f64, scale: f64) -> bool {
        let top = min(self.top, self.bottom);
        let bottom = max(self.top, self.bottom);
        top < (offset + scale) as u64 && bottom >= offset as u64
    }
}

struct Modifier {
    pub cursor: u64,
    pub visible_frame: Frame,
    pub select_frame: Option<Frame>,
}

impl Modifier {
    pub fn to_char(&self, offset: f64, scale: f64) -> char {
        if offset as u64 <= self.cursor && self.cursor < (offset + scale) as u64 {
            return 'c';
        }

        if let Some(frame) = &self.select_frame {
            if frame.contains(offset, scale) {
                return 's';
            }
        }

        if self.visible_frame.contains(offset, scale) {
            return 'v';
        }

        ' '
    }
}

pub fn format_highlights(
    changes: Highlights,
    diags: Highlights,
    visible_frame: Frame,
    select_frame: Option<Frame>,
    cursor: u64,
    len: usize,
    height: u64,
) -> Vec<String> {
    let mut result = Vec::with_capacity(height as usize);

    if len == 0 || height == 0 {
        return vec![];
    }

    let modifier = Modifier {
        cursor,
        visible_frame,
        select_frame,
    };

    let scale = len as f64 / height as f64;

    let mut offset: f64 = 0.0;
    while offset < len as f64 {
        let mut first = [0, 0];
        let mut last = [0, 0];
        for i in (offset as u64)..((offset + scale) as u64) {
            if i >= len as u64 {
                break;
            }

            let block = if (i as f64) < (offset + scale / 2.0) {
                &mut first
            } else {
                &mut last
            };

            if changes[i as usize] > 0 {
                block[0] = changes[i as usize];
            }

            if diags[i as usize] > 0 {
                block[1] = diags[i as usize];
            }
        }

        result.push(format!(
            "{}{}{:>02}{:>02}{}",
            Block::new(first[0], last[0]),
            Block::new(first[1], last[1]),
            max(first[0], last[0]),
            max(first[1], last[1]),
            modifier.to_char(offset, scale),
        ));

        offset += scale;
    }

    result
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
