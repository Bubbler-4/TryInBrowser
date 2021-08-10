#![allow(clippy::wildcard_imports)]

mod lang;
mod runner;
mod threading;

use indoc::indoc;
use seed::{prelude::*, *};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use threading::prelude::OUT_LIMIT;
use web_sys::window;

fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(Msg::Rendered);
    runner::init();
    let languages_list = lang::lang_name_list();
    let lang_part = url.next_hash_path_part();
    let languages_shown = lang_part.is_none();
    let lang = lang_part.map_or_else(|| languages_list[0].to_string(), b64_to_string);
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
    Model {
        spinner: 0,
        thread_state: NotReady,
        stdout: String::with_capacity(OUT_LIMIT),
        stderr: String::with_capacity(OUT_LIMIT + 100),
        lang,
        code,
        stdin,
        args,
        languages_shown,
        languages_list,
        url,
        dragging: false,
        code_selection: String::default(),
    }
}

fn b64_to_string(s: &str) -> String {
    match base64::decode(s[1..].as_bytes()) {
        Ok(vec) => String::from_utf8_lossy(&vec).to_string(),
        Err(_) => "<Failed to decode>".to_string(),
    }
}

#[derive(PartialEq, Eq)]
enum ThreadState {
    NotReady,
    Ready,
    Running,
}
use ThreadState::{NotReady, Ready, Running};

struct Model {
    spinner: usize,
    thread_state: ThreadState,
    stdout: String,
    stderr: String,
    lang: String,
    code: String,
    stdin: String,
    args: String,
    languages_shown: bool,
    languages_list: Vec<&'static str>,
    url: Url,
    dragging: bool,
    code_selection: String,
}

enum Msg {
    Rendered(RenderInfo),
    Stop,
    Run,
    LangSet(String),
    CodeUpdate(String),
    StdinUpdate(String),
    ArgsUpdate(String),
    LangListToggle,
    Linkify,
    Postify,
    CodeSelect(bool),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered(_) => {
            model.spinner += 1;
            update_state(model);
            orders.after_next_render(Msg::Rendered);
        }
        Msg::Run => {
            log!("Run clicked");
            runner::reset_all_flags();
            model.thread_state = Running;
            model.stdout.clear();
            model.stderr.clear();
            runner::run(&model.lang, &model.code, &model.stdin, &model.args);
        }
        Msg::Stop => {
            log!("Stop clicked");
            model.thread_state = Ready;
            model.stderr += &format!("\n\nElapsed time: {:.6} sec", runner::get_elapsed_time());
            model.stderr += "\naborted";
            runner::reset();
        }
        Msg::LangSet(s) => {
            log!("Language set to", &s);
            model.lang = s;
            log!("current lang is", &model.lang);
        }
        Msg::CodeUpdate(s) => model.code = s,
        Msg::StdinUpdate(s) => model.stdin = s,
        Msg::ArgsUpdate(s) => model.args = s,
        Msg::CodeSelect(is_start) => {
            if is_start {
                model.dragging = true;
            } else if model.dragging {
                model.dragging = false;
                model.code_selection = get_selection().unwrap_or_default();
            } else {
                model.code_selection = String::default();
            }
            log!("current code selection: ", model.code_selection);
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
            //model.running_text.clear();
            model.stdout.clear();
            model.stdout += "https://try-in-browser.netlify.app/#";
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
            //model.running_text.clear();
            model.stdout.clear();
            model.stdout += &model.url.to_string();
            let homepage = lang::get_homepage(&model.lang).unwrap_or("");
            model.stdout = format_post(
                &model.lang,
                &model.code,
                &model.stdin,
                &model.args,
                &model.code_selection,
                homepage,
                &model.url,
            );
            model.stderr.clear();
        }
    }
}

fn update_state(model: &mut Model) {
    if model.thread_state == NotReady && runner::poll_mt_init() {
        model.thread_state = Ready;
    }
    if model.thread_state == Running {
        let finished = runner::get_th_finished();
        let (out, err) = runner::take_result_from_thread();
        let stdout_overflown = model.stdout.len() + out.len() > OUT_LIMIT;
        let stderr_overflown = model.stderr.len() + err.len() > OUT_LIMIT;
        let overflown = stdout_overflown || stderr_overflown;
        if stdout_overflown {
            model
                .stdout
                .push_str(&out[..OUT_LIMIT - model.stdout.len()]);
        } else {
            model.stdout.push_str(&out);
        }
        if stderr_overflown {
            model
                .stderr
                .push_str(&err[..OUT_LIMIT - model.stderr.len()]);
        } else {
            model.stderr.push_str(&err);
        }
        if overflown {
            runner::reset();
        }
        let crashed = runner::get_th_crashed() || overflown;
        if crashed || finished {
            model.thread_state = Ready;
        }
        if crashed || finished {
            model.stderr += &format!("\n\nElapsed time: {:.6} sec", runner::get_elapsed_time());
        }
        if crashed {
            model.stderr += if overflown {
                "\noutput limit exceeded"
            } else {
                "\ninterpreter crashed"
            };
        } else if finished {
            model.stderr += "\nfinished";
        }
    }
}

