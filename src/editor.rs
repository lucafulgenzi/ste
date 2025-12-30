use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::fs;
use std::fs::File;
use std::io::{self, Write};

mod terminal;
mod text_buffer;

use crate::editor::text_buffer::Viewport;
use text_buffer::TextBuffer;

pub struct Buffer {
    cursor_row: usize,
    cursor_col: usize,
    content: TextBuffer,
    viewport_offset: usize,
}

pub struct Editor {
    input_file: InputFile,
    buffer: Buffer,
}

struct InputFile {
    file_exists: bool,
    file_name: String,
    file_path: String,
    file_hash: u64,
}

impl InputFile {
    pub fn new() -> InputFile {
        InputFile {
            file_exists: false,
            file_name: String::new(),
            file_path: String::new(),
            file_hash: 0,
        }
    }
}

impl Editor {
    pub fn new(file_path: String) -> Self {
        let mut editor = Editor {
            input_file: InputFile::new(),
            buffer: Buffer {
                cursor_row: 0,
                cursor_col: 0,
                content: TextBuffer::new(),
                viewport_offset: 0,
            },
        };

        editor.parse_input_file(&file_path);
        editor
    }

    pub fn launch(&mut self) {
        self.clear();
        enable_raw_mode().unwrap();
        self.draw_buffer_loop();
        loop {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read()
            {
                match (code, modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        break;
                    }
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        self.save_buffer();
                    }
                    (KeyCode::Enter, _) => {
                        self.buffer
                            .content
                            .insert_newline(self.buffer.cursor_row, self.buffer.cursor_col);
                        self.buffer.cursor_row += 1;
                        self.buffer.cursor_col = 0;
                    }
                    (KeyCode::Backspace, _) => {
                        if self.buffer.cursor_col > 0 {
                            self.buffer.cursor_col -= 1;
                            self.buffer
                                .content
                                .delete_char(self.buffer.cursor_row, self.buffer.cursor_col);
                        } else if self.buffer.cursor_row > 0 {
                            let prev_line_len =
                                self.buffer.content.line_len(self.buffer.cursor_row - 1);
                            self.buffer.content.merge_lines(self.buffer.cursor_row - 1);
                            self.buffer.cursor_row -= 1;
                            self.buffer.cursor_col = prev_line_len;
                        }
                    }
                    (KeyCode::Delete, _) => {
                        // TODO: add some base features to this function
                        self.buffer
                            .content
                            .delete_char(self.buffer.cursor_row, self.buffer.cursor_col);
                    }
                    (KeyCode::Tab, _) => {
                        self.buffer.content.insert_char(
                            self.buffer.cursor_row,
                            self.buffer.cursor_col,
                            '\t',
                        );
                    }
                    (KeyCode::Up, _) => {
                        if self.buffer.cursor_row > 0 {
                            let prev_line_len =
                                self.buffer.content.line_len(self.buffer.cursor_row - 1);
                            if self.buffer.cursor_col > prev_line_len {
                                self.buffer.cursor_col = prev_line_len;
                            }
                            self.buffer.cursor_row -= 1;
                        }
                        self.buffer.content.remove_empty_lines(self.buffer.cursor_row, false);
                    }
                    (KeyCode::Right, _) => self.buffer.cursor_col += 1,
                    (KeyCode::Down, _) => {
                        self.buffer.cursor_row += 1;

                        let no_more_lines =
                            self.buffer.content.no_more_lines(self.buffer.cursor_row);
                        if no_more_lines {
                            self.buffer
                                .content
                                .insert_newline(self.buffer.cursor_row, 0);
                        }

                        let current_line_len = self.buffer.content.line_len(self.buffer.cursor_row);
                        if self.buffer.cursor_col > current_line_len {
                            self.buffer.cursor_col = current_line_len;
                        }
                    }
                    (KeyCode::Left, _) => {
                        if self.buffer.cursor_col > 0 {
                            self.buffer.cursor_col -= 1
                        }
                    }
                    (KeyCode::End, _) => {
                        let current_line_len = self.buffer.content.line_len(self.buffer.cursor_row);
                        if current_line_len > 0 {
                            self.buffer.cursor_col = current_line_len;
                        }
                    }
                    (KeyCode::Home, _) => {
                        self.buffer.cursor_col = 0;
                    }
                    (KeyCode::Char(c), _) => {
                        self.buffer.content.insert_char(
                            self.buffer.cursor_row,
                            self.buffer.cursor_col,
                            c,
                        );
                        self.buffer.cursor_col += 1;
                    }
                    _ => {}
                }
            }
            self.draw_buffer_loop();
        }

        self.clear();

        disable_raw_mode().unwrap();
    }

    /// Loop method for draw edit-buffer view
    fn draw_buffer_loop(&mut self) {
        self.clear();
        
        // Scroll viewport to keep cursor visible
        let terminal_rows = terminal::get_terminal_rows().unwrap();
        
        // Scroll down if cursor is below viewport
        if self.buffer.cursor_row >= self.buffer.viewport_offset + terminal_rows - 1 {
            self.buffer.viewport_offset = self.buffer.cursor_row.saturating_sub(terminal_rows - 2);
        }
        // Scroll up if cursor is above viewport
        if self.buffer.cursor_row < self.buffer.viewport_offset {
            self.buffer.viewport_offset = self.buffer.cursor_row;
        }
        
        let viewport = Viewport {
            start_row: self.buffer.viewport_offset,
            end_row: self.buffer.viewport_offset + terminal_rows - 1,
        };

        let filled_rows = self.buffer.content.visible_rows(&viewport);

        // Print filled rows
        for row in filled_rows {
            for &ch in row {
                print!("{}", ch);
            }
            print!("\r\n");
        }

        // Print void rows in viewport
        let void_rows = terminal::get_terminal_rows().unwrap() - filled_rows.len() - 1;
        for _ in 0..void_rows {
            print!("~\r\n");
        }

        self.draw_statusbar();

        execute!(
            io::stdout(),
            MoveTo(
                self.buffer.cursor_col as u16,
                (self.buffer.cursor_row - self.buffer.viewport_offset) as u16
            )
        )
        .unwrap();

        io::stdout().flush().unwrap();
    }

    fn draw_statusbar(&mut self) {
        let edit_state = if self.is_file_modified() {
            "(modified)"
        } else {
            ""
        };
        let status_text = format!(
            "{} - {}/{} {}",
            self.input_file.file_path,
            self.buffer.cursor_row + 1,
            self.buffer.content.lines_count(),
            edit_state
        );

        let term_width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80);

        let padding = " ".repeat(term_width.saturating_sub(status_text.len()));

        let s = format!("\x1b[30m\x1b[47m{}{}\x1b[0m", status_text, padding);

        print!("{}", s);
    }

    fn clear(&mut self) {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    }

    fn parse_input_file(&mut self, file_path: &String) {
        self.input_file.file_name = file_path.split('/').last().unwrap().to_string();
        self.input_file.file_path = file_path.to_string();

        if let Ok(contents) = fs::read_to_string(file_path) {
            self.input_file.file_exists = true;

            for (row, line) in contents.lines().enumerate() {
                self.buffer.content.insert_line(row, line.to_string());
            }

            self.input_file.file_hash = self.buffer.content.calculate_hash();
        }
    }

    fn is_file_modified(&self) -> bool {
        let current_hash = self.buffer.content.calculate_hash();
        current_hash != self.input_file.file_hash
    }

    fn save_buffer(&mut self) {
        // TODO: verify clone() speed

        self.buffer.content.remove_empty_lines(self.buffer.cursor_row, true);

        let contents_length = self.buffer.content.lines_count();

        if self.buffer.cursor_row > contents_length {
            self.buffer.cursor_row = contents_length;
        }

        if !self.input_file.file_exists {
            File::create(self.input_file.file_path.clone()).expect("Couldn't create file");
        }
        fs::write(
            self.input_file.file_path.clone(),
            self.buffer.content.to_string().as_bytes(),
        )
        .expect("Couldn't save file");

        self.input_file.file_hash = self.buffer.content.calculate_hash();
    }
}
