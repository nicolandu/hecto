#[warn(clippy::pedantic)]
mod editor;

use anyhow::Result;
use editor::Editor;

fn main() -> Result<()> {
    Editor::default().run()
}
