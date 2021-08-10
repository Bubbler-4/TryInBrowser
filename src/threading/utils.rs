use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Response, /*TextDecoder,*/ TextEncoder};

/* use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::cell::RefCell; */

pub use super::prelude::*;

/* pub struct Counter(RefCell<usize>);
impl Counter {
    pub fn new() -> Self {
        Self(RefCell::new(0))
    }
    pub fn inc(&self) -> usize {
        let mut num = self.0.borrow_mut();
        *num += 1;
        *num
    }
    pub fn num(&self) -> usize {
        *self.0.borrow()
    }
} */

// https://github.com/rustwasm/wasm-bindgen/blob/master/examples/performance/src/lib.rs
/* pub fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = ((amt as u32) % 1_000) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
} */
// let start = perf_to_system(performance.timing().request_start());
// let end = perf_to_system(performance.timing().response_end());
// console_ln!("request started at {}", humantime::format_rfc3339(start));
// console_ln!("request ended at {}", humantime::format_rfc3339(end));

pub fn u8arr_from_vec(vec: &[u8]) -> Uint8Array {
    let arr = js_sys::Array::new();
    for &el in vec {
        arr.push(&JsValue::from(el));
    }

    Uint8Array::new(&arr)
}

pub fn ab_from_text(text: &str) -> ArrayBuffer {
    u8arr_from_vec(&TextEncoder::new().unwrap().encode_with_input(text)).buffer()
}
/* pub fn text_from_ab(ab: &ArrayBuffer) -> Option<String> {
    TextDecoder::new().ok()?
        .decode_with_buffer_source(ab).ok()
} */

pub async fn fetch_and_response(url: &str) -> Result<Response, JsValue> {
    // debug_ln!("fetch_and_response(): url: {}", url);

    //====
    // kludge: force cast `WorkerGlobalScope` as `Window` and use `::fetch_with_str()`
    // cf. `pub fn window() -> Option<Window> {...}` in wasm-bindgen/crates/web-sys/src/lib.rs
    let obj = js_sys::global().unchecked_into::<web_sys::Window>();
    // debug_ln!("fetch_and_response(): obj: {:?}", obj);
    //====
    // let obj = js_sys::Function::new_no_args("return self;") // hack'ish
    //     .call0(&JsValue::NULL)
    //     .unwrap_throw()
    //     .unchecked_into::<web_sys::Window>();
    //====
    // cf: another approach to tell the context -- call Window/WorkerGlobalScope getters and compare with `.is_undefined()`
    // https://github.com/rustwasm/gloo/pull/106/files

    let ret = JsFuture::from(obj.fetch_with_str(url))
        .await?
        .unchecked_into::<Response>();
    Ok(ret)
}
pub async fn fetch_as_text(url: &str) -> Result<String, JsValue> {
    let resp = fetch_and_response(url).await?;
    let ret = JsFuture::from(resp.text()?)
        .await?
        .as_string()
        .unwrap_throw();
    Ok(ret)
}
pub async fn fetch_as_arraybuffer(url: &str) -> Result<ArrayBuffer, JsValue> {
    let resp = fetch_and_response(url).await?;
    let ret = JsFuture::from(resp.array_buffer()?)
        .await?
        .unchecked_into::<ArrayBuffer>();
    Ok(ret)
}
