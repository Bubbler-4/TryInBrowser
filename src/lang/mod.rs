#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
mod brainfuck;
mod deadfish;
mod example_lang;
mod s10k;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use typemap::{Key, ShareMap};

pub trait LangWriter: 'static + Sized {
    fn init_impls() {
        init_impls::<Self>();
    }
    fn write_both(&mut self, out: &str, err: &str);
    fn write_out(&mut self, out: &str) {
        self.write_both(out, "");
    }
    fn write_err(&mut self, err: &str) {
        self.write_both("", err);
    }
    fn terminate(&mut self) {}
    fn terminate_with_error(&mut self, _msg: &str) {}
}

struct KeyWrapper<T: LangWriter> {
    _content: T,
}

type Interpret<T> = fn(&str, &str, &str, &mut T);

impl<T: LangWriter> Key for KeyWrapper<T> {
    type Value = HashMap<&'static str, Interpret<T>>;
}

struct LangImpls {
    names: Vec<&'static str>,
    homepages: HashMap<&'static str, &'static str>,
    helps: HashMap<&'static str, &'static str>,
    interprets: ShareMap,
}

static IMPLS: OnceCell<LangImpls> = OnceCell::new();

fn init_impls<T: LangWriter>() {
    let mut names = vec![];
    let mut homepages = HashMap::new();
    let mut helps = HashMap::new();
    let mut interprets = ShareMap::custom();
    let mut interpret_inner = HashMap::new();

    macro_rules! add_lang {
        ($lang: ident) => {
            names.push($lang::NAME);
            homepages.insert($lang::NAME, $lang::HOMEPAGE);
            helps.insert($lang::NAME, $lang::HELP);
            interpret_inner.insert($lang::NAME, $lang::interpret::<T> as Interpret<T>);
        };
    }

    add_lang!(deadfish);
    add_lang!(brainfuck);
    add_lang!(s10k);

    if cfg!(feature = "ui_debug") {
        add_lang!(example_lang);
    }

    interprets.insert::<KeyWrapper<T>>(interpret_inner);
    let _res = IMPLS.set(LangImpls {
        names,
        homepages,
        helps,
        interprets,
    });
}

#[allow(clippy::missing_panics_doc)]
pub fn get_lang_names() -> &'static Vec<&'static str> {
    &IMPLS.get().unwrap().names
}

fn get_help(lang_name: &str) -> Option<&'static str> {
    IMPLS.get().unwrap().helps.get(lang_name).copied()
}

#[allow(clippy::missing_panics_doc)]
pub fn get_homepage(lang_name: &str) -> Option<&'static str> {
    IMPLS.get().unwrap().homepages.get(lang_name).copied()
}

#[allow(clippy::missing_panics_doc)]
pub fn interpret<T: LangWriter>(lang: &str, pgm: &str, input: &str, args: &str, writer: &mut T) {
    T::init_impls();
    if args == "-h" {
        if let Some(help) = get_help(lang) {
            writer.write_out(help);
            writer.terminate();
            return;
        }
    }
    let interprets = &IMPLS.get().unwrap().interprets;
    if let Some(interpret) = interprets.get::<KeyWrapper<T>>().unwrap().get(lang) {
        interpret(pgm, input, args, writer);
        writer.terminate();
    } else {
        let err = format!("Unknown lang: {}", lang);
        writer.write_err(&err);
        writer.terminate_with_error(&err);
    }
}
