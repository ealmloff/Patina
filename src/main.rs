#[macro_use]
extern crate lazy_static;

use dioxus::prelude::*;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::code_area::CodeArea;

mod code_area;
mod cursor;
mod cursors;
mod span;
mod utils;

lazy_static! {
    static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref TS: ThemeSet = ThemeSet::load_defaults();
    static ref THEME: &'static Theme = &TS.themes["base16-ocean.dark"];
}

const DEMO_TEXT: &str = r"// alt-move to spawn cursor
// ctrl-move to move word
// shift-move to select
// ___       __   _______   ___       ________  ________  _____ ______   _______      
// |\  \     |\  \|\  ___ \ |\  \     |\   ____\|\   __  \|\   _ \  _   \|\  ___ \     
// \ \  \    \ \  \ \   __/|\ \  \    \ \  \___|\ \  \|\  \ \  \\\__\ \  \ \   __/|    
//  \ \  \  __\ \  \ \  \_|/_\ \  \    \ \  \    \ \  \\\  \ \  \\|__| \  \ \  \_|/__  
//   \ \  \|\__\_\  \ \  \_|\ \ \  \____\ \  \____\ \  \\\  \ \  \    \ \  \ \  \_|\ \ 
//    \ \____________\ \_______\ \_______\ \_______\ \_______\ \__\    \ \__\ \_______\
//     \|____________|\|_______|\|_______|\|_______|\|_______|\|__|     \|__|\|_______|";
// const DEMO_TEXT: &str = "1234567890
// 1234567890
// 1234567890";

fn main() {
    #[cfg(target_arch = "wasm32")]
    dioxus::web::launch(App);

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "term")]
        rink::launch(App);

        #[cfg(not(feature = "term"))]
        dioxus::desktop::launch(App);
    }
}

#[derive(Props, PartialEq)]
struct TabProps {
    initial_text: String,
}
fn Tab(cx: Scope<TabProps>) -> Element {
    cx.render(rsx! {
        CodeArea{
            initial_text: cx.props.initial_text.clone()
        }
    })
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div{
            width: "100%",
            height: "100%",
            position: "absolute",
            display: "flex",
            flex_direction: "row",
            Tab{
                initial_text: DEMO_TEXT.to_string()
            }
        }
    })
}
