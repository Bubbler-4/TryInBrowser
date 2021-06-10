#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_possible_truncation)]

mod languages;
use languages::{LangContext, LANGS};

use seed::{prelude::*, *};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    let lang = url
        .next_hash_path_part()
        .map_or_else(|| LANGS[0].to_string(), b64_to_string);
    log!(lang);
    let code = url
        .next_hash_path_part()
        .map_or_else(|| "".to_string(), b64_to_string);
    log!(code);
    let stdin = url
        .next_hash_path_part()
        .map_or_else(|| "".to_string(), b64_to_string);
    log!(stdin);
    let args = url
        .next_hash_path_part()
        .map_or_else(|| "".to_string(), b64_to_string);
    log!(args);
    let context = LangContext::init(&lang, &code, &stdin, &args);
    Model {
        url,
        lang,
        code,
        stdin,
        args,
        stdout: "".to_string(),
        stderr: "".to_string(),
        is_running: false,
        languages_shown: true,
        running_text: String::default(),
        context,
    }
}

fn b64_to_string(s: &str) -> String {
    match base64::decode(s[1..].as_bytes()) {
        Ok(vec) => String::from_utf8_lossy(&vec).to_string(),
        Err(_) => "<Failed to decode>".to_string(),
    }
}

struct Model {
    url: Url,
    lang: String,
    code: String,
    stdin: String,
    args: String,
    stdout: String,
    stderr: String,
    is_running: bool,
    languages_shown: bool,
    running_text: String,
    context: LangContext,
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
    Linkify,
    Postify,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(ri) => {
            let delta = ri.timestamp_delta.unwrap_or_default();
            if model.is_running {
                let ctx = &mut model.context;
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
                    model.running_text = "Finished.".to_string();
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
            if &model.args == "-h" {
                model.is_running = false;
                model.context = LangContext::init(&model.lang, "", "", "");
                model.stdout.clear();
                model.stdout += model.context.help();
                model.stderr.clear();
                model.running_text.clear();
            } else {
                model.is_running = true;
                model.stdout.clear();
                model.stderr.clear();
                model.running_text = "Running.".to_string();
                model.context =
                    LangContext::init(&model.lang, &model.code, &model.stdin, &model.args);
            }
        }
        Msg::Stop => {
            log!("Stop clicked");
            model.is_running = false;
            model.running_text = "Stopped.".to_string();
        }
        Msg::LangListToggle => {
            model.languages_shown = !model.languages_shown;
        }
        Msg::Linkify => {
            model.url = update_url(
                model.url.clone(),
                &model.lang,
                &model.code,
                &model.stdin,
                &model.args,
            );
            model.running_text.clear();
            model.stdout.clear();
            model.stdout += "https://bubbler-4.github.io/TryInBrowser/#";
            model.stdout += model.url.hash().unwrap_or(&"".to_string());
            model.stderr.clear();
        }
        Msg::Postify => {
            model.url = update_url(
                model.url.clone(),
                &model.lang,
                &model.code,
                &model.stdin,
                &model.args,
            );
            model.running_text.clear();
            model.stdout.clear();
            model.stdout += &model.url.to_string();
            model.stdout = format_post(
                &model.lang,
                &model.code,
                &model.stdin,
                &model.args,
                model.context.homepage(),
                &model.url,
            );
            model.stderr.clear();
        }
    }
}

fn format_post(
    lang: &str,
    code: &str,
    input: &str,
    args: &str,
    lang_link: &str,
    url: &Url,
) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(lang.as_bytes());
    hasher.write_u8(0);
    hasher.write(code.as_bytes());
    hasher.write_u8(0);
    hasher.write(input.as_bytes());
    hasher.write_u8(0);
    hasher.write(args.as_bytes());
    let hash = hasher.finish();
    format!(
        r#"# [{0}][tib-{0}], {1} bytes

```
{2}
```

[Try in browser!][tib-{3:016x}]

[tib-{0}]: {4}
[tib-{3:016x}]: https://bubbler-4.github.io/TryInBrowser/#{5}"#,
        lang,
        code.len(),
        code,
        hash,
        lang_link,
        url.hash().unwrap_or(&"".to_string())
    )
}

fn update_url(url: Url, lang: &str, code: &str, input: &str, args: &str) -> Url {
    let lang = "@".to_string() + &base64::encode(lang);
    let code = "@".to_string() + &base64::encode(code);
    let input = "@".to_string() + &base64::encode(input);
    let args = "@".to_string() + &base64::encode(args);
    let url = url.set_hash_path(&[lang, code, input, args]);
    url.go_and_replace();
    url
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
            attrs! {At::Rows => rows(&model.code, 4), At::Cols => COLS, At::Value => model.code},
            input_ev(Ev::Input, Msg::CodeUpdate)
        ],
        br![],
        b!["Stdin"],
        textarea![
            id!("stdin"),
            attrs! {At::Rows => rows(&model.stdin, 4), At::Cols => COLS, At::Value => model.stdin},
            input_ev(Ev::Input, Msg::StdinUpdate)
        ],
        br![],
        b!["Arguments (enter -h and press Run for usage)"],
        textarea![
            id!("args"),
            attrs! {At::Rows => rows(&model.args, 1), At::Cols => COLS, At::Value => model.args},
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
        button![
            id!("linkify"),
            attrs! {At::Disabled => model.is_running.as_at_value()},
            "Linkify",
            ev(Ev::Click, |_| Msg::Linkify)
        ],
        button![
            id!("postify"),
            attrs! {At::Disabled => model.is_running.as_at_value()},
            "Postify",
            ev(Ev::Click, |_| Msg::Postify)
        ],
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
