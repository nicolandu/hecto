use crate::{terminal, Document, Row, Terminal, TruncateGraphemes};

use anyhow::Result;
use regex::Regex;
use std::cmp;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use termion::event::Key;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP_MESSAGE: &str =
    "<C-Q>: quit (don't save); <C-S>: save; <C-W>: save as; <C-F>: search regex in line; <F1>: Display this help message";

const STATUS_BG_COLOR: terminal::RgbColor = terminal::RgbColor(0, 128, 128);
const LINE_NUM_BG_COLOR: terminal::RgbColor = terminal::RgbColor(255, 255, 255);
const LINE_NUM_FG_COLOR: terminal::RgbColor = terminal::RgbColor(0, 0, 0);
/// Cursor margin at top/bottom
const SCROLL_OFFSET: usize = 5;

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
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

    pub fn from_file_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let doc = Document::open(path.clone());
        let mess = match doc {
            Ok(_) => HELP_MESSAGE.into(),
            Err(_) => format!("Couldn't open file: \"{}\"", path.to_string_lossy()),
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

    fn save(&mut self, always_ask: bool) {
        if always_ask || !self.document.has_path() {
            let path = self
                .prompt("Save as: ", self.document.get_path_string(), |_, _, _| {})
                .unwrap_or(None);

            match path {
                None => {
                    self.status_message = "Save aborted".into();
                    return;
                }
                Some(p) => self.document.set_path(p.into()),
            }
        }

        self.status_message = match self.document.save() {
            Ok(sz) => format!(
                r#""{}" {}L, {sz}B written"#,
                self.document.get_path_string().unwrap_or_default(),
                self.document.len()
            ),
            Err(e) => format!(
                r#""{}" Error writing to file: {}"#,
                self.document.get_path_string().unwrap_or_default(),
                e
            ),
        }
    }

    fn useful_text_width(&self) -> usize {
        let width: usize = self.terminal.size().width.into();
        width.saturating_sub(self.num_col_width())
    }

    fn num_col_width(&self) -> usize {
        (self.document.len().checked_ilog10().unwrap_or(0) + 1 + 1) as _
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
                x: self.cursor_position.x.saturating_sub(self.offset.x) + self.num_col_width() + 1,
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
        for rel_line_num in 0..height {
            Terminal::clear_current_line();

            let line_num = rel_line_num + self.offset.y;
            if let Some(row) = self.document.get(line_num) {
                self.draw_row(row, line_num + 1, self.num_col_width());
            } else if self.document.is_empty() && rel_line_num == height / 3 {
                self.draw_welcome_message(width);
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_row(&self, row: &Row, line_num: usize, num_width: usize) {
        let width = self.useful_text_width();

        let start = self.offset.x;
        let end = start + width;

        let row = row.render(start..end);
        Terminal::set_bg_color(LINE_NUM_BG_COLOR);
        Terminal::set_fg_color(LINE_NUM_FG_COLOR);
        print!("{line_num:>num_width$}");
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
        println!(" {row}\r");
    }

    fn draw_status_bar(&self) {
        let file_name = match self.document.get_file_name() {
            Some(name) => {
                let mut name = name.clone();
                if name.len() <= 30 {
                    name
                } else {
                    name.truncate_graphemes(29);
                    format!("<{name}")
                }
            }

            None => "[Untitled]".into(),
        };

        let modified = if self.document.is_dirty() { " [+]" } else { "" };

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

            format!("{percent_done} [{:>4}:{:<2}]", cursor_y + 1, cursor_x + 1)
        };

        let width: usize = self.terminal.size().width.into();

        let padding = " ".repeat(
            width
                .saturating_sub(file_name.len())
                .saturating_sub(modified.len())
                .saturating_sub(progression.len()),
        );

        let mut status_line = format!("{file_name}{modified}{padding}{progression}");
        status_line.truncate_graphemes(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        println!("{status_line}\r");
        Terminal::reset_bg_color();
    }
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let mut mess = self.status_message.clone();
        mess.truncate_graphemes(self.terminal.size().width.into());
        print!("{}", mess);
    }

    fn draw_welcome_message(&self, width: usize) {
        let message = format!("{NAME} text editor version {VERSION}");
        let len = std::cmp::min(message.len(), width);
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        let mut message = format!("~{spaces}{message}\r");
        message.truncate_graphemes(width);

        println!("{message}\r");
    }

    fn process_keypress(&mut self) -> Result<()> {
        let pressed_key = Terminal::read_key()?;

        #[allow(clippy::single_match)]
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Ctrl('s') => self.save(false),
            Key::Ctrl('w') => self.save(true),
            Key::Ctrl('f') => self.search(),
            Key::F(1) => self.status_message = HELP_MESSAGE.into(),

            Key::Char(c) => {
                self.document.insert_or_append(self.cursor_position, c);
                self.move_cursor(Key::Right);
            }

            Key::Delete => {
                self.document.delete(self.cursor_position);
                self.scroll();
            }
            Key::Backspace => {
                if (self.cursor_position.x > 0) || (self.cursor_position.y > 0) {
                    self.move_cursor(Key::Left);
                    self.document.delete(self.cursor_position);
                    self.scroll();
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

    fn prompt<C>(
        &mut self,
        prompt: &str,
        already_filled: Option<String>,
        callback: C,
    ) -> Result<Option<String>, io::Error>
    where
        C: Fn(&mut Self, Key, &String),
    {
        let mut result = already_filled.unwrap_or_default();
        loop {
            self.status_message = format!("{prompt}{result}\u{258f}");
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Char('\n') => break,
                Key::Char(c) => result.push(c),
                Key::Backspace => {
                    if !result.is_empty() {
                        result.pop();
                    }
                }
                Key::Esc | Key::Ctrl('q') => {
                    result.clear();
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }

        self.status_message.clear();

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    fn search(&mut self) {
        let old_pos = self.cursor_position;

        let query = self
            .prompt("Search: ", None, |editor, key, query| {
                let mut moved = false;
                let direction = match key {
                    Key::Right | Key::Down => {
                        editor.move_cursor(Key::Right);
                        moved = true;
                        SearchDirection::Forward
                    }
                    Key::Left | Key::Up => SearchDirection::Backward,
                    _ => SearchDirection::Forward,
                };

                if let Ok(Some(pos)) = Regex::from_str(query)
                    .map(|r| editor.document.find(&r, editor.cursor_position, direction))
                {
                    editor.cursor_position = pos;
                    editor.scroll()
                }
                // Not found, move back
                else if moved {
                    editor.move_cursor(Key::Left);
                }
            })
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_pos;
            self.scroll();
        }
    }

    fn move_cursor(&mut self, k: Key) {
        let (mut x, mut y) = (self.cursor_position.x, self.cursor_position.y);
        let x_max = match self.document.get(y) {
            Some(row) => row.len(),
            None => 0,
        };
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

        let (width, height) = {
            let s = self.terminal.size();
            (s.width.into(), s.height.into())
        };

        if y < self.offset.y.saturating_add(SCROLL_OFFSET) {
            // If cursor has left top of viewport, scroll and cap offset
            self.offset.y = y.saturating_sub(SCROLL_OFFSET);
        } else if y
            >= self
                .offset
                .y
                .saturating_add(height)
                .saturating_sub(SCROLL_OFFSET)
        {
            // If cursor has left bottom of viewport
            self.offset.y = cmp::min(
                y
                    // These operations need to be in this order for saturating arithmetic to work
                    // properly.
                    .saturating_add(SCROLL_OFFSET)
                    .saturating_sub(height)
                    .saturating_add(1),
                self.document.len().saturating_sub(height),
            );
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
