#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]
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

fn get_help(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HELP),
        s10k::NAME => Some(s10k::HELP),
        deadfish::NAME => Some(deadfish::HELP),
        brainfuck::NAME => Some(brainfuck::HELP),
        _ => None,
    }
}

pub fn get_homepage(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HOMEPAGE),
        s10k::NAME => Some(s10k::HOMEPAGE),
        deadfish::NAME => Some(deadfish::HOMEPAGE),
        brainfuck::NAME => Some(brainfuck::HOMEPAGE),
        _ => None,
    }
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

pub fn interpret<T: LangWriter>(lang: &str, pgm: &str, input: &str, args: &str, writer: &mut T) {
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
}
