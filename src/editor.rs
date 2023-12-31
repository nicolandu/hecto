use crate::{Document, Row, Terminal};

use anyhow::Result;
use std::io;
use std::path::Path;
use termion::event::Key;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    document: Document,
    cursor_position: Position,
    offset: Position,
}

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[allow(clippy::unused_self)]
impl Editor {
    pub fn default() -> Result<Self, std::io::Error> {
        Self::common_init(Document::default())
    }

    pub fn from_file_path(path: &Path) -> Result<Self, std::io::Error> {
        Self::common_init(Document::open(path)?)
    }

    #[inline(always)]
    fn common_init(document: Document) -> Result<Self, std::io::Error> {
        Ok(Self {
            should_quit: false,
            terminal: Terminal::init()?,
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("<C-Q> to quit\r");
        loop {
            self.refresh_screen()?;

            if self.should_quit {
                return Ok(());
            }

            self.process_keypress()?;
        }
    }

    fn refresh_screen(&self) -> Result<(), io::Error> {
        self.terminal.cursor_position(Position::default());
        if self.should_quit {
            self.terminal.clear_screen();
            println!("Goodbye!\r");
        } else {
            self.draw_rows();
            self.terminal.cursor_position(Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        self.terminal.flush()
    }

    fn draw_rows(&self) {
        let (width, height): (usize, usize) = {
            let s = self.terminal.size();
            (s.width.into(), s.height.into())
        };

        // Leave last row intact for status bar
        for l in 0..height - 1 {
            self.terminal.clear_current_line();
            if let Some(row) = self.document.get(l + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && l == height / 3 {
                self.draw_welcome_message(width);
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row) {
        let width: usize = self.terminal.size().width.into();
        let start = self.offset.x;
        let end = start + width;

        let row = row.render(start..end);
        println!("{row}\r");
    }

    fn draw_welcome_message(&self, width: usize) {
        let message = format!("{NAME} text editor version {VERSION} (Ctrl-Q to quit)");
        let len = std::cmp::min(message.len(), width);
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        let mut message = format!("~{spaces}{message}\r");
        message.truncate(width);

        println!("{message}\r");
    }

    fn process_keypress(&mut self) -> Result<()> {
        let pressed_key = self.terminal.read_key()?;

        #[allow(clippy::single_match)]
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End => self.move_cursor(pressed_key),
            _ => (),
        }

        Ok(())
    }

    fn move_cursor(&mut self, k: Key) {
        let (mut x, mut y) = (self.cursor_position.x, self.cursor_position.y);
        let x_max = match self.document.get(y) {
            Some(row) => row.len(),
            None => 0,
        };
        let y_max = self.document.len();

        let height: usize = self.terminal.size().height.into();

        match k {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => y = y.saturating_add(1),
            Key::Left => x = x.saturating_sub(1),
            Key::Right => x = x.saturating_add(1),

            Key::PageUp => y = y.saturating_sub(height),
            // No need to clamp here, will be clamped shortly after
            Key::PageDown => y = y.saturating_add(height),
            Key::Home => x = 0,
            Key::End => x = x_max,
            _ => (),
        }

        // Clamp x and y
        let x = std::cmp::min(x, x_max);
        let y = std::cmp::min(y, y_max);

        // Re-snap x to width for new line
        let x_max = match self.document.get(y) {
            Some(row) => row.len(),
            None => 0,
        };
        let x = std::cmp::min(x, x_max);

        self.cursor_position = Position { x, y };
        self.scroll();
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;

        let (width, height): (usize, usize) = {
            let s = self.terminal.size();
            (s.width.into(), s.height.into())
        };

        if y < self.offset.y {
            // If cursor has left top of viewport
            self.offset.y = y
        } else if y >= self.offset.y.saturating_add(height) {
            // If cursor has left bottom of viewport
            self.offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < self.offset.x {
            // If cursor has left top of viewport
            self.offset.x = x
        } else if x >= self.offset.x.saturating_add(width) {
            // If cursor has left bottom of viewport
            self.offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
}
