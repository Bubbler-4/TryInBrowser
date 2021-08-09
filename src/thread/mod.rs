use js_sys::{ArrayBuffer, Object, Reflect};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

mod atw;
mod job;
pub mod prelude;
mod thread;
pub mod utils;
mod worker;
//mod lang;

pub use thread::Thread;

#[macro_export]
macro_rules! console_ln {
    ( $( $x:expr ),* ) => (web_sys::console::log_1(&format!( $( $x ),* ).into()));
}

#[macro_export]
macro_rules! debug_ln {
    ( $( $x:expr ),* ) => {
        if cfg!(debug_assertions) {
            let mut ln = String::from("👀 ");
            ln.push_str(&format!( $( $x ),* ));
            web_sys::console::log_1(&ln.into());
        }
    };
}

#[macro_export]
macro_rules! exec_lang {
    ($th:expr, $str1:expr, $str2:expr, $str3:expr, $str4:expr) => {
        ($th).exec_lang($str1, $str2, $str3, $str4)
    };
}

pub const OUT_LIMIT: usize = 131_072;

pub struct WasmMt {
    pkg_js_uri: Option<String>,
    ab_init: RefCell<Option<ArrayBuffer>>,
    ab_wasm: RefCell<Option<ArrayBuffer>>,
    is_initialized: RefCell<bool>,
}

impl WasmMt {
    pub fn new(pkg_js_uri: &str) -> Self {
        debug_ln!("pkg_js_uri: {}", pkg_js_uri);

        Self {
            pkg_js_uri: Some(String::from(pkg_js_uri)),
            ab_init: RefCell::new(None),
            ab_wasm: RefCell::new(None),
            is_initialized: RefCell::new(false),
        }
    }

    pub fn set_ab_init(&self, ab: ArrayBuffer) {
        self.ab_init.replace(Some(ab));
    }

    pub fn set_ab_wasm(&self, ab: ArrayBuffer) {
        self.ab_wasm.replace(Some(ab));
    }

    pub async fn init(&self) -> Result<&Self, JsValue> {
        assert!(!*self.is_initialized.borrow());
        self.is_initialized.replace(true);

        if let Some(ref pkg_js_uri) = self.pkg_js_uri {
            let pkg_wasm_uri = if pkg_js_uri.ends_with("wasm-bindgen-test") {
                // We defer updating `self.ab_init` in this 'test' context
                format!("{}_bg.wasm", pkg_js_uri)
            } else {
                self.set_ab_init(Self::create_ab_init(pkg_js_uri).await?);
                pkg_js_uri.replace(".js", "_bg.wasm")
            };

            if !pkg_wasm_uri.ends_with("_bg.wasm") {
                wasm_bindgen::throw_str("failed to resolve `pkg_wasm_uri`");
            }

            self.set_ab_wasm(utils::fetch_as_arraybuffer(&pkg_wasm_uri).await?);
        } else {
            debug_ln!("init(): `pkg_js_uri` is `None`; should be using `new_with_arraybuffers()`");
            assert!(self.ab_init.borrow().is_some());
            assert!(self.ab_wasm.borrow().is_some());
        }

        Ok(self)
    }

    pub async fn and_init(self) -> Result<Self, JsValue> {
        self.init().await?;
        Ok(self)
    }

    pub fn thread(&self) -> Thread {
        assert!(*self.is_initialized.borrow());

        // https://rustwasm.github.io/wasm-bindgen/api/js_sys/struct.ArrayBuffer.html#method.slice
        Thread::new(
            self.ab_init.borrow().as_ref().unwrap().slice(0),
            self.ab_wasm.borrow().as_ref().unwrap().slice(0),
        )
    }

    fn ab_init_from(pkg_js: &str) -> ArrayBuffer {
        let mut init_js = String::new();
        init_js.push_str("return () => { ");
        init_js.push_str(pkg_js);
        init_js.push_str(" return wasm_bindgen; };");

        utils::ab_from_text(&init_js)
    }

    async fn create_ab_init(pkg_js_uri: &str) -> Result<ArrayBuffer, JsValue> {
        let pkg_js = utils::fetch_as_text(pkg_js_uri).await?;

        Ok(Self::ab_init_from(&pkg_js))
    }
}

fn encode_task_msg(name: &str, data: Option<&JsValue>) -> Object {
    let msg = Object::new();
    Reflect::set(msg.as_ref(), &JsValue::from("task"), &JsValue::from(name)).unwrap();
    if let Some(jsv) = data {
        Reflect::set(msg.as_ref(), &JsValue::from("data"), jsv).unwrap();
    }
    msg
}

fn decode_task_msg(msg: &JsValue) -> (String, JsValue) {
    let name = Reflect::get(msg, &JsValue::from("task"))
        .unwrap_throw()
        .as_string()
        .unwrap_throw();
    let jsv = Reflect::get(msg, &JsValue::from("data")).unwrap_throw();
    (name, jsv)
}
