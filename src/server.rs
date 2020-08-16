use crate::channel_stream::*;
use crate::highlighter::*;
use crate::message::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use neovim_lib::neovim_api::{Buffer, Window};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use smol::stream::*;
use std::cmp::max;
use std::convert::TryFrom;
use std::sync::mpsc;

#[async_trait]
pub trait ServerTrait {
    async fn start(&mut self, done: mpsc::Receiver<()>) -> Result<()>;
}

pub struct Server {
    nvim: Neovim,
    buf: Option<Buffer>,
    win: Option<Window>,
    diags: DiagnosticsHighlighter,
    changes: ChangeHighlighter,
}

impl Default for Server {
    fn default() -> Self {
        let session = Session::new_parent();

        Self {
            nvim: Neovim::new(session.expect("session not found")),
            buf: None,
            win: None,
            diags: DiagnosticsHighlighter::default(),
            changes: ChangeHighlighter::default(),
        }
    }
}

// TODO move somewhere
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
        i: hunk.lnum as usize - 1,
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

#[async_trait]
impl ServerTrait for Server {
    async fn start(&mut self, done: mpsc::Receiver<()>) -> Result<()> {
        let recv = self.nvim.session.start_event_loop_channel();

        eprintln!("start event loop");

        self.buf = Some(
            self.nvim
                .create_buf(false, true)
                .context("failed to create buf")?,
        );

        let mut events = ChannelStream::from(recv).with_done(done);

        while let Some((event, values)) = events.next().await {
            if let Err(err) = match Message::from(event) {
                Message::Sync => {
                    self.sync(values).context("failed to call sync handler")?;
                    Ok(())
                }
                Message::Show => {
                    self.show(values).context("failed to call show handler")?;
                    Ok(())
                }
                Message::Resize => {
                    self.resize(values)
                        .context("failed to call resize handler")?;
                    Ok(())
                }
                Message::Close => {
                    self.close(values).context("failed to call close handler")?;
                    Ok(())
                }
                _ => {
                    eprintln!("unknown message");
                    Ok(())
                }
            } {
                return err;
            }
        }

        eprintln!("exit event loop");

        Ok(())
    }
}

impl Server {
    fn sync(&mut self, values: Vec<Value>) -> Result<()> {
        let payload = SyncPayload::try_from(values).context("invalid payload")?;

        let diags = payload.locations.iter().map(to_diagnostic).collect();
        let changes = payload.hunks.iter().map(to_change).collect();

        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get window")?;
        let cur_buf = self
            .nvim
            .get_current_buf()
            .context("failed to get buffer")?;

        let buf_len = cur_buf
            .line_count(&mut self.nvim)
            .context("failed to get line count")? as usize;

        let win_height = cur_win
            .get_height(&mut self.nvim)
            .context("failed to get window height")? as u64;

        let cursor = cur_win
            .get_cursor(&mut self.nvim)
            .context("failed to get cursor")?;

        let scroll = self
            .nvim
            .eval("line('w0')")
            .context("failed to eval scroll position")?
            .as_u64()
            .context("invalid scroll position")?;

        self.diags.sync(buf_len, diags);
        self.changes.sync(buf_len, changes);

        let frame = VisibleFrame {
            cursor: cursor.0 as u64,
            top: scroll,
            bottom: scroll + win_height,
        };

        let buffer = format(
            self.changes.highlight(),
            self.diags.highlight(),
            frame,
            buf_len,
            win_height,
        );

        let buf = match &self.buf {
            Some(buf) => buf,
            None => return Ok(()),
        };

        buf.set_lines(&mut self.nvim, 0, -1, false, buffer)
            .context("failed to set buf lines")?;

        Ok(())
    }

    fn show(&mut self, _values: Vec<Value>) -> Result<()> {
        let config = self.win_config()?;

        let buf = match &self.buf {
            Some(buf) => buf,
            None => return Ok(()),
        };

        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get current window")?;

        self.win = Some(
            self.nvim
                .open_win(buf, true, config)
                .context("failed to create win")?,
        );

        let winblend = self
            .nvim
            .get_var("picomap_winblend")
            .context("failed to get global winblend option")?;

        let win = match &self.win {
            Some(win) => win,
            None => return Ok(()),
        };

        win.set_option(&mut self.nvim, "winhl", Value::from("Normal:Picomap"))
            .context("failed to set winhl option to win")?;
        win.set_option(&mut self.nvim, "winblend", winblend)
            .context("failed to set winblend option to win")?;

        buf.set_option(&mut self.nvim, "filetype", Value::from("picomap"))
            .context("failed to set filetype option")?;

        self.nvim
            .set_current_win(&cur_win)
            .context("failed to set current win")?;

        Ok(())
    }

    fn resize(&mut self, _values: Vec<Value>) -> Result<()> {
        let config = self.win_config()?;

        let win = match &self.win {
            Some(win) => win,
            None => return Ok(()),
        };

        win.set_config(&mut self.nvim, config)
            .context("failed to set window config")?;

        Ok(())
    }

    fn close(&mut self, _values: Vec<Value>) -> Result<()> {
        let win = match &self.win {
            Some(win) => win,
            None => return Ok(()),
        };

        win.close(&mut self.nvim, true)
            .context("failed to close picomap")?;

        self.win = None;

        Ok(())
    }

    fn win_config(&mut self) -> Result<Vec<(Value, Value)>> {
        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get current win")?;

        let cur_win_height = cur_win
            .get_height(&mut self.nvim)
            .context("failed to get current win height")?;
        let cur_win_width = cur_win
            .get_width(&mut self.nvim)
            .context("failed to get current win width")?;
        let cur_win_pos = cur_win
            .get_position(&mut self.nvim)
            .context("failed to get current win pos")?;

        Ok(vec![
            (Value::from("relative"), Value::from("editor")),
            (Value::from("anchor"), Value::from("NE")),
            (Value::from("width"), Value::from(2)),
            (Value::from("focusable"), Value::from(false)),
            (Value::from("style"), Value::from("minimal")),
            (Value::from("height"), Value::from(cur_win_height)),
            (
                Value::from("col"),
                Value::from(cur_win_pos.1 + cur_win_width),
            ),
            (Value::from("row"), Value::from(cur_win_pos.0)),
        ])
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        eprintln!("server dropped");

        match &self.win {
            Some(win) => {
                win.close(&mut self.nvim, true)
                    .expect("failed to close win");
            }
            None => (),
        };
    }
}
