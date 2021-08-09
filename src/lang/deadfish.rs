use super::LangWriter;
use indoc::indoc;

pub const NAME: &'static str = "Deadfish";
pub const HOMEPAGE: &'static str = "https://esolangs.org/wiki/Deadfish";
pub const HELP: &'static str = indoc!(r#"
    Deadfish (https://esolangs.org/wiki/Deadfish)
    Accepted arguments:
    -h    Show this help and exit
    -o    Output as charcode
    -n    Output as number (default)
"#);

pub fn interpret<T: LangWriter>(pgm: &str, _input: &str, args: &str, writer: &mut T) {
    let mut counter = 0;
    let is_char_output = args == "-o";
    for b in pgm.bytes() {
        match b {
            b'i' => {
                counter = if counter == 255 { 0 } else { counter + 1 };
            }
            b'd' => {
                counter = if counter == 0 || counter == 257 { 0 } else { counter - 1 };
            }
            b's' => {
                counter *= if counter == 16 { 0 } else { counter };
            }
            b'o' => {
                if is_char_output {
                    writer.write_out(&((counter % 256) as u8 as char).to_string());
                } else {
                    writer.write_out(&format!("{}\n", counter));
                }
            }
            _ => ()
        }
    }
    writer.terminate();
}
