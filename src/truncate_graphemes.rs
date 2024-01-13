use unicode_segmentation::UnicodeSegmentation;

/// Similar to `String::truncate`, `TruncateGraphemes::truncate_graphemes` is used to truncate a String.
/// However, this function indexes uses Unicode graphemes, not bytes, and doesn't panic.
pub trait TruncateGraphemes {
    fn truncate_graphemes(&mut self, new_len_glyphs: usize);
}

impl TruncateGraphemes for String {
    /// This is O(n), vs O(1) for truncate
    fn truncate_graphemes(&mut self, new_len_glyphs: usize) {
        if let Some(idx) = self.grapheme_indices(true).nth(new_len_glyphs) {
            self.truncate(idx.0);
        }
    }
}
