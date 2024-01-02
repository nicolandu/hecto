use crate::{terminal, Document, Row, Terminal};

use anyhow::Result;
use std::cmp;
use std::io;
use std::path::Path;
use termion::event::Key;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const STATUS_BG_COLOR: terminal::RgbColor = terminal::RgbColor(128, 128, 255);

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    document: Document,
    status_message: String,
    cursor_position: Position,
    offset: Position,
}

#[allow(clippy::unused_self)]
impl Editor {
    pub fn default() -> Result<Self, std::io::Error> {
        Self::common_init(Document::default(), "".into())
    }

    pub fn from_file_path(path: &Path) -> Result<Self, std::io::Error> {
        let doc = Document::open(path);
        let mess = match doc {
            Ok(_) => "".into(),
            Err(_) => format!("Couldn't open file: \"{}\"", path.to_string_lossy(),),
        };
        Self::common_init(doc.unwrap_or_default(), mess)
    }

    #[inline(always)]
    fn common_init(document: Document, status_message: String) -> Result<Self, std::io::Error> {
        Ok(Self {
            should_quit: false,
            terminal: Terminal::init()?,
            document,
            status_message,
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
        Terminal::cursor_position(Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye!\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::flush()
    }

    fn draw_rows(&self) {
        let (width, height): (usize, usize) = {
            let s = self.terminal.size();
            (s.width.into(), s.height.into())
        };

        // Terminal::size already takes care of leaving space for status bars
        for l in 0..height {
            Terminal::clear_current_line();
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

    fn draw_status_bar(&self) {
        let file_name = match self.document.get_file_name() {
            Some(name) => {
                let mut name = name.clone();
                if name.len() <= 30 {
                    name
                } else {
                    name.truncate(29);
                    format!("<{name}")
                }
            }

            None => "[Untitled]".into(),
        };

        let progression = {
            let cursor_x = self.cursor_position.x;
            let cursor_y = self.cursor_position.y;

            let percent_done = {
                let y_max = self.document.len().saturating_sub(1);

                if cursor_y == 0 {
                    "Top".into()
                } else if cursor_y == y_max {
                    "Bot".into()
                } else {
                    format!("{}%", cursor_y.saturating_mul(100) / y_max)
                }
            };

            format!("{percent_done} [{}:{}]", cursor_y + 1, cursor_x + 1)
        };

        let width: usize = self.terminal.size().width.into();

        let spaces = " ".repeat(
            width
                .saturating_sub(file_name.len())
                .saturating_sub(progression.len()),
        );

        let mut status_line = format!("{file_name}{spaces}{progression}");
        status_line.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        println!("{status_line}\r");
        Terminal::reset_bg_color();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let mut mess = self.status_message.clone();
        mess.truncate(self.terminal.size().width.into());
        print!("{}", mess);
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
        let pressed_key = Terminal::read_key()?;

        #[allow(clippy::single_match)]
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Char(c) => {
                self.document.insert_or_append(self.cursor_position, c);
                self.move_cursor(Key::Right);
            }
            Key::Delete => self.document.delete(self.cursor_position),
            Key::Backspace => {
                if (self.cursor_position.x > 0) || (self.cursor_position.y > 0) {
                    self.move_cursor(Key::Left);
                    self.document.delete(self.cursor_position);
                }
            }
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
        dbg!(x_max);
        let y_max = self.document.len().saturating_sub(1);

        let height: usize = self.terminal.size().height.into();

        match k {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => y = cmp::min(y.saturating_add(1), y_max),
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = match self.document.get(y) {
                        Some(row) => row.len(),
                        None => 0,
                    };
                }
            }

            Key::Right => {
                if x < x_max {
                    x += 1;
                } else if y < y_max {
                    y += 1;
                    x = 0;
                }
            }

            Key::PageUp => y = y.saturating_sub(height),
            Key::PageDown => y = cmp::min(y.saturating_add(height), y_max),
            Key::Home => x = 0,
            Key::End => x = x_max,
            _ => (),
        }

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
