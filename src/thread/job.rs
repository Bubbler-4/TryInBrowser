use super::atw::ThreadWorker as AtwThreadWorker;
use super::prelude::*;
use crate::lang::{interpret, LangWriter};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

type ResultJJ = Result<JsValue, JsValue>;

pub struct AtwThreadWriter {
    atw_thw: Rc<AtwThreadWorker>,
}

impl AtwThreadWriter {
    pub fn new(atw_thw: Rc<AtwThreadWorker>) -> Self {
        Self { atw_thw }
    }
}

fn pass_encode(out: &str, err: &str) -> ResultJJ {
    /* let (out, err) = (JsValue::from(out), JsValue::from(err));
    let jsv = Array::of2(&out, &err).unchecked_into();
    Ok(jsv) */
    Ok(JsValue::from_serde(&(out, err)).unwrap())
}

fn err_encode(err: &str) -> ResultJJ {
    /* let (out, err) = (JsValue::from(out), JsValue::from(err));
    let jsv = Array::of2(&out, &err).unchecked_into();
    Ok(jsv) */
    Err(JsValue::from(err))
}

impl LangWriter for AtwThreadWriter {
    fn write_both(&mut self, out: &str, err: &str) {
        send_result(pass_encode(out, err), self.atw_thw.clone(), true);
    }
    fn terminate(&mut self) {
        send_result(pass_encode(&"", &""), self.atw_thw.clone(), false);
    }
    fn terminate_with_error(&mut self, msg: &str) {
        send_result(err_encode(msg), self.atw_thw.clone(), false);
    }
}

pub fn send_result(result: ResultJJ, atw_thw: Rc<AtwThreadWorker>, cont: bool) {
    match result {
        // TODO !!!! optimise transferables cases
        Ok(ref ret) => atw_thw.send_response(ret, None, cont),
        Err(ref ret) => atw_thw.send_error(ret),
    }
}

pub fn run_job_lang(jsv: &JsValue, atw_thw: Rc<AtwThreadWorker>) {
    let (lang, pgm, input, args) = jsv
        .into_serde::<(String, String, String, String)>()
        .unwrap();
    console_ln!("run_job_lang: {} {} {} {}", lang, pgm, input, args);
    let mut writer = AtwThreadWriter::new(atw_thw);
    interpret(&lang, &pgm, &input, &args, &mut writer);
}
