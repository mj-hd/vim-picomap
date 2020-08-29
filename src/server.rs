use crate::highlighter::*;
use crate::message::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use neovim_lib::neovim_api::{Buffer, Window};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use std::convert::TryFrom;
use std::sync::mpsc;
use std::time::Duration;

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
    modifier: Modifier,
    buf_len: usize,
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
            modifier: Modifier::default(),
            buf_len: 0,
        }
    }
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

        loop {
            if Err(mpsc::TryRecvError::Empty) != done.try_recv() {
                break;
            }

            match recv.try_recv() {
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    eprintln!("neovim has disconeccted");
                    break;
                }
                Ok((event, values)) => {
                    let result = match Message::from(event) {
                        Message::Sync => self.sync(values).context("failed to call sync handler"),
                        Message::Show => self.show(values).context("failed to call show handler"),
                        Message::Resize => {
                            self.resize(values).context("failed to call resize handler")
                        }
                        Message::Close => {
                            self.close(values).context("failed to call close handler")
                        }
                        _ => {
                            eprintln!("unknown message");
                            Ok(())
                        }
                    };
                    if let Err(err) = result {
                        eprintln!("err: {}", err);
                    }
                }
            }

            smol::Timer::new(Duration::from_millis(10)).await;
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

        let cur_buf = self
            .nvim
            .get_current_buf()
            .context("failed to get current buffer")?;

        let buf_len = cur_buf
            .line_count(&mut self.nvim)
            .context("failed to get line count")? as usize;

        self.diags.sync(buf_len, diags);
        self.changes.sync(buf_len, changes);
        self.modifier = self.get_modifier()?;
        self.buf_len = buf_len;

        self.redraw()
    }

    fn show(&mut self, _values: Vec<Value>) -> Result<()> {
        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get current win")?;

        let config = self.get_win_config(&cur_win)?;

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
        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get current win")?;

        let config = self.get_win_config(&cur_win)?;

        let win = match &self.win {
            Some(win) => win,
            None => return Ok(()),
        };

        win.set_config(&mut self.nvim, config)
            .context("failed to set window config")?;

        self.modifier = self.get_modifier()?;

        self.redraw()
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

    fn redraw(&mut self) -> Result<()> {
        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get window")?;

        let win_height = cur_win
            .get_height(&mut self.nvim)
            .context("failed to get window height")? as u64;

        let buffer = format_highlights(
            self.changes.highlight(),
            self.diags.highlight(),
            &self.modifier,
            self.buf_len,
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

    fn get_modifier(&mut self) -> Result<Modifier> {
        let cur_win = self
            .nvim
            .get_current_win()
            .context("failed to get current window")?;

        let win_height = cur_win
            .get_height(&mut self.nvim)
            .context("failed to get window height")? as u64;

        let mode = self
            .nvim
            .eval("mode()")
            .context("failed to eval mode")?
            .as_str()
            .context("invalid mode str")?
            .to_owned();

        let select_start = self
            .nvim
            .eval("getpos('v')")
            .context("failed to eval select start position")?
            .as_array()
            .context("invalid select start position")?[1]
            .as_u64()
            .context("invalid select start value")?;

        let scroll = self
            .nvim
            .eval("line('w0')")
            .context("failed to eval scroll position")?
            .as_u64()
            .context("invalid scroll position")?;

        let cursor = cur_win
            .get_cursor(&mut self.nvim)
            .context("failed to get cursor")?
            .0 as u64;

        let visible_frame = Frame {
            top: scroll - 1,
            bottom: scroll - 1 + win_height,
        };

        let mut modifier = Modifier::new(cursor - 1, visible_frame);

        modifier.select_frame = match &mode[..] {
            "v" | "V" | "CTRL-V" => Some(Frame {
                top: select_start - 1,
                bottom: cursor - 1,
            }),
            _ => None,
        };

        Ok(modifier)
    }

    fn get_win_config(&mut self, cur_win: &Window) -> Result<Vec<(Value, Value)>> {
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
