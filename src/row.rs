use std::cmp;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

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
    /// The length of the Row's underlying string, in bytes.
    pub fn len_bytes(&self) -> usize {
        self.content.len()
    }

    #[must_use]
    /// The length of the Row, in graphemes (as defined by Unicode).
    pub fn len(&self) -> usize {
        self.grapheme_count
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn update_grapheme_count(&mut self) {
        self.grapheme_count = self.content.graphemes(true).count()
    }
}
