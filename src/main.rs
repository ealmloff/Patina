#[macro_use]
extern crate lazy_static;

use dioxus::prelude::*;
use ropey::Rope;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

mod cursor;

use cursor::Cursor;

lazy_static! {
    static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();
    static ref THEME: &'static Theme = &TS.themes["base16-ocean.dark"];
}

const DEMO_TEXT: &str = "fn app(cx: Scope) -> Element {
    let (count, set_count) = use_state(&cx, || 0);

    cx.render(rsx!(
        h1 { \"High-Five counter: {count}\" }
        button { onclick: move |_| set_count(count + 1), \"Up high!\" }
        button { onclick: move |_| set_count(count - 1), \"Down low!\" }
    ))
}";

fn color_to_str(c: Color) -> String {
    format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a as f32 / 255.0)
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    dioxus::web::launch(App);

    #[cfg(not(target_arch = "wasm32"))]
    if cfg!(feature = "term") {
        rink::launch(App);
    } else {
        dioxus::desktop::launch(App);
    }
}

#[derive(Props, PartialEq)]
struct SpanProps {
    style: Style,
    text: String,
}
fn Span(cx: Scope<SpanProps>) -> Element {
    let text = &cx.props.text;

    let fg = color_to_str(cx.props.style.foreground);
    let bg = color_to_str(cx.props.style.background);
    let text_decoration = if cx.props.style.font_style.contains(FontStyle::UNDERLINE) {
        "underline"
    } else {
        "none"
    };
    let font_weight = if cx.props.style.font_style.contains(FontStyle::BOLD) {
        "bold"
    } else {
        "none"
    };
    let font_style = if cx.props.style.font_style.contains(FontStyle::ITALIC) {
        "italic"
    } else {
        "none"
    };
    #[cfg(feature = "term")]
    {
        cx.render(rsx! {
            span{
                background_color: "{bg}",
                text_decoration: "{text_decoration}",
                font_weight: "{font_weight}",
                font_style: "{font_style}",
                color: "{fg}",
                "{text}"
            }
        })
    }
    #[cfg(not(feature = "term"))]
    {
        cx.render(rsx! {
            span{
                font_family: "monospace",
                white_space: "pre",
                background_color: "{bg}",
                text_decoration: "{text_decoration}",
                font_weight: "{font_weight}",
                font_style: "{font_style}",
                color: "{fg}",
                "{text}"
            }
        })
    }
}

#[derive(Props, PartialEq)]
struct TabProps {
    initial_text: String,
}
fn Tab(cx: Scope<TabProps>) -> Element {
    let (scroll_y, set_scroll_y) = use_state(&cx, || 0.0);
    let rope = use_ref(&cx, || Rope::from_str(&cx.props.initial_text));
    // sorted array of cursors
    let cursors = use_ref(&cx, || vec![Cursor::default()]);
    let current_cursors = cursors.read().clone();
    let mut cursor_iter = current_cursors
        .iter()
        .map(|c| c.idx(&rope.read()))
        .peekable();

    let text = rope.read().clone();
    let num_lines = text.len_lines();
    let lines = text.lines();

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &THEME);
    let bg = &color_to_str(THEME.settings.background.unwrap());

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
                for c in cursors.write().iter_mut(){
                    c.handle_input(&*k.data, write);
                }
            },
            tabindex: "0",
            // onwheel: move |w| set_scroll_y((scroll_y + w.data.delta_y.signum() as f32).max(0.0)),

            lines.enumerate().map(|(i, l)| {
                let cs: std::borrow::Cow<str>  = l.into();
                let ranges = h.highlight(&cs, &PS);

                let mut ranges: Vec<_> = ranges.into_iter().map(|(s, t)|{
                    let len = t.len();
                    let mut tail = t;
                    let mut segments = Vec::new();
                    while let Some(idx) = cursor_iter.next_if(|idx|{
                        text_pos + len > *idx
                    }){
                        let (before, new_tail) = tail.split_at(idx - text_pos);
                        tail = new_tail;
                        segments.push((s, before));
                        segments.push((cursor_style, "|"));
                    }
                    text_pos += len;
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
        }
    })
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div{
            width: "100%",
            height: "100%",
            Tab{
                initial_text: DEMO_TEXT.to_string()
            }
        }
    })
}
