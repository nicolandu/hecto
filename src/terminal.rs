use crate::Position;
use std::io::{self, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    _stdout: RawTerminal<io::Stdout>,
    size: Size,
}

impl Terminal {
    pub fn init() -> Result<Self, io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            _stdout: io::stdout().into_raw_mode()?,
            size: Size {
                width: size.0,
                height: size.1,
            },
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn clear_screen(&self) {
        print!("{}", termion::clear::All);
    }

    pub fn clear_current_line(&self) {
        print!("{}", termion::clear::CurrentLine);
    }

    /// 0-based coords
    pub fn cursor_position(&self, p: Position) {
        print!(
            "{}",
            termion::cursor::Goto(p.x.saturating_add(1) as u16, p.y.saturating_add(1) as u16)
        );
    }

    pub fn flush(&self) -> Result<(), io::Error> {
        io::stdout().flush()
    }

    pub fn read_key(&self) -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
}
