use crate::highlighter::*;
use bitflags::bitflags;
use std::cmp::{max, min};
use std::fmt;

bitflags! {
    struct Block: u8 {
        const NONE = 0b00;
        const FULL = 0b11;
        const TOP = 0b01;
        const BOTTOM = 0b10;
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Block::FULL => '▌',
                Block::TOP => '▘',
                Block::BOTTOM => '▖',
                _ => ' ',
            }
        )
    }
}

#[derive(Debug)]
struct Line {
    values: Vec<(Block, Highlight)>,
}

impl Line {
    pub fn new(highlights: &[Highlight]) -> Self {
        let len = highlights.len();
        let mut values = Vec::with_capacity(len);

        values.push(if highlights[0] > 0 {
            (Block::FULL, highlights[0])
        } else {
            (Block::NONE, 0)
        });

        for i in 1..len {
            let prev = highlights[i - 1];
            let curr = highlights[i];

            let mut block = Block::NONE;
            let mut highlight = 0;

            if prev > 0 {
                block |= Block::TOP;
                highlight = prev;
            }

            if curr > 0 {
                block |= Block::BOTTOM;
                highlight = curr;
            }

            values.push((block, highlight));
        }

        Line { values }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn scale(&self, height: usize) -> Self {
        let len = self.values.len();
        let scale = len as f64 / height as f64;

        let mut result = Vec::with_capacity(height);

        for i in 0..height {
            let offset = ((i as f64) * scale) as usize;
            let limit = ((i + 1) as f64 * scale) as usize;
            let init = self.values[offset];
            let range = &self.values[offset..limit];

            let block = range.iter().fold(init.0, |acc, val| acc | val.0);
            let highlight = range.iter().fold(init.1, |acc, val| max(acc, val.1));

            result.push((block, highlight));
        }

        let mut prev_block = self.values[0].0;

        for i in 1..height {
            let block = result[i].0;

            if prev_block == block && block == Block::BOTTOM {
                result[i].0 = Block::FULL;
            }

            prev_block = block;
        }

        for i in (height - 1)..0 {
            let block = result[i].0;

            if prev_block == block && block == Block::TOP {
                result[i].0 = Block::FULL;
            }

            prev_block = block;
        }

        Line { values: result }
    }
}

impl std::ops::Index<usize> for Line {
    type Output = (Block, Highlight);

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

#[derive(Clone, Copy)]
pub struct Frame {
    pub top: u64,
    pub bottom: u64,
}

impl Default for Frame {
    fn default() -> Self {
        Frame { top: 0, bottom: 0 }
    }
}

impl Frame {
    fn contains(&self, offset: f64, scale: f64) -> bool {
        let top = min(self.top, self.bottom);
        let bottom = max(self.top, self.bottom);
        top < (offset + scale) as u64 && bottom >= offset as u64
    }
}

#[derive(Clone, Copy)]
pub struct Modifier {
    pub cursor: u64,
    pub visible_frame: Frame,
    pub select_frame: Option<Frame>,
}

impl Default for Modifier {
    fn default() -> Self {
        Modifier {
            cursor: 0,
            visible_frame: Frame::default(),
            select_frame: None,
        }
    }
}

impl Modifier {
    pub fn new(cursor: u64, visible_frame: Frame) -> Self {
        Modifier {
            cursor,
            visible_frame,
            select_frame: None,
        }
    }

    pub fn to_char(&self, i: u64, len: usize, height: u64) -> char {
        let scale = len as f64 / height as f64;
        let offset = (i as f64) * scale;

        if offset as u64 <= self.cursor && (self.cursor as f64) < offset + scale {
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

pub struct Picomap {
    pub changes: Highlights,
    pub diags: Highlights,
    pub modifier: Modifier,
}

impl Default for Picomap {
    fn default() -> Self {
        Picomap {
            changes: Highlights::default(),
            diags: Highlights::default(),
            modifier: Modifier::default(),
        }
    }
}

impl Picomap {
    pub fn new(changes: Highlights, diags: Highlights, modifier: Modifier) -> Self {
        Picomap {
            changes,
            diags,
            modifier,
        }
    }

    pub fn to_strings(&self, len: usize, height: u64) -> Vec<String> {
        let mut result = Vec::with_capacity(height as usize);

        if len == 0 || height == 0 {
            return vec![];
        }

        let change_line = Line::new(&self.changes).scale(height as usize);
        let diag_line = Line::new(&self.diags).scale(height as usize);

        for i in 0..height {
            let change = change_line[i as usize];
            let diag = diag_line[i as usize];

            result.push(format!(
                "{}{}{:>02}{:>02}{}",
                change.0,
                diag.0,
                change.1,
                diag.1,
                self.modifier.to_char(i, len, height),
            ));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picomap_format() {
        let len = 3;
        let height = 3;
        let changes = vec![1, 2, 3];
        let diags = vec![0, 0, 0];
        let modifier = Modifier::default();

        let picomap = Picomap::new(changes, diags, modifier);

        assert_eq!(
            picomap.to_strings(len, height),
            vec!["▌ 0100c", "▌ 0200 ", "▌ 0300 ",]
        );
    }

    #[test]
    fn test_picomap_format_zoom_in() {
        let len = 3;
        let height = 10;
        let changes = vec![1, 2, 3];
        let diags = vec![0, 0, 0];
        let modifier = Modifier::default();

        let picomap = Picomap::new(changes, diags, modifier);

        assert_eq!(
            picomap.to_strings(len, height),
            vec![
                "▌ 0100c",
                "▌ 0100c",
                "▌ 0100c",
                "▌ 0100c",
                "▌ 0200 ",
                "▌ 0200 ",
                "▌ 0200 ",
                "▌ 0300 ",
                "▌ 0300 ",
                "▌ 0300 ",
            ]
        );
    }

    #[test]
    fn test_picomap_format_zoom_out() {
        let len = 10;
        let height = 5;
        let changes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let diags = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let modifier = Modifier::default();

        let picomap = Picomap::new(changes, diags, modifier);

        assert_eq!(
            picomap.to_strings(len, height),
            vec!["▖ 0100c", "▌ 0300 ", "▌ 0500 ", "▌ 0700 ", "▌ 0900 ",]
        );
    }
}
