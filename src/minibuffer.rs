use std::cmp;
use std::io::{self, Write};

use crate::key::Key;
use crate::row::Row;

pub struct Minibuffer {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    cx: usize,
    rx: usize,
    coloff: usize,
    row: Row,
}

impl Minibuffer {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            cx: 0,
            rx: 0,
            coloff: 0,
            row: Row::new(vec![]),
        }
    }

    pub fn set_position(&mut self, x: usize, y: usize, width: usize, height: usize) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    pub fn set_message(&mut self, text: &str) {
        self.row.truncate_chars(0);
        self.row.append_chars(&mut text.as_bytes().to_vec());
    }

    pub fn draw(&mut self, bufout: &mut Vec<u8>) -> io::Result<()> {
        bufout.write(format!("\x1b[{};{}H", self.y + 1, self.x + 1).as_bytes())?;

        if self.row.render.len() > self.coloff {
            let len = cmp::min(self.row.render.len() - self.coloff, self.width);
            bufout.write(&self.row.render[self.coloff..(self.coloff + len)])?;
        }

        bufout.write(b"\x1b[K")?;
        Ok(())
    }

    pub fn draw_cursor(&mut self, bufout: &mut Vec<u8>) -> io::Result<()> {
        bufout.write(
            format!(
                "\x1b[{};{}H",
                self.y + 1,
                self.x + self.rx - self.coloff + 1,
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    pub fn process_keypress(&mut self, key: Key) -> io::Result<()> {
        match key {
            Key::ArrowLeft | Key::Ctrl(b'b') => {
                if self.cx > 0 {
                    self.cx -= 1;
                    self.rx = self.row.cx_to_rx[self.cx];
                }
            }
            Key::ArrowRight | Key::Ctrl(b'f') => {
                if self.cx < self.row.chars.len() {
                    self.cx += 1;
                    self.rx = self.row.cx_to_rx[self.cx];
                }
            }
            Key::Home | Key::Ctrl(b'a') => {
                self.cx = 0;
                self.rx = 0;
            }
            Key::End | Key::Ctrl(b'e') => {
                self.cx = self.row.chars.len();
                self.rx = self.row.cx_to_rx[self.cx];
            }
            Key::Backspace | Key::Ctrl(b'h') => {
                if self.cx > 0 {
                    self.row.delete_char(self.cx - 1);
                    self.cx -= 1;
                    self.rx = self.row.cx_to_rx[self.cx];
                }
            }
            Key::Delete | Key::Ctrl(b'd') => {
                if self.cx < self.row.chars.len() {
                    self.row.delete_char(self.cx);
                }
            }
            Key::Ctrl(b'i') => {
                self.row.insert_char(self.cx, b'\t');
                self.cx += 1;
                self.rx = self.row.cx_to_rx[self.cx];
            }
            Key::Ctrl(b'j') | Key::Ctrl(b'm') => {
                // TODO
            }
            Key::Ctrl(b'k') => {
                self.row.truncate_chars(self.cx);
            }
            Key::Ctrl(b'u') => {
                self.row.truncate_prev_chars(self.cx);
                self.cx = 0;
                self.rx = 0;
            }
            Key::Alt(b'<') => {
                self.cx = 0;
                self.rx = 0;
            }
            Key::Alt(b'>') => {
                self.cx = self.row.chars.len();
                self.rx = self.row.cx_to_rx[self.cx];
            }
            Key::Plain(ch) => {
                self.row.insert_char(self.cx, ch);
                self.cx += 1;
                self.rx = self.row.cx_to_rx[self.cx];
            }
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        if self.rx < self.coloff {
            self.coloff = self.rx;
        }
        if self.rx >= self.coloff + self.width {
            self.coloff = self.rx - self.width + 1;
        }
    }
}
