use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Hash)]
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

    pub fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.lines.hash(&mut hasher);
        hasher.finish()
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if ch == '\n' {
            self.insert_newline(row, col);
        } else if ch == '\t' {
            // TODO: implement text shift
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
        } else {
            self.lines.push(Vec::new());
        }
    }



    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map(|l| l.len()).unwrap_or(0)
    }

    pub fn no_more_lines(&self, row: usize) -> bool {
        self.lines.len() == row
    }

    pub fn visible_rows(&self, viewport: &Viewport) -> &[Vec<char>] {
        let start = viewport.start_row.min(self.lines.len());
        let end = viewport.end_row.min(self.lines.len());
        &self.lines[start..end]
    }

    pub fn lines_count(&self) -> usize {
        self.lines.len()
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

    pub fn remove_empty_lines(&mut self, row: usize, clear_end: bool) {

        if !clear_end && row >= self.lines.len() {
            return;
        }

        let min_lines = if clear_end { 1 } else { row };

        // Remove empty lines from the end until we find a non-empty line
        while self.lines.len() > min_lines && self.lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            self.lines.pop();
        }
    }
}
