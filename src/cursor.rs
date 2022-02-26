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
        let new = self.row as i32 + change;
        if new >= 0 && new < rope.len_lines() as i32 {
            self.row = new as usize;
        }
    }

    pub fn move_col(&mut self, change: i32, rope: &Rope) {
        self.realize_col(rope);
        let idx = self.idx(rope) as i32;
        if idx + change >= 0 && idx + change <= rope.len_chars() as i32 {
            let len_line = self.len_line(rope) as i32;
            let new_col = self.col as i32 + change;
            let diff = new_col - len_line;
            if diff > 0 {
                self.down(rope);
                self.col = 0;
                self.move_col(diff - 1, rope);
            } else if new_col < 0 {
                self.up(rope);
                self.col = self.len_line(rope);
                self.move_col(new_col + 1, rope);
            } else {
                self.col = new_col as usize;
            }
        }
    }

    pub fn move_col_raw(&mut self, change: i32) {
        self.col = (self.col as i32 + change) as usize;
    }

    pub fn col(&self, rope: &Rope) -> usize {
        self.col.min(self.len_line(rope))
    }

    pub fn row(&self) -> usize {
        self.row
    }

    fn len_line(&self, rope: &Rope) -> usize {
        let line = rope.line(self.row);
        let len = line.len_chars();
        if len > 0 && line.get_char(len - 1) == Some('\n') {
            len - 1
        } else {
            len
        }
    }

    pub fn idx(&self, rope: &Rope) -> usize {
        rope.line_to_char(self.row) + self.col(rope)
    }

    // the column can be more than the line length, cap it
    fn realize_col(&mut self, rope: &Rope) {
        self.col = self.col(rope);
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub start: Pos,
    pub end: Option<Pos>,
}

impl Cursor {
    pub fn new(pos: Pos) -> Self {
        Self {
            start: pos,
            end: None,
        }
    }

    pub fn handle_input(
        &mut self,
        data: &dioxus_html::on::KeyboardData,
        rope: &mut Rope,
    ) -> [i32; 2] {
        use dioxus_html::KeyCode::*;
        match data.key_code {
            UpArrow => {
                if data.shift_key {
                    self.with_end(|c| c.up(rope));
                } else {
                    self.start.up(rope);
                    self.end = None;
                }
                [0, 0]
            }
            DownArrow => {
                if data.shift_key {
                    self.with_end(|c| c.down(rope));
                } else {
                    self.start.down(rope);
                    self.end = None;
                }
                [0, 0]
            }
            RightArrow => {
                if data.shift_key {
                    self.with_end(|c| c.right(rope));
                } else {
                    self.start.right(rope);
                    self.end = None;
                }
                [0, 0]
            }
            LeftArrow => {
                if data.shift_key {
                    self.with_end(|c| c.left(rope));
                } else {
                    self.start.left(rope);
                    self.end = None;
                }
                [0, 0]
            }
            End => {
                self.start.col = self.start.len_line(rope);
                self.end = None;
                [0, 0]
            }
            Home => {
                self.start.col = 0;
                self.end = None;
                [0, 0]
            }
            Backspace => {
                self.start.realize_col(rope);
                let idx = self.start.idx(rope);
                if idx > 0 {
                    let old_row = self.start.row;
                    self.start.left(rope);
                    rope.remove(idx - 1..idx);
                    if old_row == self.start.row {
                        [0, -1]
                    } else {
                        [-1, 0]
                    }
                } else {
                    [0, 0]
                }
            }
            Enter => {
                let old_col = self.start.col(&rope);
                rope.insert_char(self.start.idx(rope), '\n');
                self.start.col = 0;
                self.start.down(rope);
                [1, -(old_col as i32)]
            }
            Tab => {
                self.start.realize_col(rope);
                rope.insert_char(self.start.idx(rope), '\t');
                self.start.right(rope);
                [0, 1]
            }
            _ => {
                self.start.realize_col(rope);
                if data.key.len() == 1 {
                    let c = data.key.chars().next().unwrap();
                    rope.insert_char(self.start.idx(rope), c);
                    self.start.right(rope);
                    [0, 1]
                } else {
                    [0, 0]
                }
            }
        }
    }

    pub fn with_end(&mut self, f: impl FnOnce(&mut Pos)) {
        let mut new = self.end.take().unwrap_or(self.start.clone());
        f(&mut new);
        self.end.replace(new);
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            start: Pos::new(0, 0),
            end: None,
        }
    }
}

