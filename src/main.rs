mod document;
mod editor;
mod row;
mod terminal;
mod truncate_graphemes;

pub use document::Document;
use editor::Editor;
pub use editor::{Position, SearchDirection};
pub use row::Row;
pub use terminal::Terminal;
pub use truncate_graphemes::TruncateGraphemes;

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    let mut editor = match env::args().nth(1) {
        Some(p) => Editor::from_file_path(p.into()),
        None => Editor::default(),
    }?;

    if let Err(e) = editor.run() {
        eprintln!("{}", e);
        return Err(e);
    }
    Ok(())
}
