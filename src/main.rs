mod editor;
use editor::Editor;

fn main() -> std::io::Result<()> {
    Editor::default("blue.txt".to_string()).run()?;

    Ok(())
}
