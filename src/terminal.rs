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

pub struct RgbColor(pub u8, pub u8, pub u8);

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
                height: size.1.saturating_sub(2),
            },
        })
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    /// 0-based coords
    pub fn cursor_position(pos: Position) {
        print!(
            "{}",
            termion::cursor::Goto(
                pos.x.saturating_add(1) as u16,
                pos.y.saturating_add(1) as u16
            )
        );
    }

    pub fn set_bg_color(color: RgbColor) {
        print!(
            "{}",
            termion::color::Bg(termion::color::Rgb(color.0, color.1, color.2))
        );
    }

    pub fn reset_bg_color() {
        print!("{}", termion::color::Bg(termion::color::Reset));
    }

    pub fn set_fg_color(color: RgbColor) {
        print!(
            "{}",
            termion::color::Fg(termion::color::Rgb(color.0, color.1, color.2))
        );
    }

    pub fn reset_fg_color() {
        print!("{}", termion::color::Fg(termion::color::Reset));
    }

    pub fn flush() -> Result<(), io::Error> {
        io::stdout().flush()
    }

    pub fn read_key() -> Result<Key, io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
}
