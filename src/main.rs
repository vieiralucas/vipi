extern crate termion;

use std::env;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod buffer;
mod cursor_line;
mod vec2;

use buffer::Buffer;
use cursor_line::CursorLine;
use vec2::Vec2;

#[derive(Debug)]
struct State {
    buffer: Buffer,
}

impl State {
    fn render(&self, term: &mut impl Write) {
        self.buffer.render(term);
    }

    fn update(&mut self, evt: Event) {
        match evt {
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
                self.buffer.move_cursor_right();
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
            _ => {}
        }
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

    let buffer = if let Some(file_path) = env::args().nth(1) {
        Buffer::from_file_path(&file_path)
    } else {
        Buffer::from_lines(vec![])
    };

    let mut state = State { buffer };

    state.buffer.write_debug();

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        write_debug(&format!("{:?}\n", evt));

        if evt == Event::Key(Key::Char('q')) {
            break;
        }

        state.buffer.write_debug();

        state.update(evt);
        state.render(&mut stdout);

        state.buffer.write_debug();
    }
}
