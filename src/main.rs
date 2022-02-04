use dioxus::prelude::*;
use ide::{
    Analysis, AnalysisHost, Change, CrateGraph, FileId, Highlight, HlRange, HlTag, SourceRoot,
};
use ra_ap_base_db::{CrateOrigin, Env};
use std::sync::Arc;
use vfs::file_set::FileSet;
use vfs::VfsPath;
use std::cell::Cell;

mod cursor;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // dioxus::desktop::launch(App);

        rink::launch(App);
    }
    #[cfg(target_arch = "wasm32")]
    {
        dioxus::web::launch(App);
    }
}

fn get_color(hl: Highlight) -> [u8; 3] {
    let color = match hl.tag {
        HlTag::Symbol(k) => [255, 255, 0],
        // HlTag::AttributeBracket => {
        //
        // },
        HlTag::BoolLiteral => [200, 200, 255],
        // HlTag::BuiltinType => {
        //
        // },
        // HlTag::ByteLiteral => {
        //
        // },
        // HlTag::CharLiteral => {
        //
        // },
        HlTag::Comment => [75, 75, 75],
        // HlTag::EscapeSequence => {
        //
        // },
        // HlTag::FormatSpecifier => {
        //
        // },
        HlTag::Keyword => [255, 255, 100],
        // HlTag::NumericLiteral => {
        //
        // },
        // HlTag::Operator(HlOperator) => {
        //
        // },
        HlTag::Punctuation(HlPunct) => [150, 150, 150],
        // HlTag::StringLiteral => {
        //
        // },
        // HlTag::UnresolvedReference => {
        //
        // },
        // HlTag::None => {
        //
        // },
        _ => [255, 255, 255],
    };
    color
}

#[derive(Props, PartialEq)]
struct ColoredSpanProps {
    text: String,
    highlight: Highlight,
}
fn ColoredSpan(cx: Scope<ColoredSpanProps>) -> Element {
    // let a = cx.props.host.analysis();
    // if let Ok(hl_ranges) = a.highlight(cx.props.f_id) {
    //     println!("{hl_ranges:?}");
    // }
    let color: String = get_color(cx.props.highlight)
        .iter()
        .map(|c| format!("{c:02X?}"))
        .collect();
    let color = &("#".to_string() + &color);
    // println!("{color}", );
    cx.render(rsx! {
        cx.props.text.split_inclusive('\n').map(|l| rsx!{
            span{
                // style: "color: {color}; white-space: pre;",
                "{l}"
            }
            // l.ends_with('\n').then(|| rsx!{ br{} })
        })
    })
}

#[derive(Props)]
struct TabProps<'a> {
    children: Element<'a>,
}
fn Tab<'a>(cx: Scope<'a, TabProps<'a>>) -> Element<'a> {
    // let a = cx.props.host.analysis();
    // if let Ok(hl_ranges) = a.highlight(cx.props.f_id) {
    //     println!("{hl_ranges:?}");
    // }
    cx.render(rsx! {
        div{
            &cx.props.children
        }
    })
}

fn App(cx: Scope) -> Element {
    let current_file = FileId(0);
    let mut host = cx.use_hook(|_| {
        let root_file = FileId(0);
        let mut host = AnalysisHost::new(Some(1000));
        // let mut graph = CrateGraph::default();
        // let crate_id = graph.add_crate_root(
        //     root_file,
        //     ide::Edition::Edition2021,
        //     None,
        //     None,
        //     ra_ap_cfg::CfgOptions::default(),
        //     ra_ap_cfg::CfgOptions::default(),
        //     Env::default(),
        //     Vec::new(),
        //     CrateOrigin::Unknown,
        // );

        let mut change = Change::new();
        let mut set = FileSet::default();
        set.insert(
            root_file,
            VfsPath::new_virtual_path("/main.rs".to_string()),
        );
        change.set_roots(vec![SourceRoot::new_local(set)]);
        let initial_code = "if true {
    // print stuff
    println!(\"{:?}\", );
}
else {
    let no = true;
    if no {
        panic!();
    }
}"
        .to_string();
        change.change_file(root_file, Some(Arc::new(initial_code)));
        // change.set_crate_graph(graph);
        host.apply_change(change);
        host
    });
    let analysis = host.analysis();
    let current_text = analysis.file_text(current_file).unwrap();
    cx.render(rsx!{
        div{
            width: "100%",
            height: "10px",
            background_color: "#888888",
            justify_content: "center",
            align_items: "center",
            // class: "main",
            // tabindex: "1",
            // prevent_default: "onkeypress",
            // onkeypress: move |data| {
            //     let mut change = Change::new();
            //     let key = &data.data.key;
            //     println!("{key:?}");
            //     change.change_file(current_file, Some(Arc::new(key.clone())));
            //     host.apply_change(change);
            //     cx.needs_update();
            // },
            // rink::InputHandler{},
            // h1{
            //     color: "#FF0000",
            //     "{current_text}"
            // }
            "{current_text}"
            // Tab{
            //     analysis.highlight(current_file).unwrap().iter().map(|hl|{
            //         let text = (&current_text[hl.range.start().into()..hl.range.end().into()]).to_string();
            //         rsx!{
            //             ColoredSpan{
            //                 text: text,
            //                 highlight: hl.highlight,
            //             }
            //         }
            //     })
            // }
        }
    })
}
