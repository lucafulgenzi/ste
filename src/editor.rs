use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, Read, Write};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

use log::info;

mod text_buffer;
use crate::editor::text_buffer::Viewport;
use text_buffer::TextBuffer;

pub struct Buffer {
    cursor_row: usize,
    cursor_col: usize,
    content: TextBuffer,
}

pub struct Editor {
    buffer: Buffer,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            buffer: Buffer {
                cursor_row: 0,
                cursor_col: 0,
                content: TextBuffer::new(),
            },
        }
    }

    pub fn launch(&mut self) {
        self.clear();
        enable_raw_mode().unwrap();
        loop {
            if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = read() {
                match (code, modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        break;
                    },
                    (KeyCode::Enter, _) => {
                        self.buffer
                            .content
                            .insert_newline(self.buffer.cursor_row, self.buffer.cursor_col);
                        self.buffer.cursor_row += 1;
                        self.buffer.cursor_col = 0;
                    },
                    (KeyCode::Backspace, _) => {
                        if self.buffer.cursor_col > 0 {
                            self.buffer.cursor_col -= 1;
                            self.buffer.content.delete_char(
                                self.buffer.cursor_row,
                                self.buffer.cursor_col,
                            );
                        } else if self.buffer.cursor_row > 0 {
                            let prev_line_len = self.buffer.content.line_len(self.buffer.cursor_row - 1);
                            self.buffer.content.merge_lines(self.buffer.cursor_row - 1);
                            self.buffer.cursor_row -= 1;
                            self.buffer.cursor_col = prev_line_len;
                        }
                    },
                    (KeyCode::Up, _) => {
                        if self.buffer.cursor_row > 0 {
                            let prev_line_len = self.buffer.content.line_len(self.buffer.cursor_row - 1);
                            if self.buffer.cursor_col > prev_line_len {
                                self.buffer.cursor_col = prev_line_len;
                            }
                            self.buffer.cursor_row -= 1;
                        }
                    }
                    (KeyCode::Right, _) => {
                        self.buffer.cursor_col += 1
                    }
                    (KeyCode::Down, _) => {
                        let next_line_len = self.buffer.content.line_len(self.buffer.cursor_row + 1);
                        if self.buffer.cursor_col > next_line_len {
                            self.buffer.cursor_col = next_line_len;
                        }
                        self.buffer.cursor_row += 1;
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
            self.buffer_loop();
        }

        disable_raw_mode().unwrap();
    }

    fn buffer_loop(&mut self) {
        self.clear();
        let viewport = Viewport {
            start_row: 0,
            end_row: 24,
        };

        for row in self.buffer.content.visible_rows(&viewport) {
            for &ch in row {
                print!("{}", ch);
            }
            print!("\r\n");
        }

        execute!(
            io::stdout(),
            MoveTo(self.buffer.cursor_col as u16, self.buffer.cursor_row as u16)
        )
        .unwrap();

        io::stdout().flush().unwrap();
    }

    fn clear(&mut self) {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
    }
}
