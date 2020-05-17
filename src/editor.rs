use std::cmp;
use std::io::{self, Read, Write};
use std::time::Duration;

use crate::buffer::Buffer;
use crate::key::Key;
use crate::message::Message;

pub struct Editor {
    stdin: io::Stdin,
    stdout: io::Stdout,
    bufout: Vec<u8>,
    width: usize,
    height: usize,
    buffer: Buffer,
    message: Option<Message>,
}

impl Editor {
    pub fn new(filename: Option<String>) -> io::Result<Self> {
        let mut editor = Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            bufout: vec![],
            width: 0,
            height: 0,
            buffer: Buffer::new(filename)?,
            message: None,
        };
        editor.get_window_size()?;
        editor.buffer.set_size(editor.width, editor.height - 1);
        Ok(editor)
    }

    fn get_window_size(&mut self) -> io::Result<()> {
        self.stdout.write(b"\x1b[999C\x1b[999B")?;
        self.stdout.write(b"\x1b[6n")?;
        self.stdout.flush()?;

        let mut buf = [0];
        let mut num = 0;

        while self.stdin.read(&mut buf)? == 1 {
            match buf[0] {
                b'\x1b' | b'[' => (),
                b';' => {
                    self.height = num;
                    num = 0;
                }
                b'R' => {
                    self.width = num;
                    break;
                }
                ch => {
                    num = num * 10 + (ch - b'0') as usize;
                }
            }
        }
        Ok(())
    }

    pub fn set_message(&mut self, text: &str) {
        self.message = Some(Message::new(text));
    }

    pub fn looop(&mut self) -> io::Result<()> {
        let mut quitting = false;

        loop {
            self.refresh_screen()?;
            match self.read_key()? {
                Key::Ctrl(b'q') => {
                    if !self.buffer.modified || quitting {
                        break;
                    } else {
                        quitting = true;
                        self.set_message("File has unsaved changes. Press Ctrl-Q to quit.");
                    }
                }
                key => {
                    quitting = false;
                    self.buffer.process_keypress(key)?;
                }
            }
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        self.bufout.write(b"\x1b[?25l")?;

        self.buffer.draw(&mut self.bufout)?;
        self.draw_message_bar()?; // TODO

        self.buffer.draw_cursor(&mut self.bufout)?;

        self.bufout.write(b"\x1b[?25h")?;

        self.stdout.write(&self.bufout)?;
        self.bufout.clear();

        self.stdout.flush()
    }

    fn draw_message_bar(&mut self) -> io::Result<()> {
        if let Some(message) = &self.message {
            if message.timestamp.elapsed() < Duration::from_secs(5) {
                let len = cmp::min(message.text.as_bytes().len(), self.width);
                self.bufout.write(&message.text.as_bytes()[0..len])?;
            }
        }
        self.bufout.write(b"\x1b[K")?;
        Ok(())
    }

    fn read_key(&mut self) -> io::Result<Key> {
        let mut buf = [0; 4];
        while self.stdin.read(&mut buf)? == 0 {}

        match buf {
            [1..=26, 0, 0, 0] => Ok(Key::Ctrl(b'a' + buf[0] - 1)),
            [127, 0, 0, 0] => Ok(Key::Backspace),
            [b'\x1b', _, 0, 0] => Ok(Key::Alt(buf[1])),
            [b'\x1b', b'[', b'A', 0] => Ok(Key::ArrowUp),
            [b'\x1b', b'[', b'B', 0] => Ok(Key::ArrowDown),
            [b'\x1b', b'[', b'C', 0] => Ok(Key::ArrowRight),
            [b'\x1b', b'[', b'D', 0] => Ok(Key::ArrowLeft),
            [b'\x1b', b'[', b'F', 0] => Ok(Key::End),
            [b'\x1b', b'[', b'H', 0] => Ok(Key::Home),
            [b'\x1b', b'[', b'O', b'F'] => Ok(Key::End),
            [b'\x1b', b'[', b'O', b'H'] => Ok(Key::Home),
            [b'\x1b', b'[', b'1', b'~'] => Ok(Key::Home),
            [b'\x1b', b'[', b'3', b'~'] => Ok(Key::Delete),
            [b'\x1b', b'[', b'4', b'~'] => Ok(Key::End),
            [b'\x1b', b'[', b'5', b'~'] => Ok(Key::PageUp),
            [b'\x1b', b'[', b'6', b'~'] => Ok(Key::PageDown),
            [b'\x1b', b'[', b'7', b'~'] => Ok(Key::Home),
            [b'\x1b', b'[', b'8', b'~'] => Ok(Key::End),
            [b'\x1b', ..] => Ok(Key::Escape),
            _ => Ok(Key::Plain(buf[0])),
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        self.stdout.write(b"\x1b[2J").unwrap();
        self.stdout.write(b"\x1b[H").unwrap();
        self.stdout.flush().unwrap();
    }
}
