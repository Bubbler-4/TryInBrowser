#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_possible_truncation)]

mod languages;
use languages::{LangContext, LANGS};

use seed::{prelude::*, *};

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    Model {
        lang: LANGS[0].to_string(),
        code: String::default(),
        stdin: String::default(),
        args: String::default(),
        stdout: String::default(),
        stderr: String::default(),
        is_running: false,
        languages_shown: true,
        running_text: String::default(),
        context: None,
    }
}

struct Model {
    lang: String,
    code: String,
    stdin: String,
    args: String,
    stdout: String,
    stderr: String,
    is_running: bool,
    languages_shown: bool,
    running_text: String,
    context: Option<LangContext>,
}

enum Msg {
    Rendered(RenderInfo),
    LangSet(String),
    CodeUpdate(String),
    StdinUpdate(String),
    ArgsUpdate(String),
    Run,
    Stop,
    LangListToggle,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(ri) => {
            let delta = ri.timestamp_delta.unwrap_or_default();
            if let (true, Some(ctx)) = (model.is_running, model.context.as_mut()) {
                let (out, err, running) = ctx.step_adaptive(delta);
                model.stdout += &out;
                model.stderr += &err;
                model.is_running = running;
                if running {
                    model.running_text += ".";
                    if model.running_text.len() > 15 {
                        model.running_text.truncate(8);
                    }
                } else {
                    model.running_text = "Finished".to_string();
                }
            }
            orders.after_next_render(Msg::Rendered);
        }
        Msg::LangSet(s) => {
            log!("Language set to", &s);
            model.lang = s;
            log!("current lang is", &model.lang);
        }
        Msg::CodeUpdate(s) => model.code = s,
        Msg::StdinUpdate(s) => model.stdin = s,
        Msg::ArgsUpdate(s) => model.args = s,
        Msg::Run => {
            log!("Run clicked");
            model.is_running = true;
            model.stdout.clear();
            model.stderr.clear();
            model.running_text = "Running.".to_string();
            model.context = Some(LangContext::init(
                &model.lang,
                &model.code,
                &model.stdin,
                &model.args,
            ));
        }
        Msg::Stop => {
            log!("Stop clicked");
            model.is_running = false;
            model.running_text = "Stopped.".to_string();
        }
        Msg::LangListToggle => {
            model.languages_shown = !model.languages_shown;
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        id!("main"),
        b!["Languages"],
        span![
            if model.languages_shown {
                "(Hide)"
            } else {
                "(Show)"
            },
            ev(Ev::Click, |_| Msg::LangListToggle)
        ],
        br![],
        IF!(model.languages_shown => div![
            id!("langs"),
            LANGS.iter().map(|s| {
                div![
                    C![IF!(&model.lang == s => "active"), IF!(&model.lang != s => "inactive"), IF!(model.is_running => "disabled")],
                    s,
                    IF!(!model.is_running => ev(Ev::Click, move |_| Msg::LangSet((*s).to_string())))
                ]
            })
        ]),
        br![],
        b!["Code"],
        textarea![
            id!("code"),
            attrs! {At::Rows => rows(&model.code, 4), At::Cols => COLS},
            input_ev(Ev::Input, Msg::CodeUpdate)
        ],
        br![],
        b!["Stdin"],
        textarea![
            id!("stdin"),
            attrs! {At::Rows => rows(&model.stdin, 4), At::Cols => COLS},
            input_ev(Ev::Input, Msg::StdinUpdate)
        ],
        br![],
        b!["Arguments (enter -h and press Run for usage)"],
        textarea![
            id!("args"),
            attrs! {At::Rows => rows(&model.args, 1), At::Cols => COLS},
            input_ev(Ev::Input, Msg::ArgsUpdate)
        ],
        br![],
        button![
            id!("run"),
            attrs! {At::Disabled => model.is_running.as_at_value()},
            "Run",
            ev(Ev::Click, |_| Msg::Run)
        ],
        button![
            id!("stop"),
            attrs! {At::Disabled => (!model.is_running).as_at_value()},
            "Stop",
            ev(Ev::Click, |_| Msg::Stop)
        ],
        &model.running_text,
        br![],
        br![],
        b!["Output"],
        textarea![
            id!("stdout"),
            attrs! {At::Rows => rows(&model.stdout, 1), At::Cols => COLS, At::Value => model.stdout},
        ],
        br![],
        b!["Error"],
        textarea![
            id!("stderr"),
            attrs! {At::Rows => rows(&model.stderr, 1), At::Cols => COLS, At::Value => model.stderr},
        ],
    ]
}

const COLS: usize = 80;
fn rows(s: &str, min_rows: usize) -> usize {
    s.split('\n')
        .map(|l| (l.len().max(1) + COLS - 1) / COLS)
        .sum::<usize>()
        .max(min_rows)
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
