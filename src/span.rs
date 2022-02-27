use crate::utils::color_to_string;
use dioxus::prelude::*;
use syntect::highlighting::{FontStyle, Style};

#[derive(Props, PartialEq)]
pub struct SpanProps {
    style: Style,
    text: String,
}
pub fn Span(cx: Scope<SpanProps>) -> Element {
    let text = &cx.props.text;

    let fg = color_to_string(cx.props.style.foreground);
    let bg = color_to_string(cx.props.style.background);
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
