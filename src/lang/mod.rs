mod example_lang;

pub fn lang_name_list() -> Vec<&'static str> {
    vec![example_lang::NAME]
}

fn get_help(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HELP),
        _ => None
    }
}

pub fn get_homepage(lang_name: &str) -> Option<&'static str> {
    match lang_name {
        example_lang::NAME => Some(example_lang::HOMEPAGE),
        _ => None
    }
}

pub trait LangWriter {
    fn write_both(&mut self, out: &str, err: &str);
    fn write_out(&mut self, out: &str) {
        self.write_both(out, &"");
    }
    fn write_err(&mut self, err: &str) {
        self.write_both(&"", err);
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
        _ => {
            let err = format!("Unknown lang: {}", lang);
            writer.write_err(&err);
            writer.terminate_with_error(&err);
            return;
        }
    }
    writer.terminate();
}
