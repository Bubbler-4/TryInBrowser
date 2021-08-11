#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
mod brainfuck;
mod deadfish;
mod example_lang;
mod s10k;
use typemap::{ShareMap, Key};
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/* struct LangImpl<T> {
    name: &'static str,
    homepage: &'static str,
    help: &'static str,
    interpret: fn(&str, &str, &str, &mut T),
} */

struct LangImpls {
    names: Vec<&'static str>,
    homepages: HashMap<&'static str, &'static str>,
    helps: HashMap<&'static str, &'static str>,
    interprets: ShareMap,
}

struct KeyWrapper<T: LangWriter> {
    _content: T,
}

type Interpret<T> = fn(&str, &str, &str, &mut T);

impl<T: LangWriter> Key for KeyWrapper<T> {
    type Value = HashMap<&'static str, Interpret<T>>;
}

static IMPLS: OnceCell<LangImpls> = OnceCell::new();

fn init_impls<T: LangWriter>() {
    let mut names = vec![];
    let mut homepages = HashMap::new();
    let mut helps = HashMap::new();
    let mut interprets = ShareMap::custom();
    let mut interpret_inner = HashMap::new();

    names.push(deadfish::NAME);
    homepages.insert(deadfish::NAME, deadfish::HOMEPAGE);
    helps.insert(deadfish::NAME, deadfish::HELP);
    interpret_inner.insert(deadfish::NAME, deadfish::interpret::<T> as Interpret<T>);

    names.push(brainfuck::NAME);
    homepages.insert(brainfuck::NAME, brainfuck::HOMEPAGE);
    helps.insert(brainfuck::NAME, brainfuck::HELP);
    interpret_inner.insert(brainfuck::NAME, brainfuck::interpret::<T> as Interpret<T>);

    names.push(s10k::NAME);
    homepages.insert(s10k::NAME, s10k::HOMEPAGE);
    helps.insert(s10k::NAME, s10k::HELP);
    interpret_inner.insert(s10k::NAME, s10k::interpret::<T> as Interpret<T>);

    if cfg!(feature = "ui_debug") {
        names.push(example_lang::NAME);
        homepages.insert(example_lang::NAME, example_lang::HOMEPAGE);
        helps.insert(example_lang::NAME, example_lang::HELP);
        interpret_inner.insert(example_lang::NAME, example_lang::interpret::<T> as Interpret<T>);
    }

    interprets.insert::<KeyWrapper<T>>(interpret_inner);
    let _ = IMPLS.set(LangImpls {
        names,
        homepages,
        helps,
        interprets
    });
}

pub fn get_lang_names() -> &'static Vec<&'static str> {
    &IMPLS.get().unwrap().names
}

fn get_help2(lang_name: &str) -> Option<&'static str> {
    IMPLS.get().unwrap().helps.get(lang_name).copied()
}

pub fn get_homepage2(lang_name: &str) -> Option<&'static str> {
    IMPLS.get().unwrap().homepages.get(lang_name).copied()
}

/* pub fn lang_name_list() -> Vec<&'static str> {
    let mut names = vec![s10k::NAME, deadfish::NAME, brainfuck::NAME];
    names.sort_unstable();
    if cfg!(feature = "ui_debug") {
        names.push(example_lang::NAME);
    }
    names
} */

/* fn lookup_impl<T: LangWriter + 'static>(lang_name: &str) -> Option<&LangImpl<T>> {
    let impls = get_impls::<T>();
    impls.get(lang_name)
} */

/* fn get_help(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HELP),
        s10k::NAME => Some(s10k::HELP),
        deadfish::NAME => Some(deadfish::HELP),
        brainfuck::NAME => Some(brainfuck::HELP),
        _ => None,
    }
} */

/* pub fn get_homepage(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HOMEPAGE),
        s10k::NAME => Some(s10k::HOMEPAGE),
        deadfish::NAME => Some(deadfish::HOMEPAGE),
        brainfuck::NAME => Some(brainfuck::HOMEPAGE),
        _ => None,
    }
} */

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

pub fn interpret2<T: LangWriter>(lang: &str, pgm: &str, input: &str, args: &str, writer: &mut T) {
    if args == "-h" {
        if let Some(help) = get_help2(lang) {
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

/* pub fn interpret<T: LangWriter>(lang: &str, pgm: &str, input: &str, args: &str, writer: &mut T) {
    if args == "-h" {
        if let Some(help) = get_help(lang) {
            writer.write_out(help);
            writer.terminate();
            return;
        }
    }
    match lang {
        example_lang::NAME => example_lang::interpret(pgm, input, args, writer),
        s10k::NAME => s10k::interpret(pgm, input, args, writer),
        deadfish::NAME => deadfish::interpret(pgm, input, args, writer),
        brainfuck::NAME => brainfuck::interpret(pgm, input, args, writer),
        _ => {
            let err = format!("Unknown lang: {}", lang);
            writer.write_err(&err);
            writer.terminate_with_error(&err);
            return;
        }
    }
    writer.terminate();
} */
