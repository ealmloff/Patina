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
    pub fn new(pos: Pos) -> Self {
        Self { pos }
    }

    pub fn handle_input(
        &mut self,
        data: &dioxus_html::on::KeyboardData,
        rope: &mut Rope,
    ) -> [i32; 2] {
        use dioxus_html::KeyCode::*;
        match data.key_code {
            UpArrow => {
                self.up(rope);
                [0, 0]
            }
            DownArrow => {
                self.down(rope);
                [0, 0]
            }
            RightArrow => {
                self.right(rope);
                [0, 0]
            }
            LeftArrow => {
                self.left(rope);
                [0, 0]
            }
            End => {
                self.pos.col = self.len_line(rope);
                [0, 0]
            }
            Home => {
                self.pos.col = 0;
                [0, 0]
            }
            Backspace => {
                self.realize_col(rope);
                let idx = self.idx(rope);
                if idx > 0 {
                    let old_row = self.pos.row;
                    self.left(rope);
                    rope.remove(idx - 1..idx);
                    if old_row == self.pos.row {
                        [0, -1]
                    } else {
                        [-1, 0]
                    }
                } else {
                    [0, 0]
                }
            }
            Enter => {
                let old_col = self.pos.col;
                rope.insert_char(self.idx(rope), '\n');
                self.pos.col = 0;
                self.down(rope);
                [1, -(old_col as i32)]
            }
            Tab => {
                self.realize_col(rope);
                rope.insert_char(self.idx(rope), '\t');
                self.right(rope);
                [0, 1]
            }
            _ => {
                self.realize_col(rope);
                if data.key.len() == 1 {
                    let c = data.key.chars().next().unwrap();
                    rope.insert_char(self.idx(rope), c);
                    self.right(rope);
                    [0, 1]
                } else {
                    [0, 0]
                }
            }
        }
    }

    pub fn up(&mut self, rope: &Rope) {
        self.move_row(-1, rope);
    }

    pub fn down(&mut self, rope: &Rope) {
        self.move_row(1, rope);
    }

    pub fn right(&mut self, rope: &Rope) {
        self.move_col(1, rope);
    }

    pub fn left(&mut self, rope: &Rope) {
        self.move_col(-1, rope);
    }

    pub fn move_row(&mut self, change: i32, rope: &Rope) {
        let new = self.pos.row as i32 + change;
        if new >= 0 && new < rope.len_lines() as i32 {
            self.pos.row = new as usize;
        }
    }

    pub fn move_col(&mut self, change: i32, rope: &Rope) {
        self.realize_col(rope);
        let idx = self.idx(rope) as i32;
        if idx + change >= 0 && idx + change <= rope.len_chars() as i32 {
            let len_line = self.len_line(rope) as i32;
            let new_col = self.pos.col as i32 + change;
            let diff = new_col - len_line;
            if diff > 0 {
                self.down(rope);
                self.pos.col = 0;
                self.move_col(diff - 1, rope);
            } else if new_col < 0 {
                self.up(rope);
                self.pos.col = self.len_line(rope);
                self.move_col(new_col + 1, rope);
            } else {
                self.pos.col = new_col as usize;
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

#[test]
fn cursor_direction_movement() {
    let mut cursor = Cursor::new(Pos::new(0, 100));
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len());
    cursor.down(&rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(1).unwrap().len());
    cursor.up(&rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len());
    cursor.left(&rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len() - 1);
    cursor.right(&rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len());
}

#[test]
fn cursor_col_movement() {
    let mut cursor = Cursor::new(Pos::new(0, 100));
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    // move inside a row
    cursor.move_col(-5, &rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len() - 5);
    cursor.move_col(5, &rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len());

    // move between rows
    cursor.move_col(3, &rope);
    assert_eq!(cursor.col(&rope), 2);
    cursor.move_col(-3, &rope);
    assert_eq!(cursor.col(&rope), text.lines().nth(0).unwrap().len());

    // don't panic if moving out of range
    cursor.move_col(-100, &rope);
    cursor.move_col(1000, &rope);
}

#[test]
fn cursor_row_movement() {
    let mut cursor = Cursor::new(Pos::new(0, 100));
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    cursor.move_row(1, &rope);
    assert_eq!(cursor.row(), 1);
    cursor.move_row(-1, &rope);
    assert_eq!(cursor.row(), 0);

    // don't panic if moving out of range
    cursor.move_row(-100, &rope);
    cursor.move_row(1000, &rope);
}
