use super::LangWriter;
use indoc::indoc;

pub const NAME: &str = "S10K";
pub const HOMEPAGE: &str = "https://try-in-browser.netlify.app/";
pub const HELP: &str = indoc!(
    r#"
    S10K, the first TIB-original language.
    Prints 10,000 copies of "S" and halts.
    "#
);

pub fn interpret<T: LangWriter>(_pgm: &str, _input: &str, _args: &str, writer: &mut T) {
    writer.write_out(&"S".repeat(10000));
    writer.terminate();
}
