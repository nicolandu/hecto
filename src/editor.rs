use anyhow::Result;

use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct EditorExit;

impl Error for EditorExit {}

impl fmt::Display for EditorExit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Editor exit")
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
}

#[allow(clippy::unused_self)]
impl Editor {
    pub fn default() -> Self {
        Self { should_quit: false }
    }

    pub fn run(&mut self) -> Result<()> {
        let _stdout = io::stdout().into_raw_mode()?;
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
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));

        if self.should_quit {
            println!("Goodbye!\r");
        } else {
            self.draw_rows();
            print!("{}", termion::cursor::Goto(1, 1));
        }

        io::stdout().flush()
    }

    fn draw_rows(&self) {
        for _ in 0..24 {
            println!("~\r");
        }
    }

    fn process_keypress(&mut self) -> Result<()> {
        let pressed_key = read_key()?;

        #[allow(clippy::single_match)]
        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            _ => (),
        }

        Ok(())
    }
}

fn read_key() -> Result<Key, io::Error> {
    loop {
        // This Option<_> comes from next().
        if let Some(key) = io::stdin().lock().keys().next() {
            return key;
        }
    }
}
