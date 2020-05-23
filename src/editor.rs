use std::io::{self, Read, Write};

use crate::buffer::Buffer;
use crate::key::Key;
use crate::minibuffer::Minibuffer;

enum State {
    Default,
    Save,
    Quit,
    Quitted,
}

pub struct Editor {
    stdin: io::Stdin,
    stdout: io::Stdout,
    bufout: Vec<u8>,
    width: usize,
    height: usize,
    state: State,
    buffer: Buffer,
    minibuffer: Minibuffer,
}

impl Editor {
    pub fn new(filename: Option<String>) -> io::Result<Self> {
        let mut editor = Self {
            stdin: io::stdin(),
            stdout: io::stdout(),
            bufout: vec![],
            width: 0,
            height: 0,
            state: State::Default,
            buffer: Buffer::new(filename)?,
            minibuffer: Minibuffer::new(),
        };
        editor.get_window_size()?;
        editor
            .buffer
            .set_position(0, 0, editor.width, editor.height - 1);
        editor
            .minibuffer
            .set_position(0, editor.height - 1, editor.width, 1);
        editor.minibuffer.set_message("Press Ctrl-Q to quit");
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

    pub fn looop(&mut self) -> io::Result<()> {
        loop {
            self.refresh_screen()?;

            let key = self.read_key()?;
            self.process_keypress(key)?;

            if let State::Quitted = self.state {
                break;
            }
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        self.bufout.write(b"\x1b[?25l")?;

        self.buffer.draw(&mut self.bufout)?;
        self.minibuffer.draw(&mut self.bufout)?;

        match self.state {
            State::Default => self.buffer.draw_cursor(&mut self.bufout)?,
            _ => self.minibuffer.draw_cursor(&mut self.bufout)?,
        }

        self.bufout.write(b"\x1b[?25h")?;

        self.stdout.write(&self.bufout)?;
        self.bufout.clear();
        self.stdout.flush()
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

    fn process_keypress(&mut self, key: Key) -> io::Result<()> {
        match self.state {
            State::Default => match key {
                Key::Ctrl(b's') => {
                    if self.buffer.filename.is_none() {
                        self.minibuffer.set_prompt("Save as: ");
                        self.state = State::Save;
                    } else {
                        self.buffer.save()?;
                        self.minibuffer.set_message("Saved");
                    }
                }
                Key::Ctrl(b'q') => {
                    if self.buffer.modified {
                        self.minibuffer.set_prompt("Quit without saving? (Y/n): ");
                        self.state = State::Quit;
                    } else {
                        self.state = State::Quitted;
                    }
                }
                _ => self.buffer.process_keypress(key),
            },
            State::Save => match key {
                Key::Ctrl(b'g') => {
                    self.minibuffer.set_message("");
                    self.state = State::Default;
                }
                Key::Ctrl(b'j') | Key::Ctrl(b'm') => {
                    let input = self.minibuffer.get_input();
                    self.buffer.filename = Some(input);
                    self.buffer.save()?;
                    self.minibuffer.set_message("");
                    self.state = State::Default;
                }
                _ => self.minibuffer.process_keypress(key),
            },
            State::Quit => match key {
                Key::Ctrl(b'g') => {
                    self.minibuffer.set_message("");
                    self.state = State::Default;
                }
                Key::Ctrl(b'j') | Key::Ctrl(b'm') => {
                    let input = self.minibuffer.get_input();
                    if input.is_empty() || input.to_lowercase() == "y" {
                        self.state = State::Quitted;
                    } else {
                        self.minibuffer.set_message("");
                        self.state = State::Default;
                    }
                }
                _ => self.minibuffer.process_keypress(key),
            },
            State::Quitted => unreachable!(),
        }
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        self.stdout.write(b"\x1b[2J").unwrap();
        self.stdout.write(b"\x1b[H").unwrap();
        self.stdout.flush().unwrap();
    }
}
