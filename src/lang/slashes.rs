use indoc::indoc;

use super::LangWriter;

pub const NAME: &str = "///";
pub const HOMEPAGE: &str = "https://esolangs.org/wiki////";
pub const HELP: &str = indoc!(
    r#"
    /// (https://esolangs.org/wiki////)
    Accepted arguments:
    -h    Show this help and exit

    /pattern/replacement/string replaces all instances of pattern in string with replacement.
    Note that /// doesn't use regex, this is simple string substitution. To escape `/` or `\`,
    you can use `\`.
    "#
);

pub fn interpret<T: LangWriter>(pgm_str: &str, _input: &str, _args: &str, writer: &mut T) {
    let mut mode = Mode::Print;
    let mut patt = Vec::<char>::new();
    let mut repl = Vec::<char>::new();

    let mut pgm_str = pgm_str.to_string();
    let mut pgm = pgm_str.chars();

    loop {
        match pgm.next() {
            Some(chr) => {
                if chr == '/' {
                    mode = match mode {
                        Mode::Print => Mode::Pattern,
                        Mode::Pattern => Mode::Replacement,
                        Mode::Replacement => {
                            //We have a pattern and replacement
                            //so apply substitutions
                            let patt_str = &patt.iter().collect::<String>();
                            let repl_str = &repl.iter().collect::<String>();
                            pgm_str = pgm.collect::<String>();

                            loop {
                                match repl_if_needed(&pgm_str, patt_str, repl_str) {
                                    Some(new_pgm) => {
                                        pgm_str = new_pgm;
                                    }
                                    None => {
                                        //Reset everything
                                        pgm = pgm_str.chars();
                                        patt = Vec::<char>::new();
                                        repl = Vec::<char>::new();
                                        break;
                                    }
                                }
                            }

                            Mode::Print
                        }
                    };
                } else {
                    let mut chr = chr;
                    //Escape, so skip ahead to the next character
                    if chr == '\\' {
                        match pgm.next() {
                            Some(c) => chr = c,
                            None => continue,
                        }
                    }
                    match mode {
                        Mode::Print => writer.write_out(&*chr.to_string()),
                        Mode::Pattern => patt.push(chr),
                        Mode::Replacement => repl.push(chr),
                    }
                }
            }
            None => {
                return;
            }
        }
    }
}

fn repl_if_needed(input: &String, patt: &String, repl: &String) -> Option<String> {
    if input.contains(patt) {
        Some(input.replace(patt, repl))
    } else {
        None
    }
}

#[derive(Debug)]
enum Mode {
    Print,
    Pattern,
    Replacement,
}
