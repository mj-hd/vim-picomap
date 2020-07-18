use crate::highlighter::*;
use crate::message::*;
use anyhow::{Context, Result};
use neovim_lib::{Neovim, Session, Value};
use std::cmp::max;
use std::convert::TryFrom;

pub struct Server {
    nvim: Neovim,
    diags: DiagnosticsHighlighter,
    changes: ChangeHighlighter,
}

impl Default for Server {
    fn default() -> Self {
        let session = Session::new_parent();

        Self {
            nvim: Neovim::new(session.expect("session not found")),
            diags: DiagnosticsHighlighter::default(),
            changes: ChangeHighlighter::default(),
        }
    }
}

struct VisibleFrame {
    cursor: u64,
    top: u64,
    bottom: u64,
}

fn to_diagnostic(loc: &Location) -> Diagnostic {
    Diagnostic {
        i: loc.lnum as usize,
        text: loc.text.to_string(),
        level: match loc.typ {
            LocationType::Warning => DiagnosticLevel::Warning,
            LocationType::Error => DiagnosticLevel::Danger,
            _ => DiagnosticLevel::None,
        },
    }
}

fn to_change(hunk: &Hunk) -> Change {
    Change {
        i: hunk.lnum as usize,
        len: hunk.len,
    }
}

fn to_block_char(first: Highlight, last: Highlight) -> char {
    if first > 0 && last > 0 {
        '▌'
    } else if first > 0 {
        '▘'
    } else if last > 0 {
        '▖'
    } else {
        ' '
    }
}

fn format(
    changes: Highlights,
    diags: Highlights,
    frame: VisibleFrame,
    len: usize,
    height: u64,
) -> Vec<String> {
    let mut result = Vec::with_capacity(height as usize);

    if len == 0 || height == 0 {
        return vec![];
    }

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
            to_block_char(first[0], last[0]),
            to_block_char(first[1], last[1]),
            max(first[0], last[0]),
            max(first[1], last[1]),
            if offset as u64 <= frame.cursor && frame.cursor < (offset + scale) as u64 {
                'c'
            } else if frame.top < (offset + scale) as u64 && frame.bottom >= offset as u64 {
                'v'
            } else {
                ' '
            },
        ));

        offset += scale;
    }

    result
}

impl Server {
    pub fn start(&mut self) -> Result<()> {
        let recv = self.nvim.session.start_event_loop_channel();

        for (event, values) in recv {
            match Message::from(event) {
                Message::Sync => {
                    let payload =
                        SyncPayload::try_from(values).with_context(|| "invalid payload")?;

                    let diags = payload.locations.iter().map(to_diagnostic).collect();
                    let changes = payload.hunks.iter().map(to_change).collect();

                    let len = payload.lines.len();

                    self.diags.sync(len, diags);
                    self.changes.sync(len, changes);

                    let frame = VisibleFrame {
                        cursor: payload.pos.y,
                        top: payload.scroll,
                        bottom: payload.scroll + payload.height,
                    };

                    let buffer = format(
                        self.changes.highlight(),
                        self.diags.highlight(),
                        frame,
                        len,
                        payload.height,
                    );

                    self.nvim
                        .session
                        .call(
                            "nvim_buf_set_lines",
                            vec![
                                Value::from(payload.bufnr),
                                Value::from(0),
                                Value::from(-1),
                                Value::from(false),
                                Value::from(
                                    buffer.into_iter().map(Value::from).collect::<Vec<_>>(),
                                ),
                            ],
                        )
                        .expect("failed to update");
                }
                _ => {
                    eprintln!("unknown message");
                }
            }
        }

        Ok(())
    }

    pub fn stop(&self) {}
}
