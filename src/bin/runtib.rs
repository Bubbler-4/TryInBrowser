use std::io::{self, Read};
use std::fs::read_to_string;
use std::env::args;
use try_in_browser::lang::{interpret, LangWriter};

struct StdWriter {
}

impl StdWriter {
    fn new() -> Self {
        Self {}
    }
}

impl LangWriter for StdWriter {
    fn write_both(&mut self, out: &str, err: &str) {
        print!("{}", out);
        eprint!("{}", err);
    }
    fn terminate(&mut self) {
        std::process::exit(0);
    }
    fn terminate_with_error(&mut self, msg: &str) {
        eprint!("{}", msg);
        std::process::exit(1);
    }
}

fn main() {
    let mut args = args();
    let _ = args.next(); // discard binary name
    let lang = if let Some(lang) = args.next() { lang } else {
        eprintln!("Error: Missing language name");
        return;
    };
    if lang == "-h" {
        println!("Usage: runtib <language> <sourcefile> [arg]");
        return;
    }
    let file = if let Some(file) = args.next() { file } else {
        eprintln!("Error: Missing source filename");
        return;
    };
    let arg = args.next().unwrap_or_default();
    let pgm = if let Ok(pgm) = read_to_string(&file) { pgm } else {
        eprintln!("Error: Error encountered while reading source code");
        return;
    };
    let mut stdin = String::new();
    if io::stdin().read_to_string(&mut stdin).is_err() {
        eprintln!("Error: Error encountered while reading stdin");
        return;
    }

    let mut writer = StdWriter::new();
    interpret(&lang, &pgm, &stdin, &arg, &mut writer);
}