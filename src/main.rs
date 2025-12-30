use crate::editor::Editor;
use clap::Parser;

use simplelog::*;

use std::fs::File;

mod editor;

#[derive(Parser)]
struct Args {
    /// Input file
    file: String,
}

fn main() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("ste.log").unwrap(),
    )])
    .unwrap();

    let args = Args::parse();

    let mut editor = Editor::new(args.file);
    editor.launch();
}
