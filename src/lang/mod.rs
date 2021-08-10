#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

mod brainfuck;
mod deadfish;
mod example_lang;
mod s10k;

pub fn lang_name_list() -> Vec<&'static str> {
    let mut names = vec![s10k::NAME, deadfish::NAME, brainfuck::NAME];
    names.sort_unstable();
    if cfg!(feature = "ui_debug") {
        names.push(example_lang::NAME);
    }
    names
}

pub fn find_lang(lang_name: &str) -> Option<&'static Language2> {
    LANGS.iter().find(|tib_lang| tib_lang.name == lang_name).copied()
}

fn get_help(lang_name: &str) -> Option<&'static str> {
    find_lang(lang_name).map(|lang| lang.help)
}

pub fn get_homepage(lang_name: &str) -> Option<&'static str> {
    find_lang(lang_name).map(|lang| lang.homepage)
}

pub trait LangWriter {
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

pub struct Language2 {
    name: &'static str,
    help: &'static str,
    homepage: &'static str,
    interpret: fn(pgm: &str, input: &str, args: &str, writer: &mut dyn LangWriter),
}

static LANGS: &[&Language2] = &[&brainfuck::IMPL, &deadfish::IMPL, &s10k::IMPL];

pub fn interpret<T: LangWriter>(lang: &str, pgm: &str, input: &str, args: &str, writer: &mut T) {
    if args == "-h" {
        if let Some(help) = get_help(lang) {
            writer.write_out(help);
            writer.terminate();
            return;
        }
    }

    match find_lang(lang) {
        Some(tib_lang) => {
            (tib_lang.interpret)(pgm, input, args, writer);
            writer.terminate();
        }
        None => {
            let err = format!("Unknown lang: {}", lang);
            writer.write_err(&err);
            writer.terminate_with_error(&err);
        }
    }
}
