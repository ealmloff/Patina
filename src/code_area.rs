use crate::cursor::{Cursor, Pos};
use crate::span::Span;
use crate::utils::color_to_string;
use crate::{PS, THEME};
use dioxus::prelude::*;
use ropey::Rope;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style};

#[derive(Props, PartialEq)]
pub struct CodeAreaProps {
    initial_text: String,
}
pub fn CodeArea(cx: Scope<CodeAreaProps>) -> Element {
    let (scroll_y, set_scroll_y) = use_state(&cx, || 0.0);
    let rope = use_ref(&cx, || Rope::from_str(&cx.props.initial_text));
    // sorted array of cursor
    let cursors = use_ref(&cx, || {
        vec![
            Cursor::new(Pos::new(0, 3)),
            Cursor::new(Pos::new(0, 5)),
            Cursor::new(Pos::new(0, 7)),
        ]
    });
    let current_cursors = cursors.read().clone();
    let mut cursor_iter = current_cursors
        .iter()
        .map(|c| c.start.idx(&rope.read()))
        .peekable();

    let text = rope.read().clone();
    let num_lines = text.len_lines();
    let lines = text.lines();

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &THEME);
    let bg = &color_to_string(THEME.settings.background.unwrap());

    let mut text_pos = 0;

    let cursor_style = Style {
        foreground: Color::WHITE,
        background: THEME.settings.background.unwrap(),
        ..Default::default()
    };

    cx.render(rsx! {
        div{
            width: "100%",
            height: "100%",
            flex_direction: "column",
            align_items: "left",
            justify_content: "left",
            background_color: "{bg}",
            prevent_default: "onkeydown",
            onkeydown: |k| {
                let write = &mut rope.write();
                let mut row = 0;
                let mut new_rows = 0;
                let mut new_chars = 0;
                for c in cursors.write().iter_mut(){
                    // use web_sys::console;
                    let r = c.start.row();
                    // console::log_1(&format!("{c:?} {r}={row} {new_chars}, {new_rows}").into());
                    if new_rows != 0{
                        c.start.move_row(new_rows, write);
                    }
                    if r <= row{
                        if new_chars != 0{
                            c.start.move_col_raw(new_chars);
                        }
                    }
                    else{
                        row = r;
                        new_chars = 0;
                    }
                    // console::log_1(&format!("{c:?}").into());
                    let [dr, dc] = c.handle_input(&*k.data, write);
                    new_rows += dr;
                    new_chars += dc;
                }
            },
            tabindex: "0",
            // onwheel: move |w| set_scroll_y((scroll_y + w.data.delta_y.signum() as f32).max(0.0)),

            lines.enumerate().map(|(i, l)| {
                let cs: std::borrow::Cow<str> = l.into();
                let ranges = h.highlight(&cs, &PS);

                let mut ranges: Vec<_> = ranges.into_iter().map(|(s, t)|{
                    let final_text_pos = text_pos + t.len();
                    let mut tail = t;
                    let mut segments = Vec::new();
                    while let Some(idx) = cursor_iter.next_if(|idx|{
                        final_text_pos > *idx
                    }){
                        let (before, new_tail) = tail.split_at(idx - text_pos);
                        tail = new_tail;
                        segments.push((s, before));
                        segments.push((cursor_style, "|"));
                        text_pos += before.len();
                    }
                    text_pos += tail.len();
                    segments.push((s, tail.trim_end_matches('\n')));
                    segments.into_iter()
                }).flatten().filter(|(_, t)| t.len() > 0).collect();
                // if this is the last line add any unrendered cursors
                if i == num_lines - 1{
                    if let Some(_) = cursor_iter.next(){
                        ranges.push((cursor_style, "|"));
                    }
                }
                // force rendering of line
                if ranges.len() == 0{
                    ranges.push((Style {
                        background: THEME.settings.background.unwrap(),
                        ..Default::default()
                    }, " "))
                }
                cx.render(rsx! {
                    div{
                        width: "100%",
                        flex_direction: "row",
                        ranges.into_iter().map(|(s, t)|{
                            cx.render(rsx! {
                                Span{
                                    style: s,
                                    text: t.to_string()
                                }
                            })
                        })
                    }
                })
            }).skip(*scroll_y as usize)
            cursors.read().iter().map(|c|
                cx.render(rsx! {
                    p{
                        "{c:?}"
                    }
                })
            )
        }
    })
}
