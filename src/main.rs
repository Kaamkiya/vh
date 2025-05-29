use clap::Parser;

mod editor;
use editor::Editor;

const VH_VERSION: &'static str = "0.0.1";

#[derive(Parser)]
struct Cli {
    filename: String,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    Editor::default(cli.filename).run()?;

    Ok(())
}
