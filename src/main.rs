extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Cursor {
    x: u16,
    y: u16,
}
struct State {
    lines: Vec<String>,
    cursor: Cursor,
}

impl State {
    fn render(&self, term: &mut impl Write) {
        let mut y = 1;
        write!(
            term,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )
        .unwrap();
        for line in &self.lines {
            write!(term, "{}", line).unwrap();
            y += 1;
            write!(term, "{}", termion::cursor::Goto(1, y)).unwrap();
        }
        write!(
            term,
            "{}",
            termion::cursor::Goto(self.cursor.x, self.cursor.y)
        )
        .unwrap();

        term.flush().unwrap();
    }

    fn update(&mut self, evt: Event) -> bool {
        match evt {
            Event::Key(Key::Char('q')) => return true,
            Event::Key(Key::Char('h')) => {
                self.cursor.x -= 1;
            }
            Event::Key(Key::Char('j')) => {
                self.cursor.y += 1;
            }
            Event::Key(Key::Char('k')) => {
                self.cursor.y -= 1;
            }
            Event::Key(Key::Char('l')) => {
                self.cursor.x += 1;
            }
            _ => {}
        }

        false
    }
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut state = State {
        lines: vec![
            "Hello, World".to_string(),
            "Line below".to_string(),
            "Line 3".to_string(),
        ],
        cursor: Cursor { x: 1, y: 1 },
    };

    state.render(&mut stdout);
    for c in stdin.events() {
        let evt = c.unwrap();
        let quit = state.update(evt);
        if quit {
            break;
        }

        state.render(&mut stdout);
    }
}
