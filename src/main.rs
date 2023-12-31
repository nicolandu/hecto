#[warn(clippy::pedantic)]
mod document;
mod editor;
mod row;
mod terminal;

pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

use anyhow::Result;
use std::{env, path::Path};

fn main() -> Result<()> {
    let mut editor = match env::args().nth(1) {
        Some(p) => {
            let path = Path::new(&p);
            if !path.is_file() {
                panic!("Not a file");
            }
            Editor::from_file_path(path)?
        }
        None => Editor::default()?,
    };

    if let Err(e) = editor.run() {
        panic!("{}", e);
    }
    Ok(())
}
