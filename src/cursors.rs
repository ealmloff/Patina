use dioxus::events::KeyboardData;
use dioxus_html::KeyCode;
use ropey::Rope;

use crate::cursor::{Cursor, Pos};
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum SelectionMarkerType {
    End,
    Start,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionMarker<'a> {
    pub pos: &'a Pos,
    pub marker_type: SelectionMarkerType,
    id: usize,
}

impl<'a> SelectionMarker<'a> {
    fn new(pos: &'a Pos, id: usize, selection_type: SelectionMarkerType) -> Self {
        Self {
            pos,
            id,
            marker_type: selection_type,
        }
    }

    fn is_matching(&self, other: &'a SelectionMarker) -> bool {
        self.marker_type != other.marker_type && self.id == other.id
    }
}

impl<'a> Ord for SelectionMarker<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos
            .cmp(&other.pos)
            .then(self.marker_type.cmp(&other.marker_type))
    }
}

impl<'a> PartialOrd for SelectionMarker<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursors(pub Vec<Cursor>);

impl Cursors {
    pub fn process_input(&mut self, keyboard_data: &KeyboardData, rope: &mut Rope) {
        if keyboard_data.alt_key {
            let direction = match &keyboard_data.key_code {
                KeyCode::UpArrow => [0, -1],
                KeyCode::DownArrow => [0, 1],
                KeyCode::RightArrow => [1, 0],
                KeyCode::LeftArrow => [-1, 0],
                _ => return,
            };
            let mut new = Vec::new();
            for c in &self.0 {
                let mut new_cursor = c.clone();

                new_cursor.start.move_col(direction[0], rope);
                new_cursor.start.move_row(direction[1], rope);
                if let Some(e) = &mut new_cursor.end {
                    e.move_col(direction[0], rope);
                    e.move_row(direction[1], rope);
                }
                new.push(new_cursor);
            }
            self.0.append(&mut new);
        } else {
            let mut row = 0;
            let mut new_rows = 0;
            let mut new_chars = 0;
            for c in self.0.iter_mut() {
                let r = c.start.row();
                if new_rows != 0 {
                    c.start.move_row(new_rows, rope);
                    if let Some(e) = &mut c.end {
                        e.move_row(new_rows, rope);
                    }
                }
                if r <= row {
                    if new_chars != 0 {
                        c.start.move_col_raw(new_chars);
                        if let Some(e) = &mut c.end {
                            e.move_col_raw(new_chars);
                        }
                    }
                } else {
                    row = r;
                    new_chars = 0;
                }
                let [dc, dr] = c.handle_input(&keyboard_data, rope);
                new_rows += dr;
                new_chars += dc;
            }
        }

        self.remove_overlaping();
    }

    fn remove_overlaping(&mut self) {
        let mut new: Vec<Cursor> = Vec::new();
        let mut open = Vec::new();
        let mut cursor_first = None;
        for s in self.sorted() {
            match &cursor_first {
                None => {
                    open = vec![s.clone()];
                    cursor_first = Some(s);
                }
                Some(first) => {
                    let mut some_matched = false;
                    open.retain(|other| {
                        let matched = s.is_matching(other);
                        some_matched |= matched;
                        !matched
                    });
                    if !some_matched {
                        open.push(s)
                    } else {
                        if open.is_empty() {
                            match first.marker_type {
                                SelectionMarkerType::Start => new.push(Cursor {
                                    start: first.pos.clone(),
                                    end: (s.pos != first.pos).then(|| s.pos.clone()),
                                }),
                                SelectionMarkerType::End => new.push(if s.pos == first.pos {
                                    Cursor {
                                        start: first.pos.clone(),
                                        end: None,
                                    }
                                } else {
                                    Cursor {
                                        start: s.pos.clone(),
                                        end: Some(first.pos.clone()),
                                    }
                                }),
                            }
                            cursor_first = None;
                        }
                    }
                }
            }
        }
        self.0 = new;
    }

    pub fn sorted(&self) -> Vec<SelectionMarker> {
        let mut v: Vec<_> = self
            .0
            .iter()
            .enumerate()
            .flat_map(|(i, c)| {
                if let Some(e) = &c.end {
                    vec![
                        SelectionMarker::new(&c.start, i, SelectionMarkerType::Start),
                        SelectionMarker::new(e, i, SelectionMarkerType::End),
                    ]
                    .into_iter()
                } else {
                    vec![
                        SelectionMarker::new(&c.start, i, SelectionMarkerType::Start),
                        SelectionMarker::new(&c.start, i, SelectionMarkerType::End),
                    ]
                    .into_iter()
                }
            })
            .collect();
        v.sort_unstable();
        v
    }
}