#[test]
fn pos_direction_movement() {
    let mut pos = Pos::new(0, 100);
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len());
    pos.down(&rope);
    assert_eq!(pos.col(&rope), text.lines().nth(1).unwrap().len());
    pos.up(&rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len());
    pos.left(&rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len() - 1);
    pos.right(&rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len());
}

#[test]
fn pos_col_movement() {
    let mut pos = Pos::new(0, 100);
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    // move inside a row
    pos.move_col(-5, &rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len() - 5);
    pos.move_col(5, &rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len());

    // move between rows
    pos.move_col(3, &rope);
    assert_eq!(pos.col(&rope), 2);
    pos.move_col(-3, &rope);
    assert_eq!(pos.col(&rope), text.lines().nth(0).unwrap().len());

    // don't panic if moving out of range
    pos.move_col(-100, &rope);
    pos.move_col(1000, &rope);
}

#[test]
fn cursor_row_movement() {
    let mut pos = Pos::new(0, 100);
    let text = "hello world\nhi";
    let rope = Rope::from_str(text);

    pos.move_row(1, &rope);
    assert_eq!(pos.row(), 1);
    pos.move_row(-1, &rope);
    assert_eq!(pos.row(), 0);

    // don't panic if moving out of range
    pos.move_row(-100, &rope);
    pos.move_row(1000, &rope);
}

#[test]
fn cursor_input() {
    let mut cursor = Cursor::new(Pos::new(0, 0));
    let text = "hello world\nhi";
    let mut rope = Rope::from_str(text);

    for _ in 0..5 {
        cursor.handle_input(
            &dioxus_html::on::KeyboardData {
                char_code: 0,
                key: "".to_string(),
                key_code: dioxus_html::KeyCode::RightArrow,
                alt_key: false,
                ctrl_key: false,
                meta_key: false,
                shift_key: false,
                locale: "".to_string(),
                location: 0,
                repeat: false,
                which: 0,
            },
            &mut rope,
        );
    }

    for _ in 0..5 {
        cursor.handle_input(
            &dioxus_html::on::KeyboardData {
                char_code: 0,
                key: "".to_string(),
                key_code: dioxus_html::KeyCode::Backspace,
                alt_key: false,
                ctrl_key: false,
                meta_key: false,
                shift_key: false,
                locale: "".to_string(),
                location: 0,
                repeat: false,
                which: 0,
            },
            &mut rope,
        );
    }

    assert_eq!(rope.to_string(), " world\nhi");

    cursor.handle_input(
        &dioxus_html::on::KeyboardData {
            char_code: 'h'.into(),
            key: 'h'.to_string(),
            key_code: dioxus_html::KeyCode::H,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            locale: "".to_string(),
            location: 0,
            repeat: false,
            which: 0,
        },
        &mut rope,
    );

    cursor.handle_input(
        &dioxus_html::on::KeyboardData {
            char_code: 'e'.into(),
            key: 'e'.to_string(),
            key_code: dioxus_html::KeyCode::E,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            locale: "".to_string(),
            location: 0,
            repeat: false,
            which: 0,
        },
        &mut rope,
    );

    cursor.handle_input(
        &dioxus_html::on::KeyboardData {
            char_code: 'l'.into(),
            key: 'l'.to_string(),
            key_code: dioxus_html::KeyCode::L,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            locale: "".to_string(),
            location: 0,
            repeat: false,
            which: 0,
        },
        &mut rope,
    );

    cursor.handle_input(
        &dioxus_html::on::KeyboardData {
            char_code: 'l'.into(),
            key: 'l'.to_string(),
            key_code: dioxus_html::KeyCode::L,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            locale: "".to_string(),
            location: 0,
            repeat: false,
            which: 0,
        },
        &mut rope,
    );

    cursor.handle_input(
        &dioxus_html::on::KeyboardData {
            char_code: 'o'.into(),
            key: 'o'.to_string(),
            key_code: dioxus_html::KeyCode::O,
            alt_key: false,
            ctrl_key: false,
            meta_key: false,
            shift_key: false,
            locale: "".to_string(),
            location: 0,
            repeat: false,
            which: 0,
        },
        &mut rope,
    );

    assert_eq!(rope.to_string(), "hello world\nhi");
}
