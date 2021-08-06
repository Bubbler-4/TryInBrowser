use super::prelude::*;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use js_sys::{Array, ArrayBuffer, Object, Reflect};
use web_sys::{Blob, BlobPropertyBag, Url};
use super::atw::Thread as AtwThread;
use super::encode_task_msg;

type ResultJJ = Result<JsValue, JsValue>;

pub struct Thread {
    ab_init: RefCell<Option<ArrayBuffer>>,
    ab_wasm: RefCell<Option<ArrayBuffer>>,
    atw_th: AtwThread,
    is_initialized: RefCell<bool>,
}

impl Thread {
    fn create_blob_url(content: &str) -> String {
        // rust-wasm equivalent of --
        //   return URL.createObjectURL(
        //     new Blob([content], {type: 'text/javascript'}));
        let blob = Blob::new_with_str_sequence_and_options(
                &Array::of1(&JsValue::from(content)),
                BlobPropertyBag::new().type_("text/javascript"))
            .unwrap();

        Url::create_object_url_with_blob(&blob).unwrap()
    }
    fn revoke_blob_url(blob_url: String) {
        Url::revoke_object_url(&blob_url).unwrap();
    }

    fn get_worker_content() -> &'static str {
        "
        const instantiate = async (abInit, abWasm) => {
            const initJs = new TextDecoder().decode(abInit);
            const init = (new Function(initJs)).call(null);
            const wbg = init();
            const wasm = await wbg(abWasm);
            return { wbg, wasm };
        };

        let first = true;
        self.onmessage = async e => {
            const payload = e.data;
            const { abInit, abWasm } = payload;
            if (first) {
                first = false;
                try {
                    const { wbg, wasm } = await instantiate(abInit, abWasm);
                    const _worker = wbg.wmt_bootstrap(self);
                    self.wmtContext = { wbg, wasm, _worker };
                } catch (e) {
                    console.log('bootstrap error:', e);
                }
                return;
            }
            throw 'oh no';
        };
        "
    }

    pub fn new(ab_init: ArrayBuffer, ab_wasm: ArrayBuffer) -> Self {
        let blob_url = Self::create_blob_url(Self::get_worker_content());
        debug_ln!("blob_url: {}", &blob_url);
        let atw_th = AtwThread::new(&blob_url);
        Self::revoke_blob_url(blob_url);

        Self {
            ab_init: RefCell::new(Some(ab_init)),
            ab_wasm: RefCell::new(Some(ab_wasm)),
            atw_th,
            is_initialized: RefCell::new(false),
        }
    }

    pub async fn init(&self) -> Result<&Self, JsValue> {
        let ab_init = self.ab_init.replace(None).unwrap_throw();
        let ab_wasm = self.ab_wasm.replace(None).unwrap_throw();

        let payload = Object::new();
        Reflect::set(payload.as_ref(), &JsValue::from("abInit"), &ab_init).unwrap();
        Reflect::set(payload.as_ref(), &JsValue::from("abWasm"), &ab_wasm).unwrap();

        let result = self.atw_th.send_request(
            &payload, Some(&Array::of2(&ab_init, &ab_wasm))).await;
        let result = match result {
            Ok(jsv) => format!("ok: {}", jsv.as_string().unwrap()),
            Err(jsv) => format!("err: {}", jsv.as_string().unwrap()),
        };
        debug_ln!("init() - result: {}", result);

        self.is_initialized.replace(true);

        Ok(self)
    }

    pub async fn and_init(self) -> Result<Self, JsValue> {
        self.init().await?;
        Ok(self)
    }

    pub async fn exec_lang(&self, lang: &str, pgm: &str, input: &str, args: &str) -> ResultJJ {
        let data = JsValue::from_serde(&(lang, pgm, input, args)).unwrap();
        let msg = encode_task_msg("job-lang", Some(&data));
        self.atw_th.send_request(&msg, None).await
    }

    pub fn terminate(&self) {
        self.atw_th.terminate();
    }

    /* pub fn is_terminated(&self) -> bool {
        self.atw_th.is_terminated()
    } */

    pub fn stdout(&self) -> String {
        self.atw_th.stdout()
    }

    pub fn stderr(&self) -> String {
        self.atw_th.stderr()
    }
}
