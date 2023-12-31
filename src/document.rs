use crate::Row;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
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
        Ok(Self { rows: lines })
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
}
