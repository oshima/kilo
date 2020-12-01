use std::io;

use crate::canvas::Canvas;
use crate::coord::{Cursor, Pos, Size};
use crate::face::Face;
use crate::key::Key;
use crate::row::Row;

pub struct Minibuffer {
    pub pos: Pos,
    pub size: Size,
    offset: Pos,
    cursor: Cursor,
    prompt_len: usize,
    row: Row,
}

impl Minibuffer {
    pub fn new() -> Self {
        Self {
            pos: Pos::new(0, 0),
            size: Size::new(0, 0),
            offset: Pos::new(0, 0),
            cursor: Cursor::new(0, 0),
            prompt_len: 0,
            row: Row::new(String::new()),
        }
    }

    pub fn set_message(&mut self, string: &str) {
        self.row.clear();
        self.row.push_str(string);
        self.offset.x = 0;
        self.cursor.x = 0;
        self.prompt_len = 0;
    }

    pub fn set_prompt(&mut self, string: &str) {
        self.row.clear();
        self.row.push_str(string);
        self.offset.x = 0;
        self.cursor.x = self.row.max_x();
        self.prompt_len = self.row.max_x();
    }

    pub fn get_input(&self) -> String {
        self.row.string[self.prompt_len..].to_string()
    }

    pub fn draw(&mut self, canvas: &mut Canvas) -> io::Result<()> {
        canvas.write(format!("\x1b[{};{}H", self.pos.y + 1, self.pos.x + 1).as_bytes())?;
        canvas.set_color(Face::Background)?;

        self.row.faces.clear();
        self.row.faces.resize(self.prompt_len, Face::Prompt);
        self.row.faces.resize(self.row.string.len(), Face::Default);

        self.row
            .draw(canvas, self.offset.x..(self.offset.x + self.size.w))?;

        canvas.write(b"\x1b[K")?;
        canvas.reset_color()?;
        Ok(())
    }

    pub fn draw_cursor(&self, canvas: &mut Canvas) -> io::Result<()> {
        canvas.write(
            format!(
                "\x1b[{};{}H",
                self.pos.y + 1,
                self.pos.x + self.cursor.x - self.offset.x + 1,
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    pub fn process_keypress(&mut self, key: Key) {
        match key {
            Key::ArrowLeft | Key::Ctrl(b'B') => {
                if self.cursor.x > 0 {
                    self.cursor.x = self.row.prev_x(self.cursor.x);
                }
            }
            Key::ArrowRight | Key::Ctrl(b'F') => {
                if self.cursor.x < self.row.max_x() {
                    self.cursor.x = self.row.next_x(self.cursor.x);
                }
            }
            Key::Home | Key::Ctrl(b'A') | Key::Alt(b'<') => {
                self.cursor.x = self.prompt_len;
                self.offset.x = 0;
            }
            Key::End | Key::Ctrl(b'E') | Key::Alt(b'>') => {
                self.cursor.x = self.row.max_x();
            }
            Key::Backspace | Key::Ctrl(b'H') => {
                if self.cursor.x > self.prompt_len {
                    self.cursor.x = self.row.prev_x(self.cursor.x);
                    self.row.remove(self.cursor.x);
                }
            }
            Key::Delete | Key::Ctrl(b'D') => {
                if self.cursor.x >= self.prompt_len && self.cursor.x < self.row.max_x() {
                    self.row.remove(self.cursor.x);
                }
            }
            Key::Ctrl(b'I') => {
                if self.cursor.x >= self.prompt_len {
                    self.row.insert(self.cursor.x, '\t');
                    self.cursor.x = self.row.next_x(self.cursor.x);
                }
            }
            Key::Ctrl(b'K') => {
                if self.cursor.x >= self.prompt_len {
                    self.row.truncate(self.cursor.x);
                }
            }
            Key::Ctrl(b'U') => {
                if self.cursor.x > self.prompt_len {
                    self.row.remove_str(self.prompt_len, self.cursor.x);
                    self.cursor.x = self.prompt_len;
                    self.offset.x = 0;
                }
            }
            Key::Char(ch) => {
                if self.cursor.x >= self.prompt_len {
                    self.row.insert(self.cursor.x, ch);
                    self.cursor.x = self.row.next_x(self.cursor.x);
                }
            }
            _ => (),
        }
        self.scroll();
    }

    fn scroll(&mut self) {
        if self.cursor.x < self.offset.x {
            self.offset.x = self.cursor.x;
        }
        if self.cursor.x >= self.offset.x + self.size.w {
            self.offset.x = self.cursor.x - self.size.w + 1;
        }
    }
}
