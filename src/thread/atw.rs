// rust-wasm porting of -- https://github.com/w3reality/async-thread-worker

use super::prelude::*;
use js_sys::{Array, Function, Object, Promise, Reflect};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MessageEvent, Worker, WorkerGlobalScope};

fn atw_encode_result_msg(result: &JsValue, is_ok: bool, cont: bool) -> Object {
    let msg = Object::new();
    Reflect::set(msg.as_ref(), &JsValue::from("result"), result).unwrap();
    Reflect::set(msg.as_ref(), &JsValue::from("isOk"), &JsValue::from(is_ok)).unwrap();
    Reflect::set(msg.as_ref(), &JsValue::from("cont"), &JsValue::from(cont)).unwrap();
    msg
}

fn atw_decode_result_msg(msg: &JsValue) -> (JsValue, bool, bool) {
    let result = Reflect::get(msg, &JsValue::from("result")).unwrap_throw();
    let is_ok = Reflect::get(msg, &JsValue::from("isOk"))
        .unwrap_throw()
        .as_bool()
        .unwrap_throw();
    let cont = Reflect::get(msg, &JsValue::from("cont"))
        .unwrap_throw()
        .as_bool()
        .unwrap_throw();
    (result, is_ok, cont)
}

// Bindings such as `post_message_with_transfer()` seem not available
// in `web_sys::WorkerGlobalScope` (as opposed to `web_sys::Worker`).
// So, we define and use a custom binding `JsWgs` instead.

pub struct ThreadWorker {
    wgs: JsWgs,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = _)]
    type JsWgs;

    #[wasm_bindgen(method, js_name = postMessage)]
    fn post_message_with_transfer(this: &JsWgs, data: &JsValue, transfer: &Array);
}

impl JsWgs {
    fn new(wgs: WorkerGlobalScope) -> Self {
        wgs.unchecked_into::<JsWgs>()
    }
}

impl ThreadWorker {
    pub fn new(wgs: WorkerGlobalScope) -> Self {
        Self {
            wgs: JsWgs::new(wgs),
        }
    }

    pub fn send_response(&self, payload: &JsValue, transfer: Option<&Array>, cont: bool) {
        let default = Array::new();
        let transfer = transfer.unwrap_or(&default);
        self.wgs
            .post_message_with_transfer(&atw_encode_result_msg(payload, true, cont), transfer);
    }

    pub fn send_error(&self, error: &JsValue) {
        self.wgs
            .post_message_with_transfer(&atw_encode_result_msg(error, false, false), &Array::new());
    }

    pub fn set_callback_of(&self, target: &str, cb: &JsValue) {
        Reflect::set(
            &self.wgs,
            &JsValue::from(target),
            &cb.unchecked_ref::<Function>().to_owned(),
        )
        .unwrap_throw();
    }
}

pub struct Thread {
    worker: Worker,
    _on_message: Box<Closure<dyn FnMut(MessageEvent)>>,
    _on_error: Box<Closure<dyn FnMut(MessageEvent)>>,
    resrej: Rc<RefCell<Option<(Function, Function)>>>,
    messages: Rc<RefCell<(String, String)>>,
    is_terminated: RefCell<bool>,
}

