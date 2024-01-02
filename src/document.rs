use crate::{Position, Row};
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    file_name: Option<String>,
}

impl Document {
    /// # Errors
    /// If file can't be open or line can't be read.
    pub fn open(path: &Path) -> Result<Self, io::Error> {
        let file = fs::File::open(path)?;
        let lines = io::BufReader::new(file)
            .lines()
            .map(|res| Ok(Row::from(res?)))
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(Self {
            rows: lines,
            file_name: path.file_name().map(|name| name.to_string_lossy().into()),
        })
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert_or_append(&mut self, pos: Position, c: char) {
        if c == '\n' {
            self.insert_newline(pos);
            return;
        }

        if pos.y >= self.len() {
            self.rows.push(Row::from(String::from(c)));
        } else {
            self.rows[pos.y].insert_or_append(pos.x, c);
        }
    }

    /// Delete character at `pos`, if it exists.
    /// Joins current row with the next if `pos.x` is at end of Row.
    pub fn delete(&mut self, pos: Position) {
        let len = self.len();
        if pos.y >= len {
            return;
        }

        if pos.x == self.rows[pos.y].len() && pos.y < len.saturating_sub(1) {
            // If at end of row, but not end of file
            let next_row = self.rows.remove(pos.y + 1);
            self.rows[pos.y].push(next_row);
        } else {
            self.rows[pos.y].delete(pos.x);
        }
    }

    #[must_use]
    pub fn get_file_name(&self) -> Option<&String> {
        self.file_name.as_ref()
    }

    pub fn set_file_name(&mut self, name: Option<String>) {
        self.file_name = name;
    }

    /// `pos.y == len()` is allowed, noop if `pos.y` > `len()`.
    fn insert_newline(&mut self, pos: Position) {
        if pos.y > self.len() {
            return;
        }

        let new_row = Row::default();

        if pos.y == self.len() {
            self.rows.push(new_row);
        }

        let new_row = self.rows[pos.y].split(pos.x);
        self.rows.insert(pos.y + 1, new_row);
    }
}
