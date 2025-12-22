use crate::editor::Editor;

extern crate simplelog;

use simplelog::*;

use std::fs::File;

mod editor;

fn main() {

    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("ste.log").unwrap()),
        ]
    ).unwrap();

    let mut editor = Editor::new();
    editor.launch();
}
