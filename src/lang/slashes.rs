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
    let mut patt = String::new();
    let mut repl = String::new();

    let mut pgm_str = pgm_str.to_string();
    let mut pgm = pgm_str.chars();

    while let Some(chr) = pgm.next() {
        if chr == '/' {
            mode = match mode {
                Mode::Print => Mode::Pattern,
                Mode::Pattern => Mode::Replacement,
                Mode::Replacement => {
                    // Substitute using pattern and replacement
                    pgm_str = pgm.collect::<String>();

                    while let Some(new_pgm) = repl_if_needed(&pgm_str, &patt, &repl) {
                        pgm_str = new_pgm;
                    }
                    // Reset everything
                    pgm = pgm_str.chars();
                    patt.clear();
                    repl.clear();

                    Mode::Print
                }
            };
        } else {
            // Escape if backslash, so skip ahead to the next character
            let chr = if chr == '\\' {
                match pgm.next() {
                    Some(c) => c,
                    None => continue,
                }
            } else {
                chr
            };
            match mode {
                Mode::Print => writer.write_out(&chr.to_string()),
                Mode::Pattern => patt.push(chr),
                Mode::Replacement => repl.push(chr),
            }
        }
    }
}

fn repl_if_needed(input: &str, patt: &str, repl: &str) -> Option<String> {
    input.contains(patt).then(|| input.replace(patt, repl))
}

#[derive(Debug)]
enum Mode {
    Print,
    Pattern,
    Replacement,
}
