use indoc::indoc;

use crate::lang::Language2;

use super::LangWriter;

pub const NAME: &str = "S10K";
pub const HOMEPAGE: &str = "https://try-in-browser.netlify.app/";
pub const HELP: &str = indoc!(
    r#"
    S10K, the first TIB-original language.
    Prints 10,000 copies of "S" and halts.
    "#
);

pub static IMPL: Language2 = Language2 {
    name: NAME,
    help: HELP,
    homepage: HOMEPAGE,
    interpret,
};


pub fn interpret(_pgm: &str, _input: &str, _args: &str, writer: &mut dyn LangWriter) {
    writer.write_out(&"S".repeat(10000));
    writer.terminate();
}
