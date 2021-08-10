use js_sys::Date;
use seed::log;
use threading::prelude::*;
use threading::Thread;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use crate::threading;

struct RunnerState {
    mt: Option<WasmMt>,
    thread: Option<Thread>,
    th_init: bool,
    result: Option<(String, String)>,
    th_finished: bool,
    th_crashed: bool,
    start_time: f64,
}

static mut STATE: RunnerState = RunnerState {
    mt: None,
    thread: None,
    th_init: false,
    result: None,
    th_finished: false,
    th_crashed: false,
    start_time: 0.0,
};

fn set_mt(mt: WasmMt) {
    unsafe {
        STATE.mt = Some(mt);
    }
}

fn get_mt() -> Option<&'static WasmMt> {
    unsafe { STATE.mt.as_ref() }
}

fn reset_thread() {
    unsafe {
        STATE.thread = None;
    }
}

fn set_thread(th: Thread) {
    unsafe {
        STATE.thread = Some(th);
    }
}

fn get_thread() -> Option<&'static Thread> {
    unsafe { STATE.thread.as_ref() }
}

fn set_th_init(b: bool) {
    unsafe {
        STATE.th_init = b;
    }
}

fn get_th_init() -> bool {
    unsafe { STATE.th_init }
}

fn set_th_finished(b: bool) {
    unsafe {
        STATE.th_finished = b;
    }
}

pub fn get_th_finished() -> bool {
    unsafe {
        let ret = STATE.th_finished;
        STATE.th_finished = false;
        ret
    }
}

fn set_th_crashed(b: bool) {
    unsafe {
        STATE.th_crashed = b;
    }
}

pub fn get_th_crashed() -> bool {
    unsafe {
        let ret = STATE.th_crashed;
        STATE.th_crashed = false;
        ret
    }
}

pub fn reset_all_flags() {
    get_th_finished();
    get_th_crashed();
}

fn reset_result() {
    unsafe {
        if let Some((ref mut x, ref mut y)) = STATE.result {
            x.clear();
            y.clear();
        } else {
            STATE.result = Some((String::new(), String::new()));
        }
    }
}

fn set_result(s1: String, s2: String) {
    unsafe {
        STATE.result = Some((s1, s2));
    }
}

fn set_start_time() {
    unsafe {
        STATE.start_time = window()
            .and_then(|w| w.performance())
            .map_or(Date::now(), |p| p.now());
    }
}

pub fn get_elapsed_time() -> f64 {
    unsafe {
        let end_time = window()
            .and_then(|w| w.performance())
            .map_or(Date::now(), |p| p.now());
        (end_time - STATE.start_time) / 1000.0
    }
}

pub fn take_result_from_thread() -> (String, String) {
    unsafe {
        STATE.thread.as_mut().map_or_else(
            || (String::new(), String::new()),
            |th| {
                let s1 = th.stdout();
                let s2 = th.stderr();
                (s1, s2)
            },
        )
    }
}

pub fn init() {
    spawn_local(async {
        let pkg_js = "./pkg/package.js";
        let mt = WasmMt::new(pkg_js).and_init().await.unwrap();
        log!("success");
        set_mt(mt);
        log!("success 2");
    });
}

pub fn poll_mt_init() -> bool {
    if get_mt().is_some() && !get_th_init() {
        log!("mt init success");
        set_th_init(true);
    }
    get_th_init()
}

pub fn run(lang: &str, code: &str, stdin: &str, args: &str) {
    reset_result();
    let lang = lang.to_string();
    let code = code.to_string();
    let stdin = stdin.to_string();
    let args = args.to_string();
    spawn_local(async move {
        let mt = get_mt().unwrap();
        let th = mt.thread().and_init().await.unwrap();
        log!("reset success");
        set_thread(th);
        log!("reset success 2");
        let thread = get_thread().unwrap();
        let start_time: f64 = Date::now();
        set_start_time();
        let result = exec_lang!(thread, &lang, &code, &stdin, &args).await;
        let end_time: f64 = Date::now();
        let elapsed = (end_time - start_time) / 1000.0;
        log!(result, elapsed);
        if let Ok(_jsval) = result {
            log!("finished");
            set_th_finished(true);
        } else {
            set_result("".to_string(), "err found".to_string());
            set_th_crashed(true);
        }
    });
}

pub fn reset() {
    let thread = get_thread().expect("A thread should be active");
    thread.terminate();
    reset_thread();
}
