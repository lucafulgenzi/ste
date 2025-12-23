pub struct TextBuffer {
    lines: Vec<Vec<char>>,
}

pub struct Viewport {
    pub(crate) start_row: usize,
    pub(crate) end_row: usize,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            lines: vec![Vec::new()],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<char> {
        self.lines.get(row)?.get(col).copied()
    }

    pub fn set(&mut self, row: usize, col: usize, ch: char) {
        if let Some(line) = self.lines.get_mut(row) {
            if col < line.len() {
                line[col] = ch
            } else if col == line.len() {
                line.push(ch)
            }
        }
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if ch == '\n' {
            self.insert_newline(row, col);
        } else if let Some(line) = self.lines.get_mut(row) {
            if col <= line.len() {
                line.insert(col, ch);
            }
        }
    }

    pub fn insert_line(&mut self, row: usize, line: String) {
        while self.lines.len() <= row {
            self.lines.push(Vec::new());
        }

        self.lines[row] = line.chars().collect();
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(row) {
            if col < line.len() {
                line.remove(col);
            }
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if let Some(line) = self.lines.get_mut(row) {
            let remaining: Vec<char> = line.drain(col..).collect();
            self.lines.insert(row + 1, remaining);
        }
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map(|l| l.len()).unwrap_or(0)
    }

    pub fn no_more_lines(&self, row: usize) -> bool {
        self.lines.get(row).is_none()
    }

    pub fn rows(&self) -> &[Vec<char>] {
        &self.lines
    }

    pub fn visible_rows(&self, viewport: &Viewport) -> &[Vec<char>] {
        let start = viewport.start_row.min(self.lines.len());
        let end = viewport.end_row.min(self.lines.len());
        &self.lines[start..end]
    }

    pub fn merge_lines(&mut self, row: usize) {
        if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            if let Some(current_line) = self.lines.get_mut(row) {
                current_line.extend(next_line);
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.iter().collect::<String>())
            .fold(String::new(), |mut acc, line| {
                if !acc.is_empty() {
                    acc.push('\n');
                }
                acc.push_str(&line);
                acc
            })
    }
}
