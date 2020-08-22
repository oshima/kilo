extern crate unicode_width;

use std::cmp;
use std::io::{self, Write};
use unicode_width::UnicodeWidthChar;

use crate::uint_vec::UintVec;

const TAB_WIDTH: usize = 4;

pub struct Row {
    pub string: String,
    pub render: String,
    pub cx_to_rx: UintVec,
    pub rx_to_cx: UintVec,
    pub cx_to_idx: UintVec,
    pub rx_to_idx: UintVec,
}

impl Row {
    pub fn new(string: String) -> Self {
        let mut row = Self {
            string,
            render: String::new(),
            cx_to_rx: UintVec::new(),
            rx_to_cx: UintVec::new(),
            cx_to_idx: UintVec::new(),
            rx_to_idx: UintVec::new(),
        };
        row.update();
        row
    }

    #[inline]
    pub fn max_cx(&self) -> usize {
        self.cx_to_idx.len() - 1
    }

    #[inline]
    pub fn max_rx(&self) -> usize {
        self.rx_to_idx.len() - 1
    }

    pub fn insert(&mut self, cx: usize, ch: char) {
        let idx = self.cx_to_idx.get(cx);
        self.string.insert(idx, ch);
        self.update();
    }

    pub fn remove(&mut self, cx: usize) {
        let idx = self.cx_to_idx.get(cx);
        self.string.remove(idx);
        self.update();
    }

    pub fn clear(&mut self) {
        self.string.clear();
        self.update();
    }

    pub fn truncate(&mut self, cx: usize) {
        let idx = self.cx_to_idx.get(cx);
        self.string.truncate(idx);
        self.update();
    }

    pub fn split_off(&mut self, cx: usize) -> String {
        let idx = self.cx_to_idx.get(cx);
        let string = self.string.split_off(idx);
        self.update();
        string
    }

    pub fn push_str(&mut self, string: &str) {
        self.string.push_str(string);
        self.update();
    }

    pub fn remove_str(&mut self, from_cx: usize, to_cx: usize) {
        let from_idx = self.cx_to_idx.get(from_cx);
        let to_idx = self.cx_to_idx.get(to_cx);
        let string = self.string.split_off(to_idx);
        self.string.truncate(from_idx);
        self.string.push_str(&string);
        self.update();
    }

    fn update(&mut self) {
        self.render.clear();
        self.cx_to_rx.clear();
        self.rx_to_cx.clear();
        self.cx_to_idx.clear();
        self.rx_to_idx.clear();

        for (cx, (idx, ch)) in self.string.char_indices().enumerate() {
            self.cx_to_rx.push(self.rx_to_idx.len());
            self.cx_to_idx.push(idx);

            if ch == '\t' {
                for _ in 0..(TAB_WIDTH - self.rx_to_idx.len() % TAB_WIDTH) {
                    self.rx_to_cx.push(cx);
                    self.rx_to_idx.push(self.render.len());
                    self.render.push(' ');
                }
            } else {
                for _ in 0..ch.width().unwrap_or(0) {
                    self.rx_to_cx.push(cx);
                    self.rx_to_idx.push(self.render.len());
                }
                self.render.push(ch);
            }
        }

        self.cx_to_rx.push(self.rx_to_idx.len());
        self.rx_to_cx.push(self.cx_to_idx.len());
        self.cx_to_idx.push(self.string.len());
        self.rx_to_idx.push(self.render.len());
    }

    pub fn draw(&self, coloff: usize, width: usize, canvas: &mut Vec<u8>) -> io::Result<()> {
        if self.max_rx() <= coloff {
            return Ok(());
        }

        let mut start_rx = coloff;
        let mut end_rx = cmp::min(coloff + width, self.max_rx());

        let truncate_start =
            start_rx > 0 && self.rx_to_idx.get(start_rx) == self.rx_to_idx.get(start_rx - 1);
        let truncate_end =
            end_rx <= self.max_rx() && self.rx_to_idx.get(end_rx - 1) == self.rx_to_idx.get(end_rx);

        if truncate_start {
            start_rx += 1;
        }
        if truncate_end {
            end_rx -= 1;
        }

        let start_idx = self.rx_to_idx.get(start_rx);
        let end_idx = self.rx_to_idx.get(end_rx);

        if truncate_start {
            canvas.write(b"\x1b[34m~\x1b[39m")?;
        }
        canvas.write(&self.render[start_idx..end_idx].as_bytes())?;
        if truncate_end {
            canvas.write(b"\x1b[34m~\x1b[39m")?;
        }
        Ok(())
    }
}