impl Thread {
    pub fn new(script_url: &str) -> Self {
        let worker = Worker::new(script_url);
        if let Err(ref jsv) = worker {
            console_ln!("error: {:?}", jsv);

            // https://developer.mozilla.org/en-US/docs/Web/API/Worker
            // https://bugs.webkit.org/show_bug.cgi?id=22723
            // https://wpt.fyi/results/workers/semantics/multiple-workers/003.html
            console_ln!("Hint: On Safari, nested Web Workers might not be supported as of now.");
        }
        let worker = worker.unwrap_throw();

        let resrej = Rc::new(RefCell::new(None));
        let messages = Rc::new(RefCell::new((
            String::with_capacity(OUT_LIMIT),
            String::with_capacity(OUT_LIMIT),
        )));
        let on_message = Self::create_onmessage(resrej.clone(), messages.clone());
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref::<Function>()));
        let on_error = Self::create_onerror(resrej.clone());
        worker.set_onerror(Some(on_error.as_ref().unchecked_ref::<Function>()));

        Self {
            worker,
            _on_message: Box::new(on_message),
            _on_error: Box::new(on_error),
            resrej,
            messages,
            is_terminated: RefCell::new(false),
        }
    }

    fn create_onmessage(
        resrej: Rc<RefCell<Option<(Function, Function)>>>,
        messages: Rc<RefCell<(String, String)>>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |me: MessageEvent| {
            let msg = me.data();

            if msg == JsValue::NULL {
                debug_ln!("on_message(): msg: {:?}; oops, `.await` will hang!!", msg);
                return;
            }

            let (result, is_ok, cont) = atw_decode_result_msg(&msg);

            if cont {
                let (out, err) = JsValue::into_serde::<(String, String)>(&result).unwrap_throw();
                {
                    let out_collect = &mut messages.borrow_mut().0;
                    if out_collect.len() + out.len() <= out_collect.capacity() {
                        out_collect.push_str(&out);
                        //console_ln!("out_collect: {}", out_collect);
                    } else {
                        out_collect.push_str(&out[..out_collect.capacity() - out_collect.len()]);
                        let resrej_borrow = resrej.borrow();
                        assert!(resrej_borrow.is_some());
                        let (_res, rej) = resrej_borrow.as_ref().unwrap_throw();
                        rej.call1(&JsValue::NULL, &JsValue::from("Stdout limit exceeded"))
                            .unwrap();
                    }
                }
                {
                    let err_collect = &mut messages.borrow_mut().1;
                    if err_collect.len() + err.len() <= err_collect.capacity() {
                        err_collect.push_str(&err);
                        //console_ln!("err_collect: {}", err_collect);
                    } else {
                        err_collect.push_str(&err[..err_collect.capacity() - err_collect.len()]);
                        let resrej_borrow = resrej.borrow();
                        assert!(resrej_borrow.is_some());
                        let (_res, rej) = resrej_borrow.as_ref().unwrap_throw();
                        rej.call1(&JsValue::NULL, &JsValue::from("Stderr limit exceeded"))
                            .unwrap();
                    }
                }
            } else {
                let resrej_borrow = resrej.borrow();
                assert!(resrej_borrow.is_some());
                let (result, reject) = resrej_borrow.as_ref().unwrap_throw();
                (if is_ok { result } else { reject })
                    .call1(&JsValue::NULL, &result)
                    .unwrap_throw();
            }
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    fn create_onerror(
        resrej: Rc<RefCell<Option<(Function, Function)>>>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::wrap(Box::new(move |_me: MessageEvent| {
            console_ln!("terminated by error");
            let resrej_borrow = resrej.borrow();
            assert!(resrej_borrow.is_some());
            let (_res, rej) = resrej_borrow.as_ref().unwrap_throw();
            rej.call1(&JsValue::NULL, &JsValue::from("Thread: last req canceled"))
                .unwrap();
        }) as Box<dyn FnMut(MessageEvent)>)
    }

    pub async fn send_request(
        &self,
        payload: &JsValue,
        transfer: Option<&Array>,
    ) -> Result<JsValue, JsValue> {
        let promise = Promise::new(&mut |res, rej| {
            if *self.is_terminated.borrow() {
                rej.call1(&JsValue::NULL, &JsValue::from("worker already terminated"))
                    .unwrap_throw();
                return;
            }

            self.resrej.borrow_mut().insert((res, rej));
            self.messages.borrow_mut().0.clear();
            self.messages.borrow_mut().1.clear();

            let default = Array::new();
            let transfer = transfer.unwrap_or(&default);
            self.worker
                .post_message_with_transfer(payload, transfer)
                .unwrap_throw();
        });

        JsFuture::from(promise).await
    }

    fn cancel_pending_requests(&self) {
        let resrej_borrow = self.resrej.borrow();
        assert!(resrej_borrow.is_some());
        let (_res, rej) = resrej_borrow.as_ref().unwrap_throw();
        rej.call1(&JsValue::NULL, &JsValue::from("Thread: last req canceled"))
            .unwrap();
    }

    pub fn terminate(&self) {
        if *self.is_terminated.borrow() {
            debug_ln!("Thread::terminate(): nop; already terminated");
        } else {
            self.is_terminated.replace(true);
            self.cancel_pending_requests();
            self.worker.terminate();
            console_ln!("Thread::terminate() complete");
        }
    }

    /* pub fn is_terminated(&self) -> bool {
        *self.is_terminated.borrow()
    } */

    pub fn stdout(&self) -> String {
        let out = &mut self.messages.borrow_mut().0;
        let ret = out.clone();
        out.clear();
        ret
    }

    pub fn stderr(&self) -> String {
        let err = &mut self.messages.borrow_mut().1;
        let ret = err.clone();
        err.clear();
        ret
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        debug_ln!("Thread::drop(): called");
        self.terminate();
    }
}