impl Default for Cursors {
    fn default() -> Self {
        Cursors(vec![Cursor::default()])
    }
}

#[test]
fn remove_overlaping_cursors() {
    let mut cursors = Cursors(vec![
        Cursor::new(Pos::new(0, 0), Pos::new(0, 1)),
        Cursor::from_start(Pos::new(1, 0)),
    ]);

    cursors.remove_overlaping();

    assert_eq!(
        cursors,
        Cursors(vec![Cursor::new(Pos::new(0, 0), Pos::new(0, 1))])
    );

    let mut cursors = Cursors(vec![
        Cursor::new(Pos::new(0, 1), Pos::new(0, 0)),
        Cursor::from_start(Pos::new(1, 0)),
    ]);

    cursors.remove_overlaping();

    assert_eq!(
        cursors,
        Cursors(vec![Cursor::new(Pos::new(0, 1), Pos::new(0, 0))])
    );

    let mut cursors = Cursors(vec![
        Cursor::new(Pos::new(0, 0), Pos::new(0, 1)),
        Cursor::new(Pos::new(1, 0), Pos::new(0, 2)),
    ]);

    cursors.remove_overlaping();

    assert_eq!(
        cursors,
        Cursors(vec![Cursor::new(Pos::new(0, 0), Pos::new(0, 2)),])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(0, 0)),
        Cursor::from_start(Pos::new(0, 0)),
    ]);

    cursors.remove_overlaping();

    assert_eq!(cursors, Cursors(vec![Cursor::from_start(Pos::new(0, 0))]));

    let mut cursors = Cursors(vec![
        Cursor::new(Pos::new(0, 0), Pos::new(0, 1)),
        Cursor::new(Pos::new(1, 1), Pos::new(0, 2)),
    ]);

    cursors.remove_overlaping();

    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::new(Pos::new(0, 0), Pos::new(0, 1)),
            Cursor::new(Pos::new(1, 1), Pos::new(0, 2)),
        ])
    );
}

#[test]
fn multicursor_input() {
    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(1, 0)),
        Cursor::from_start(Pos::new(3, 0)),
    ]);
    // h|el|lo world
    let mut rope = Rope::from_str("hello world");
    let key = &KeyboardData {
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
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "hoelolo world");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(2, 0)),
            Cursor::from_start(Pos::new(5, 0))
        ])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(1, 0)),
        Cursor::from_start(Pos::new(3, 0)),
    ]);
    // h|el|lo world
    let mut rope = Rope::from_str("hello world");
    let key = &KeyboardData {
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
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "elo world");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(0, 0)),
            Cursor::from_start(Pos::new(1, 0))
        ])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(1, 0)),
        Cursor::from_start(Pos::new(3, 0)),
    ]);
    // h|el|lo world
    let mut rope = Rope::from_str("hello world");
    let key = &KeyboardData {
        char_code: 0,
        key: "".to_string(),
        key_code: dioxus_html::KeyCode::Enter,
        alt_key: false,
        ctrl_key: false,
        meta_key: false,
        shift_key: false,
        locale: "".to_string(),
        location: 0,
        repeat: false,
        which: 0,
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "h\nel\nlo world");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(0, 1)),
            Cursor::from_start(Pos::new(0, 2))
        ])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(5, 0)),
        Cursor::from_start(Pos::new(5, 1)),
    ]);
    // hello|
    // world|
    let mut rope = Rope::from_str("hello\nworld");
    let key = &KeyboardData {
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
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "helloo\nworldo");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(6, 0)),
            Cursor::from_start(Pos::new(6, 1))
        ])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(5, 0)),
        Cursor::from_start(Pos::new(5, 1)),
    ]);
    // hello|
    // world|
    let mut rope = Rope::from_str("hello\nworld");
    let key = &KeyboardData {
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
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "hell\nworl");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(4, 0)),
            Cursor::from_start(Pos::new(4, 1))
        ])
    );

    let mut cursors = Cursors(vec![
        Cursor::from_start(Pos::new(0, 0)),
        Cursor::from_start(Pos::new(0, 1)),
    ]);
    // |hello
    // |world
    let mut rope = Rope::from_str("hello\nworld");
    let key = &KeyboardData {
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
    };

    cursors.process_input(&key, &mut rope);

    assert_eq!(rope.to_string(), "helloworld");
    assert_eq!(
        cursors,
        Cursors(vec![
            Cursor::from_start(Pos::new(0, 0)),
            Cursor::from_start(Pos::new(5, 0))
        ])
    );
}
