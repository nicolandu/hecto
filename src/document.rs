use regex::Regex;

use crate::{Position, Row, SearchDirection};
use std::fs;
use std::io::{self, BufRead, Seek, Write};
use std::path::PathBuf;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    path: Option<PathBuf>,
    /// Whether the document was modified since last save.
    dirty: bool,
}

impl Document {
    /// # Errors
    /// If file can't be opened or line can't be read.
    pub fn open(path: PathBuf) -> Result<Self, io::Error> {
        let file = fs::File::open(&path)?;
        let lines = io::BufReader::new(file)
            .lines()
            .map(|res| Ok(Row::from(res?)))
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(Self {
            rows: lines,
            path: Some(path),
            dirty: false,
        })
    }

    /// Returns number of bytes written to disk.
    /// # Errors
    /// If file can'be opened or line can't be written.
    pub fn save(&mut self) -> Result<u64, io::Error> {
        let mut bytes_written = 0;
        if let Some(ref path) = self.path {
            let mut file = fs::File::create(path)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }

            bytes_written = file.seek(io::SeekFrom::End(0))?;
        }

        self.dirty = false;
        Ok(bytes_written)
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use]
    pub fn find(
        &self,
        query: &Regex,
        limit: Position,
        direction: SearchDirection,
    ) -> Option<Position> {
        if limit.y > self.len() {
            return None;
        };

        let mut pos = limit;

        let (start, end) = match direction {
            SearchDirection::Forward => (limit.y, self.len()),
            SearchDirection::Backward => (0, limit.y + 1),
        };

        for _ in start..end {
            let row = self.rows.get(pos.y)?;

            if let Some(x) = row.find(&query, pos.x, direction) {
                pos.x = x;
                return Some(pos);
            }
            match direction {
                SearchDirection::Forward => {
                    pos.y = pos.y.saturating_add(1);
                    pos.x = 0;
                }
                SearchDirection::Backward => {
                    pos.y = pos.y.saturating_sub(1);
                    pos.x = self.rows[pos.y].len();
                }
            }
        }

        None
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn insert_or_append(&mut self, pos: Position, c: char) {
        if c == '\n' {
            self.insert_newline(pos);
            return;
        }

        self.dirty = true;

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

        self.dirty = true;

        if pos.x == self.rows[pos.y].len() && pos.y < len.saturating_sub(1) {
            // If at end of row, but not end of file
            let next_row = self.rows.remove(pos.y.saturating_add(1));
            self.rows[pos.y].push(next_row);
        } else {
            self.rows[pos.y].delete(pos.x);
        }
    }

    #[must_use]
    pub fn get_file_name(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| p.file_name().map(|name| name.to_string_lossy().into()))
    }

    #[must_use]
    pub fn get_path_string(&self) -> Option<String> {
        self.path.as_ref().map(|p| p.to_string_lossy().into())
    }

    #[must_use]
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    /// `pos.y == len()` is allowed, noop if `pos.y` > `len()`.
    fn insert_newline(&mut self, pos: Position) {
        if pos.y > self.len() {
            return;
        }

        self.dirty = true;

        let new_row = Row::default();

        if pos.y == self.len() {
            self.rows.push(new_row);
        }

        let new_row = self.rows[pos.y].split(pos.x);
        self.rows.insert(pos.y.saturating_add(1), new_row);
    }
}