fn format_post(
    lang: &str,
    code: &str,
    input: &str,
    args: &str,
    selection: &str,
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
    let display_code = if selection.is_empty() {
        code
    } else {
        selection
    };
    format!(
        indoc!(
            r#"
            # [{0}][tib-{0}], {1} byte{2}

            ```
            {3}
            ```

            [Try in browser!][tib-{4:016x}]
            [tib-{0}]: {5}
            [tib-{4:016x}]: https://try-in-browser.netlify.app/#{6}
            "#
        ),
        lang,
        display_code.len(),
        if display_code.len() == 1 { "" } else { "s" },
        display_code,
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

fn get_selection() -> Option<String> {
    let window = window();
    let selection = window.and_then(|w| w.get_selection().ok()).flatten();
    let s: Option<String> = selection.map(|s| s.to_string().into());
    s
}

fn view(model: &Model) -> Node<Msg> {
    div![
        IF!(cfg!(feature="ui_debug") => div![
            "UI health: ", ".".repeat(model.spinner / 10 % 10),
            br![], br![],
        ]),
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
        div![
            id!("langs"),
            model
                .languages_list
                .iter()
                .filter(|&x| model.languages_shown || x == &model.lang)
                .map(|s| {
                    let s_clone = (*s).to_string();
                    div![
                        C![
                            IF!(&model.lang == s => "active"),
                            IF!(&model.lang != s => "inactive"),
                            IF!(model.thread_state == Running => "disabled")
                        ],
                        s,
                        IF!(model.thread_state != Running => ev(Ev::Click, move |_| Msg::LangSet(s_clone)))
                    ]
                })
        ],
        br![],
        b!["Code"],
        textarea![
            id!("code"),
            attrs! {At::SpellCheck => false, At::Rows => rows(&model.code, 4), At::Cols => COLS, At::Value => model.code},
            input_ev(Ev::Input, Msg::CodeUpdate),
            ev(Ev::MouseDown, |_| Msg::CodeSelect(true))
        ],
        br![],
        b!["Stdin"],
        textarea![
            id!("stdin"),
            attrs! {At::SpellCheck => false, At::Rows => rows(&model.stdin, 4), At::Cols => COLS, At::Value => model.stdin},
            input_ev(Ev::Input, Msg::StdinUpdate)
        ],
        br![],
        b!["Arguments (enter -h and press Run for usage)"],
        textarea![
            id!("args"),
            attrs! {At::SpellCheck => false, At::Rows => rows(&model.args, 1), At::Cols => COLS, At::Value => model.args},
            input_ev(Ev::Input, Msg::ArgsUpdate)
        ],
        br![],
        button![
            id!("run"),
            attrs! { At::Disabled => (model.thread_state != Ready).as_at_value() },
            "Run",
            ev(Ev::Click, |_| Msg::Run)
        ],
        button![
            id!("stop"),
            attrs! { At::Disabled => (model.thread_state != Running).as_at_value() },
            "Stop",
            ev(Ev::Click, |_| Msg::Stop)
        ],
        br![],
        button![
            id!("linkify"),
            attrs! { At::Disabled => (model.thread_state != Ready).as_at_value() },
            "Linkify",
            ev(Ev::Click, |_| Msg::Linkify)
        ],
        button![
            id!("postify"),
            attrs! { At::Disabled => (model.thread_state != Ready).as_at_value() },
            "Postify",
            ev(Ev::MouseDown, |_| Msg::Postify)
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
        ev(Ev::MouseUp, |_| Msg::CodeSelect(false)),
    ]
}

const COLS: usize = 80;
fn rows(s: &str, min_rows: usize) -> usize {
    s.split('\n')
        .map(|l| (l.len().max(1) + COLS - 1) / COLS)
        .sum::<usize>()
        .max(min_rows)
}

#[wasm_bindgen]
pub fn start() {
    App::start("app", init, update, view);
}
