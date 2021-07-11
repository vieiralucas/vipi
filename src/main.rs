extern crate termion;

use std::env;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod buffer;
mod vec2;

use buffer::Buffer;
use vec2::Vec2;

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    buffer: Buffer,
    command_line: Buffer,
}

impl State {
    fn render(&self, term: &mut impl Write) {
        if self.mode == Mode::Command {
            self.command_line.render(term);
        } else {
            self.buffer.render(term);
        }

        term.flush().unwrap();
    }

    fn update(&mut self, evt: Event) -> bool {
        write_debug(&format!("{:?}", evt));

        match &self.mode {
            Mode::Normal => match evt {
                Event::Key(Key::Char('h')) => {
                    self.buffer.move_cursor_left();
                }
                Event::Key(Key::Char('j')) => {
                    self.buffer.move_cursor_down();
                }
                Event::Key(Key::Char('k')) => {
                    self.buffer.move_cursor_up();
                }
                Event::Key(Key::Char('l')) => {
                    self.buffer.move_cursor_right(false);
                }
                Event::Key(Key::Char('x')) => {
                    self.buffer.delete_char();
                }
                Event::Key(Key::Char('0')) => {
                    self.buffer.move_cursor_first_character();
                }
                Event::Key(Key::Char('w')) => {
                    self.buffer.word_forward();
                }
                Event::Key(Key::Char('J')) => {
                    self.buffer.join_line();
                }
                Event::Key(Key::Char('i')) => self.mode = Mode::Insert,
                Event::Key(Key::Char('a')) => {
                    self.buffer.move_cursor_right(true);
                    self.mode = Mode::Insert;
                }
                Event::Key(Key::Char('o')) => {
                    self.buffer.insert_line_after_cursor("".to_string());
                    self.buffer.move_cursor_down();
                    self.buffer.move_cursor_first_character();
                    self.mode = Mode::Insert;
                }
                Event::Key(Key::Char(':')) => {
                    let size: Vec2 = termion::terminal_size().unwrap().into();

                    self.command_line = Buffer::from_lines(
                        vec![":".to_string()],
                        Vec2::new(0, size.y),
                        Vec2::new(size.x, 1),
                    );
                    self.command_line.move_cursor_right(true);
                    self.mode = Mode::Command;
                }
                _ => {}
            },
            Mode::Command => match evt {
                Event::Key(Key::Esc) => {
                    self.mode = Mode::Normal;
                }
                Event::Key(Key::Backspace) => {
                    if self.command_line.current_line().len() == 1 {
                        self.mode = Mode::Normal;
                    } else {
                        self.command_line.backspace();
                    }
                }
                Event::Key(Key::Char('\n')) => {
                    let line = self.command_line.current_line();
                    let command = line.trim();

                    if command == ":q!" {
                        return true;
                    }

                    let mut parts = command.split(' ');

                    if let Some(":w") = parts.next() {
                        if let Some(file_path) = parts.next() {
                            self.buffer.write_to_file(file_path);
                        }
                    }

                    self.mode = Mode::Normal;
                }
                Event::Key(Key::Char(c)) => {
                    self.command_line.insert_char(c);
                }
                _ => {}
            },
            Mode::Insert => match evt {
                Event::Key(Key::Esc) => {
                    self.buffer.clamp_cursor(false);
                    self.mode = Mode::Normal;
                }
                Event::Key(Key::Char('\n')) => {
                    self.buffer.insert_new_line();
                }
                Event::Key(Key::Backspace) => {
                    self.buffer.backspace();
                }
                Event::Key(Key::Char(c)) => {
                    self.buffer.insert_char(c);
                }
                _ => {}
            },
        }

        false
    }
}

fn write_debug(str: &str) {
    let mut debug_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/tmp/vipi.debug")
        .expect("Failed to open debug file");

    debug_file
        .write_all(&format!("{}\n", str).into_bytes())
        .expect("Failed to write debug file");
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let size: Vec2 = termion::terminal_size().unwrap().into();

    let buffer = if let Some(file_path) = env::args().nth(1) {
        Buffer::from_file_path(&file_path, Vec2::new(0, 0), Vec2::new(size.x, size.y - 1))
    } else {
        Buffer::from_lines(vec![], Vec2::new(0, 0), Vec2::new(size.x, size.y - 1))
    };

    let mut state = State {
        buffer,
        mode: Mode::Normal,
        command_line: Buffer::from_lines(vec![], Vec2::new(0, size.y), Vec2::new(size.x, 1)),
    };

    state.buffer.write_debug();

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        write_debug(&format!("evt: {:?}\n", evt));

        state.buffer.write_debug();

        if state.update(evt) {
            break;
        }
        state.render(&mut stdout);

        state.buffer.write_debug();
    }

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
}
