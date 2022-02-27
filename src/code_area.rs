use crate::cursor::{Cursor, Pos};
use crate::cursors::{Cursors, SelectionMarkerType};
use crate::span::Span;
use crate::utils::color_to_string;
use crate::{PS, THEME};
use dioxus::prelude::*;
use ropey::Rope;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style};

fn highlight_mod(mut s: Style) -> Style {
    s.background.r += 40;
    s.background.g += 40;
    s.background.b += 40;
    s
}

#[derive(Props, PartialEq)]
pub struct CodeAreaProps {
    initial_text: String,
}
pub fn CodeArea(cx: Scope<CodeAreaProps>) -> Element {
    let (scroll_y, set_scroll_y) = use_state(&cx, || 0.0);
    let rope = use_ref(&cx, || Rope::from_str(&cx.props.initial_text));
    let cursors = use_ref(&cx, || Cursors::default());

    let text = rope.read().clone();
    let num_lines = text.len_lines();
    let lines = text.clone();
    let lines = lines.lines();

    let current_cursors = cursors.read().clone();
    let cursor_sections = current_cursors.sorted();
    let mut cursor_sections_iter = cursor_sections
        .into_iter()
        .map(|section| (section.pos.idx(&text), section.marker_type))
        .peekable();

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &THEME);
    let bg = &color_to_string(THEME.settings.background.unwrap());

    let mut text_pos = 0;
    let mut highlighted = false;

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
            tabindex: "0",
            display: "flex",
            box_sizing: "border-box",

            prevent_default: "onkeydown",
            onkeydown: |k| {
                cursors.write().process_input(&*k, &mut rope.write())
            },
            // onwheel: move |w| set_scroll_y((scroll_y + w.data.delta_y.signum() as f32).max(0.0)),

            lines.enumerate().map(|(i, l)| {
                let cs: std::borrow::Cow<str> = l.into();
                let ranges = h.highlight(&cs, &PS);

                let mut ranges: Vec<_> = ranges.into_iter().map(|(text_style, t)|{
                    let final_text_pos = text_pos + t.len();
                    let mut tail = t;
                    let mut segments = Vec::new();
                    while let Some((idx, marker_type)) = cursor_sections_iter.next_if(|(idx, _)|{
                        final_text_pos > *idx
                    }){
                        let (before, new_tail) = tail.split_at(idx - text_pos);
                        text_pos += before.len();
                        tail = new_tail;
                        segments.push((if highlighted{highlight_mod(text_style)}else{text_style}, before));
                        highlighted = !highlighted;
                        if marker_type == SelectionMarkerType::End{
                            segments.push((cursor_style, "|"));
                        }
                    }
                    text_pos += tail.len();
                    segments.push((if highlighted{highlight_mod(text_style)}else{text_style}, tail.trim_end_matches('\n')));
                    segments.into_iter()
                }).flatten().filter(|(_, t)| t.len() > 0).collect();
                // if this is the last line add any unrendered cursors
                if i == num_lines - 1{
                    if cursor_sections_iter.find(|(_, marker_type)| *marker_type == SelectionMarkerType::Start).is_some(){
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
            cursors.read().0.iter().map(|c|
                cx.render(rsx! {
                    p{
                        "{c:?}"
                    }
                })
            )
        }
    })
}
