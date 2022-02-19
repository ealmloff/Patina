use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Pos {
    row: usize,
    col: usize,
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pos: Pos,
}

impl Cursor {
    pub fn handle_input(&mut self, data: &dioxus_html::on::KeyboardData, rope: &mut Rope) {
        use dioxus_html::KeyCode::*;
        match data.key_code {
            UpArrow => self.up(rope),
            DownArrow => self.down(rope),
            RightArrow => self.right(rope),
            LeftArrow => self.left(rope),
            End => {
                self.pos.col = self.len_line(rope);
            }
            Home => {
                self.pos.col = 0;
            }
            Backspace => {
                self.realize_col(rope);
                let idx = self.idx(rope);
                if idx > 0 {
                    self.left(rope);
                    rope.remove(idx - 1..idx);
                }
            }
            Enter => {
                rope.insert_char(self.idx(rope), '\n');
                self.pos.col = 0;
                self.down(rope);
            }
            Tab => {
                self.realize_col(rope);
                rope.insert_char(self.idx(rope), '\t');
                self.right(rope);
            }
            _ => {
                self.realize_col(rope);
                if data.key.len() == 1 {
                    let c = data.key.chars().next().unwrap();
                    rope.insert_char(self.idx(rope), c);
                    self.right(rope);
                }
            }
        }
    }

    pub fn up(&mut self, rope: &Rope) {
        if self.pos.row > 0 {
            self.pos.row -= 1;
        }
    }

    pub fn down(&mut self, rope: &Rope) {
        if self.pos.row + 1 < rope.len_lines() {
            self.pos.row += 1;
        }
    }

    pub fn right(&mut self, rope: &Rope) {
        if self.idx(rope) < rope.len_chars() {
            if self.pos.col < self.len_line(rope) {
                self.pos.col += 1;
            } else {
                self.down(rope);
                self.pos.col = 0;
            }
        }
    }

    pub fn left(&mut self, rope: &Rope) {
        if self.idx(rope) > 0 {
            self.pos.col = self.col(rope);
            if self.pos.col > 0 {
                self.pos.col -= 1;
            } else {
                self.up(rope);
                self.pos.col = self.len_line(rope);
            }
        }
    }

    pub fn col(&self, rope: &Rope) -> usize {
        self.pos.col.min(self.len_line(rope))
    }

    pub fn row(&self) -> usize {
        self.pos.row
    }

    fn len_line(&self, rope: &Rope) -> usize {
        let line = rope.line(self.pos.row);
        let len = line.len_chars();
        if len > 0 && line.get_char(len - 1) == Some('\n') {
            len - 1
        } else {
            len
        }
    }

    pub fn idx(&self, rope: &Rope) -> usize {
        rope.line_to_char(self.pos.row) + self.col(rope)
    }

    // the column can be more than the line length, cap it
    fn realize_col(&mut self, rope: &Rope) {
        self.pos.col = self.col(rope);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: Pos::new(0, 0),
        }
    }
}
