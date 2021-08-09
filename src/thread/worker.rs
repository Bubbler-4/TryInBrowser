use super::atw::ThreadWorker as AtwThreadWorker;
use super::decode_task_msg;
use super::job;
use super::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WorkerGlobalScope};

#[allow(dead_code)]
#[wasm_bindgen]
pub fn wmt_bootstrap(wgs: WorkerGlobalScope) -> _Worker {
    let worker = _Worker::new(wgs);
    worker
        .atw_thw
        .send_response(&JsValue::from("bootstrap COMPLETE"), None, false);

    worker
}

#[wasm_bindgen]
pub struct _Worker {
    atw_thw: Rc<AtwThreadWorker>,
    // Store closures instead of calling `.forget()` which leaks
    _on_message: Box<Closure<dyn FnMut(MessageEvent)>>,
}

impl _Worker {
    fn new(wgs: WorkerGlobalScope) -> Self {
        let atw_thw = Rc::new(AtwThreadWorker::new(wgs));

        let on_message = Self::create_onmessage(atw_thw.clone());
        atw_thw.set_callback_of("onmessage", on_message.as_ref());

        Self {
            atw_thw,
            _on_message: Box::new(on_message),
        }
    }

    fn create_onmessage(atw_thw: Rc<AtwThreadWorker>) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |me: MessageEvent| {
            let ref data = me.data();
            Self::on_request_inner(atw_thw.clone(), data);
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    fn on_request_inner(atw_thw: Rc<AtwThreadWorker>, task_msg: &JsValue) {
        let (ref name, ref jsv) = decode_task_msg(task_msg);
        debug_ln!("on_request_inner(): task: {}", name);

        match name.as_str() {
            "job-lang" => job::run_job_lang(jsv, atw_thw),
            _ => {
                let msg = format!("unknown task: {}", name);
                console_ln!("err: {}", &msg);
                panic!("{}", msg);
            }
        }
    }
}
