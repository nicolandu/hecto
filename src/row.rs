use std::cmp;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

/// A grapheme-based string.
#[derive(Debug, Default)]
pub struct Row {
    content: String,
    grapheme_count: usize,
}

impl From<String> for Row {
    fn from(string: String) -> Self {
        let mut row = Self {
            content: string,
            grapheme_count: 0,
        };

        row.update_grapheme_count();
        row
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self::from(String::from(slice))
    }
}

impl Row {
    pub fn render(&self, range: Range<usize>) -> String {
        let end = cmp::min(range.end, self.content.len());
        let start = cmp::min(range.start, end);

        let mut result = String::new();

        for grapheme in self.content.graphemes(true).skip(start).take(end - start) {
            result.push_str(match grapheme {
                "\t" => " ",
                g => g,
            });
        }

        result
    }

    #[must_use]
    /// The length of the Row, in graphemes (as defined by Unicode).
    pub fn len(&self) -> usize {
        self.grapheme_count
    }

    #[must_use]
    /// The length of the Row's underlying string, in bytes.
    pub fn len_bytes(&self) -> usize {
        self.content.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Inserts character at index `idx` or appends if `idx` >= `len()`.
    pub fn insert_or_append(&mut self, idx: usize, c: char) {
        if idx >= self.len() {
            self.content.push(c);
        } else {
            // Handle graphemes
            let mut result: String = self.content.graphemes(true).take(idx).collect();
            let remainder: String = self.content.graphemes(true).skip(idx).collect();
            result.push(c);
            result.push_str(&remainder);
            self.content = result;
        }

        self.update_grapheme_count()
    }

    pub fn push(&mut self, other: Self) {
        self.content.push_str(&other.content);
        self.update_grapheme_count();
    }

    /// Noop if `idx` >= `len()`.
    pub fn delete(&mut self, idx: usize) {
        if idx >= self.len() {
            return;
        } else {
            // Handle graphemes
            let mut result: String = self.content.graphemes(true).take(idx).collect();
            // Skip over grapheme to delete
            let remainder: String = self.content.graphemes(true).skip(idx + 1).collect();
            result.push_str(&remainder);
            self.content = result;
        }

        self.update_grapheme_count()
    }

    /// Returns empty Row if `idx` >= `len()`.
    pub fn split(&mut self, idx: usize) -> Self {
        // Handle graphemes
        let before: String = self.content.graphemes(true).take(idx).collect();
        let after: String = self.content.graphemes(true).skip(idx).collect();

        self.content = before;
        self.update_grapheme_count();
        Self::from(after)
    }

    fn update_grapheme_count(&mut self) {
        self.grapheme_count = self.content.graphemes(true).count()
    }
}
