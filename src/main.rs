#[macro_use]
extern crate lazy_static;
use std::borrow::{Borrow, BorrowMut};

use dioxus::prelude::*;
use ropey::Rope;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

mod cursor;

use cursor::Cursor;

lazy_static! {
    static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();
    static ref THEME: &'static Theme = &TS.themes["base16-ocean.dark"];
}

fn color_to_str(c: Color) -> String {
    format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a as f32 / 255.0)
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        dioxus::web::launch(App);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "term")]
        rink::launch(App);

        #[cfg(not(feature = "term"))]
        dioxus::desktop::launch(App);
    }
}

#[derive(Props, PartialEq)]
struct SpanProps {
    style: Style,
    text: String,
}
fn Span(cx: Scope<SpanProps>) -> Element {
    // sorry tab people
    let text = cx.props.text.trim_end_matches('\n').replace('\t', "    ");
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
    cx.render(rsx! {
        span{
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

#[derive(Props, PartialEq)]
struct TabProps {
    text: Rope,
}
fn Tab(cx: Scope<TabProps>) -> Element {
    let (scroll_y, set_scroll_y) = use_state(&cx, || 0.0);
    let cursors = use_ref(&cx, || vec![Cursor::default()]);

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &THEME);
    let lines = cx.props.text.lines();
    let bg = &color_to_str(THEME.settings.background.unwrap());
    // use web_sys::console;

    // console::log_1(&format!("{scroll_y}").into());
    cx.render(rsx! {
        div{
            width: "100%",
            height: "100%",
            flex_direction: "column",
            align_items: "left",
            justify_content: "left",
            background_color: "{bg}",
            onkeypress: |k| for c in cursors.write().iter_mut(){
                c.handle_input(&*k.data);
            },
            onwheel: move |w| set_scroll_y((scroll_y + w.data.delta_y.signum() as f32).max(0.0)),

            lines.map(|l| {
                let cs: std::borrow::Cow<str>  = l.into();
                let ranges = h.highlight(&cs, &PS);
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

const CSS: &str = "html, body {
    margin: 0;
    padding: 0;
    width: 100%;
    overflow_y: hidden;
}";

fn App(cx: Scope) -> Element {
    #[cfg(feature = "term")]
    {
        cx.render(rsx! {
            div{
                width: "100%",
                height: "100%",
                Tab{
                    text: Rope::from_str("pub struct Wow {\n\thi: u64\n}\nfn blah() -> u64 {\n\n}")
                }
            }
        })
    }

    #[cfg(not(feature = "term"))]
    {
        cx.render(rsx! {
            style{
                "{CSS}"
            },
            div{
                width: "100%",
                height: "100%",
                Tab{
                    text: Rope::from_str("pub struct Wow {\n\thi: u64\n}\nfn blah() -> u64 {\n\n}")
                }
            }
        })
    }
}
